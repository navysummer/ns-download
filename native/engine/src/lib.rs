#[macro_export]
macro_rules! log_info {
    ($($arg:tt)*) => {
        tracing::info!($($arg)*)
    };
}

#[macro_export]
macro_rules! log_error {
    ($($arg:tt)*) => {
        tracing::error!($($arg)*)
    };
}

#[macro_export]
macro_rules! log_warn {
    ($($arg:tt)*) => {
        tracing::warn!($($arg)*)
    };
}

pub mod bt_downloader;
pub mod components;
pub mod dash_downloader;
pub mod data_dir;
pub mod db;
pub mod disk_space;
pub mod download_manager;
pub mod downloader;
pub mod ed2k;
pub mod events;
pub mod ftp_downloader;
pub mod hls_downloader;
pub mod meta_prober;
pub mod model;
pub mod plugin;
mod proc;
pub mod proxy_config;
pub mod segment_advisor;
pub mod segment_coordinator;
pub mod selection;
pub mod speed_limiter;
pub mod tracker_subscription;

use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;

use bt_downloader::BtConfig;
use db::{Db, DbError};
use download_manager::{DownloadManager, DownloadManagerConfig};
use downloader::DownloadError;
use events::{EngineEvent, EventSink};
use model::{BtFileEntry, HlsQualityOption, TorrentMetaResult};
use proxy_config::ProxyConfig;
use selection::{HostSelection, SelectionOutcome};

pub struct EngineConfig {
    pub max_concurrent: usize,
    pub speed_limit_bps: u64,
    pub default_save_dir: String,
    pub app_data_dir: String,
    pub bt_config: BtConfig,
    pub proxy_config: ProxyConfig,
    pub user_agent: String,
    pub data_dir_override: Option<PathBuf>,
    pub database_url: Option<String>,
}

#[derive(Debug, thiserror::Error)]
pub enum EngineError {
    #[error(transparent)]
    Db(#[from] DbError),
    #[error(transparent)]
    Download(#[from] DownloadError),
    #[error(transparent)]
    DataDir(#[from] data_dir::DataDirError),
    #[error("plugin system init failed: {0}")]
    Plugin(String),
}

pub struct Engine {
    pub db: Db,
    pub manager: DownloadManager,
    pub selector: Arc<dyn HostSelection>,
    pub data_dir: PathBuf,
}

impl Engine {
    pub async fn new(
        config: EngineConfig,
        sink: Arc<dyn EventSink>,
        selector: Arc<dyn HostSelection>,
    ) -> Result<Self, EngineError> {
        let data_dir = data_dir::resolve_data_dir(config.data_dir_override.as_deref())?;
        let db = match &config.database_url {
            Some(url) => Db::connect(url).await?,
            None => Db::open(&data_dir).await?,
        };
        segment_coordinator::load_domain_conn_caps(&db).await;
        db.seed_builtin_queues().await?;

        #[cfg(feature = "plugins")]
        let plugin_ctx = (
            config.proxy_config.clone(),
            sink.clone(),
            config.max_concurrent,
            data_dir.clone(),
        );

        #[cfg_attr(not(feature = "plugins"), allow(unused_mut))]
        let mut manager = DownloadManager::new(
            db.clone(),
            DownloadManagerConfig {
                max_concurrent: config.max_concurrent,
                speed_limit_bps: config.speed_limit_bps,
                default_save_dir: config.default_save_dir,
                app_data_dir: config.app_data_dir,
                data_dir: data_dir.clone(),
                bt_config: config.bt_config,
                proxy_config: config.proxy_config,
                user_agent: config.user_agent,
            },
            sink,
            selector.clone(),
        )?;

        #[cfg(feature = "plugins")]
        {
            let (proxy_cfg, plugin_sink, max_conc, data_dir_p) = plugin_ctx;
            let retry_tx = manager.plugin_retry_sender();
            let bridge: Arc<dyn plugin::PluginBridge> = Arc::new(
                plugin::bridge::EngineBridge::new(
                    db.clone(), &proxy_cfg, retry_tx, data_dir_p.clone(),
                ).map_err(|e| EngineError::Plugin(e.to_string()))?,
            );
            let runtime: Arc<dyn plugin::ScriptRuntime> = Arc::new(
                plugin::quickjs::QuickJsScriptRuntime::new(max_conc)
                    .map_err(|e| EngineError::Plugin(e.to_string()))?,
            );
            let app_version = db.get_config("app_version").await.ok().flatten().unwrap_or_default();
            let plugins_root = data_dir_p.join("plugins");
            let _ = tokio::fs::create_dir_all(&plugins_root).await;
            let pm = Arc::new(plugin::PluginManager::new(
                runtime, bridge, db.clone(), plugins_root, app_version, plugin_sink,
            ));
            pm.load_all().await;
            manager.install_plugin_manager(pm);
        }

        Ok(Self { db, manager, selector, data_dir })
    }

    pub async fn test_proxy_connection(
        &self, proxy_type: &str, proxy_host: &str, proxy_port: &str,
        proxy_username: &str, proxy_password: &str,
    ) -> Result<i64, DownloadError> {
        proxy_config::test_proxy_connection(proxy_type, proxy_host, proxy_port, proxy_username, proxy_password).await
    }

    pub async fn probe_torrent_meta(&self, probe_id: String, torrent_bytes: Vec<u8>) -> TorrentMetaResult {
        let probe_id_for_panic = probe_id.clone();
        tokio::task::spawn_blocking(move || bt_downloader::probe_torrent_meta(probe_id, torrent_bytes))
            .await
            .unwrap_or_else(|e| TorrentMetaResult {
                probe_id: probe_id_for_panic, name: String::new(), total_bytes: 0,
                files: Vec::new(), error: format!("probe task panicked: {e}"),
            })
    }
}

pub struct NoopSink;

impl EventSink for NoopSink {
    fn emit(&self, event: EngineEvent) {
        tracing::debug!("[noop-sink] {event:?}");
    }
}

pub struct NoopSelection;

#[async_trait::async_trait]
impl HostSelection for NoopSelection {
    async fn select_hls_quality(&self, _task_id: &str, options: &[HlsQualityOption], _timeout: Duration) -> SelectionOutcome<i32> {
        let best = options.iter().enumerate().max_by_key(|(_, o)| o.bandwidth).map(|(i, _)| i as i32).unwrap_or(0);
        SelectionOutcome::NoSelectorConfigured(best)
    }
    async fn select_bt_files(&self, _task_id: &str, _files: &[BtFileEntry], _timeout: Option<Duration>) -> SelectionOutcome<Vec<i32>> {
        SelectionOutcome::NoSelectorConfigured(Vec::new())
    }
    async fn select_resolve_variant(&self, _task_id: &str, _options: &[crate::model::ResolveVariantOption], default_index: i32, _timeout: Duration) -> SelectionOutcome<i32> {
        SelectionOutcome::NoSelectorConfigured(default_index)
    }
    fn provide_hls_selection(&self, _task_id: &str, _selected_index: i32) {}
    fn provide_bt_selection(&self, _task_id: &str, _selected_indices: Vec<i32>) {}
    fn provide_variant_selection(&self, _task_id: &str, _selected_index: i32) {}
}
