use crate::db::Db;
use crate::downloader::DownloadError;
use crate::plugin::{PluginBridge, PluginError, ResolveRequest, ResolveResult};
use crate::proxy_config::ProxyConfig;

#[allow(dead_code)]
pub struct EngineBridge {
    db: Db,
    retry_tx: tokio::sync::mpsc::UnboundedSender<(String, u64)>,
}

impl EngineBridge {
    pub fn new(
        db: Db,
        _proxy_config: &ProxyConfig,
        retry_tx: tokio::sync::mpsc::UnboundedSender<(String, u64)>,
        _data_dir: std::path::PathBuf,
    ) -> Result<Self, DownloadError> {
        Ok(Self { db, retry_tx })
    }
}

impl PluginBridge for EngineBridge {
    fn resolve(&self, _request: ResolveRequest) -> Result<Option<ResolveResult>, PluginError> {
        Ok(None)
    }
}
