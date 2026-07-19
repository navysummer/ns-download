use std::collections::HashMap;

pub struct DependencyResolver;

impl DependencyResolver {
    pub fn new() -> Self { Self }
    pub fn resolve(&self, _deps: &HashMap<String, String>) -> Result<Vec<String>, String> {
        Ok(Vec::new())
    }
}
