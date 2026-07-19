use crate::plugin::PluginError;

pub trait ScriptRuntime: Send + Sync {
    fn execute(&self, script: &str, context: &str) -> Result<String, PluginError>;
}
