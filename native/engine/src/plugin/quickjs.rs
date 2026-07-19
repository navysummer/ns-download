use crate::downloader::DownloadError;
use crate::plugin::{PluginError, ScriptRuntime};

pub struct QuickJsScriptRuntime;

impl QuickJsScriptRuntime {
    pub fn new(_max_concurrency: usize) -> Result<Self, DownloadError> {
        Ok(Self)
    }
}

impl ScriptRuntime for QuickJsScriptRuntime {
    fn execute(&self, _script: &str, _context: &str) -> Result<String, PluginError> {
        Err(PluginError::Runtime("QuickJS not compiled in".to_string()))
    }
}
