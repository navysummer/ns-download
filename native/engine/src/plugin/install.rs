use crate::plugin::PluginError;

pub async fn install_plugin(_source: &str, _target_dir: &std::path::Path) -> Result<(), PluginError> {
    Err(PluginError::NotFound("Plugin install not supported".to_string()))
}
