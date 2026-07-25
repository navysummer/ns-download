use std::sync::Arc;
use crate::db::Db;
use crate::events::EventSink;
use crate::plugin::{PluginBridge, PluginInfo, ScriptRuntime};

#[allow(dead_code)]
pub struct PluginManager {
    runtime: Arc<dyn ScriptRuntime>,
    bridge: Arc<dyn PluginBridge>,
    db: Db,
    plugins_root: std::path::PathBuf,
    app_version: String,
    sink: Arc<dyn EventSink>,
}

impl PluginManager {
    pub fn new(
        runtime: Arc<dyn ScriptRuntime>,
        bridge: Arc<dyn PluginBridge>,
        db: Db,
        plugins_root: std::path::PathBuf,
        app_version: String,
        sink: Arc<dyn EventSink>,
    ) -> Self {
        Self { runtime, bridge, db, plugins_root, app_version, sink }
    }

    pub async fn load_all(&self) {
        // Stub
    }

    pub fn match_resolver(&self, _url: &str) -> Option<String> {
        None
    }

    pub fn runtime_handle(&self) -> tokio::runtime::Handle {
        tokio::runtime::Handle::current()
    }

    pub fn plugins(&self) -> Vec<PluginInfo> {
        Vec::new()
    }

    pub async fn notify(&self, _event: crate::plugin::PluginEvent) {}

    pub async fn resolve(
        &self,
        _plugin_identity: &str,
        _req: crate::plugin::ResolveRequest,
    ) -> Result<Option<crate::plugin::ResolveResult>, crate::plugin::PluginError> {
        Ok(None)
    }
}
