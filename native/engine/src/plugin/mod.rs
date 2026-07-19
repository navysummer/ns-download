#[cfg(feature = "plugins")]
pub mod bridge;
#[cfg(feature = "plugins")]
pub mod dependencies;
#[cfg(feature = "plugins")]
pub mod install;
#[cfg(feature = "plugins")]
pub mod manager;
#[cfg(feature = "plugins")]
pub mod manifest;
#[cfg(feature = "plugins")]
pub mod market;
#[cfg(feature = "plugins")]
pub mod quickjs;
#[cfg(feature = "plugins")]
pub mod runtime;
#[cfg(feature = "plugins")]
pub mod semver;

#[cfg(feature = "plugins")]
pub use manager::PluginManager;

#[cfg(not(feature = "plugins"))]
pub struct PluginManager;

#[cfg(not(feature = "plugins"))]
impl PluginManager {
    pub fn new(
        _runtime: std::sync::Arc<dyn ScriptRuntime>,
        _bridge: std::sync::Arc<dyn PluginBridge>,
        _db: crate::db::Db,
        _plugins_root: std::path::PathBuf,
        _app_version: String,
        _sink: std::sync::Arc<dyn crate::events::EventSink>,
    ) -> Self { Self }
    pub async fn load_all(&self) {}
    pub fn match_resolver(&self, _url: &str) -> Option<String> { None }
    pub fn runtime_handle(&self) -> tokio::runtime::Handle { tokio::runtime::Handle::current() }
    pub fn plugins(&self) -> Vec<PluginInfo> { Vec::new() }
    pub fn notify(&self, _event: PluginEvent) {}
    pub fn resolve(&self, _task_id: &str, _req: ResolveRequest) -> Option<ResolveResult> { None }
}

use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct PluginInfo {
    pub identity: String,
    pub name: String,
    pub version: String,
    pub author: String,
    pub description: String,
    pub homepage: String,
    pub enabled: bool,
}

#[derive(Debug)]
pub struct ResolveRequest {
    pub url: String,
    pub task_id: String,
    pub referrer: String,
    pub user_agent: String,
    pub cookies: String,
    pub extra_headers: HashMap<String, String>,
    pub body: Option<crate::downloader::CapturedRequestBody>,
    pub method: Option<String>,
}

#[derive(Debug, Default)]
pub struct ResolveResult {
    pub url: String,
    pub file_name: Option<String>,
    pub total_bytes: Option<i64>,
    pub extra_headers: Option<HashMap<String, String>>,
    pub audio_url: Option<String>,
    pub ephemeral: bool,
    pub range_supported: bool,
    pub default_variant_index: i32,
    pub variants: Vec<ResolveVariant>,
}

#[derive(Debug)]
pub struct ResolveVariant {
    pub url: String,
    pub label: String,
    pub container: String,
    pub bandwidth: i64,
    pub width: i64,
    pub height: i64,
    pub file_name: Option<String>,
    pub total_bytes: Option<i64>,
    pub audio_url: Option<String>,
}

#[derive(Debug, thiserror::Error)]
pub enum PluginError {
    #[error("plugin not found: {0}")]
    NotFound(String),
    #[error("plugin error: {0}")]
    Runtime(String),
    #[error("plugin timeout")]
    Timeout,
    #[error("plugin disabled: {0}")]
    Disabled(String),
}

#[derive(Debug, Clone)]
pub enum PluginEvent {
    Done {
        task_id: String,
        url: String,
        file_path: Option<String>,
        audio_path: Option<String>,
        muxed: bool,
    },
    Error {
        task_id: String,
        url: String,
        message: String,
    },
    MetaProbed {
        task_id: String,
        url: String,
        file_name: String,
        total_bytes: i64,
    },
    Start {
        task_id: String,
        url: String,
    },
}

pub trait PluginBridge: Send + Sync {
    fn resolve(&self, request: ResolveRequest) -> Result<Option<ResolveResult>, PluginError>;
}

pub trait ScriptRuntime: Send + Sync {
    fn execute(&self, script: &str, context: &str) -> Result<String, PluginError>;
}

pub struct MarketClient;

impl MarketClient {
    pub fn new(_pm: std::sync::Arc<PluginManager>, _db: crate::db::Db, _sources: Vec<String>) -> Self {
        Self
    }
    pub fn source_config(_config: &HashMap<String, String>) -> Vec<String> {
        Vec::new()
    }
}
