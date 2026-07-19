use std::collections::HashMap;
use crate::db::Db;
use crate::plugin::PluginManager;

pub struct MarketClient;

impl MarketClient {
    pub fn new(_pm: std::sync::Arc<PluginManager>, _db: Db, _sources: Vec<String>) -> Self {
        Self
    }
    pub fn source_config(_config: &HashMap<String, String>) -> Vec<String> {
        Vec::new()
    }
}
