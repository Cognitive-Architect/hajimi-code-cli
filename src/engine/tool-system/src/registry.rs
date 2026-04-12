//! Tool Registry - Week 4

use std::collections::HashMap;
use std::sync::Arc;

use super::Tool;

pub struct ToolRegistry {
    tools: HashMap<String, Arc<dyn Tool>>,
}

impl Default for ToolRegistry {
    fn default() -> Self { Self::new() }
}

impl ToolRegistry {
    pub fn new() -> Self { Self { tools: HashMap::new() } }
    pub fn register(&mut self, tool: Arc<dyn Tool>) { self.tools.insert(tool.name().to_string(), tool); }
    pub fn get(&self, name: &str) -> Option<Arc<dyn Tool>> { self.tools.get(name).cloned() }
    pub fn list(&self) -> Vec<&str> { self.tools.keys().map(|s| s.as_str()).collect() }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tool::{Config, PermissionLevel, ToolArgs, ToolError, ToolOutput, ToolPermissions};
    use async_trait::async_trait;

    struct MockTool;

    #[async_trait]
    impl Tool for MockTool {
        fn name(&self) -> &str { "mock" }
        fn description(&self) -> &str { "Mock tool" }
        fn permissions(&self) -> ToolPermissions { ToolPermissions::default() }
        async fn execute(&self, _args: ToolArgs) -> Result<ToolOutput, ToolError> {
            Ok(ToolOutput { stdout: "ok".to_string(), stderr: "".to_string(), exit_code: Some(0) })
        }
    }

    #[test]
    fn test_registry() {
        let mut registry = ToolRegistry::new();
        registry.register(Arc::new(MockTool));
        assert_eq!(registry.list(), vec!["mock"]);
        assert!(registry.get("mock").is_some());
    }
}
