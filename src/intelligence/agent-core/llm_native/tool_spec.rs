//! Model-visible tool specifications and exporter.
//!
//! These types describe tools in a form the LLM can understand and decide to call
//! (equivalent to Codex `ToolSpec` + `tool_choice: "auto"`).
//!
//! The `ToolSpecExporter` is responsible for turning the runtime `ToolRegistry`
//! into a list of `ModelVisibleToolSpec` that can be sent to the model.

use serde_json::Value;

/// Risk classification for a tool (used for governance and prompting).
#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum RiskLevel {
    Low,
    Medium,
    High,
    Critical,
}

/// A tool specification that is safe and useful to expose to the LLM.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct ModelVisibleToolSpec {
    /// Primary tool name (e.g. "read_file", "write_file").
    pub name: String,

    /// Optional namespace (for MCP tools, skills, etc.).
    pub namespace: Option<String>,

    /// Human-readable description for the model.
    pub description: String,

    /// JSON Schema for the tool's parameters (following OpenAI/Anthropic conventions).
    pub parameters_schema: Value,

    /// Whether the tool supports parallel / concurrent calls in one turn.
    pub supports_parallel: bool,

    /// Risk level for governance decisions.
    pub risk_level: RiskLevel,
}

/// Exports runtime tool information into model-visible specifications.
pub struct ToolSpecExporter;

impl ToolSpecExporter {
    /// Build model-visible specs from a ToolRegistry.
    ///
    /// This method traverses all registered tools in the registry, extracts their
    /// metadata (name, description, parameters schema), and maps their risk levels.
    ///
    /// # Example
    /// ```
    /// # use engine_tool_system::ToolRegistry;
    /// # use agent_core::llm_native::ToolSpecExporter;
    /// let registry = ToolRegistry::default();
    /// let specs = ToolSpecExporter::from_registry(&registry);
    /// ```
    pub fn from_registry(registry: &engine_tool_system::ToolRegistry) -> Vec<ModelVisibleToolSpec> {
        let mut specs = Vec::new();
        for name in registry.list() {
            if let Some(tool) = registry.get(name) {
                let permissions = tool.permissions();
                let risk_level = match permissions.default_level {
                    engine_tool_system::PermissionLevel::Allow => {
                        if permissions.requires_confirmation {
                            RiskLevel::Medium
                        } else {
                            RiskLevel::Low
                        }
                    }
                    engine_tool_system::PermissionLevel::Ask => {
                        if permissions.requires_confirmation {
                            RiskLevel::High
                        } else {
                            RiskLevel::Medium
                        }
                    }
                    engine_tool_system::PermissionLevel::Deny => RiskLevel::Critical,
                };

                let schema = Self::get_tool_schema(tool.name());

                specs.push(ModelVisibleToolSpec {
                    name: tool.name().to_string(),
                    namespace: None,
                    description: tool.description().to_string(),
                    parameters_schema: schema,
                    supports_parallel: true,
                    risk_level,
                });
            }
        }
        specs
    }

    /// Convenience: build from an optional registry reference.
    pub fn from_registry_optional(
        registry: Option<&engine_tool_system::ToolRegistry>,
    ) -> Vec<ModelVisibleToolSpec> {
        match registry {
            Some(r) => Self::from_registry(r),
            None => Vec::new(),
        }
    }

    /// Safely parse a raw JSON string into a serde_json::Value.
    /// In case of syntax issues, it yields a safe default empty object.
    pub fn parse_schema_safely(raw_json: &str) -> Value {
        serde_json::from_str(raw_json).unwrap_or_else(|_| serde_json::json!({}))
    }

    /// Dynamically or statically resolve schema based on tool name.
    /// If no static schema matches, yields a default empty object.
    fn get_tool_schema(name: &str) -> Value {
        match name {
            "read_file" => serde_json::json!({
                "type": "object",
                "properties": {
                    "path": {
                        "type": "string",
                        "description": "Absolute path to the file to read"
                    }
                },
                "required": ["path"]
            }),
            "write_file" => serde_json::json!({
                "type": "object",
                "properties": {
                    "path": {
                        "type": "string",
                        "description": "Absolute path to create or write the file"
                    },
                    "content": {
                        "type": "string",
                        "description": "Complete text content to write"
                    }
                },
                "required": ["path", "content"]
            }),
            "edit_file" => serde_json::json!({
                "type": "object",
                "properties": {
                    "path": {
                        "type": "string",
                        "description": "Absolute path to the file to edit"
                    },
                    "target_content": {
                        "type": "string",
                        "description": "Exact text block to exchange"
                    },
                    "replacement_content": {
                        "type": "string",
                        "description": "New content text block"
                    }
                },
                "required": ["path", "target_content", "replacement_content"]
            }),
            "list_dir" | "list_directory" => serde_json::json!({
                "type": "object",
                "properties": {
                    "path": {
                        "type": "string",
                        "description": "Absolute path to the directory"
                    }
                },
                "required": ["path"]
            }),
            "grep_search" | "grep" => serde_json::json!({
                "type": "object",
                "properties": {
                    "query": {
                        "type": "string",
                        "description": "Search pattern query string"
                    },
                    "path": {
                        "type": "string",
                        "description": "Absolute path to search within"
                    }
                },
                "required": ["query", "path"]
            }),
            _ => serde_json::json!({}),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use async_trait::async_trait;
    use engine_tool_system::{
        PermissionLevel, Tool, ToolArgs, ToolError, ToolOutput, ToolPermissions, ToolRegistry,
    };
    use std::sync::Arc;

    struct DummyTool {
        name: String,
        description: String,
        permissions: ToolPermissions,
    }

    #[async_trait]
    impl Tool for DummyTool {
        fn name(&self) -> &str {
            &self.name
        }
        fn description(&self) -> &str {
            &self.description
        }
        fn permissions(&self) -> ToolPermissions {
            self.permissions.clone()
        }
        async fn execute(&self, _args: ToolArgs) -> Result<ToolOutput, ToolError> {
            Ok(ToolOutput::success("dummy"))
        }
    }

    #[test]
    fn test_exporter_from_registry_non_empty() {
        let mut registry = ToolRegistry::new();
        registry.register(Arc::new(DummyTool {
            name: "test_tool".to_string(),
            description: "A dummy test tool".to_string(),
            permissions: ToolPermissions::default(),
        }));

        let specs = ToolSpecExporter::from_registry(&registry);
        assert!(!specs.is_empty(), "Specs should not be empty");
        assert_eq!(specs[0].name, "test_tool");
        assert_eq!(specs[0].description, "A dummy test tool");
        assert_eq!(specs[0].risk_level, RiskLevel::Medium); // default matches PermissionLevel::Ask
    }

    #[test]
    fn test_empty_registry() {
        let registry = ToolRegistry::new();
        let specs = ToolSpecExporter::from_registry(&registry);
        assert!(specs.is_empty(), "Empty registry should return empty specs");
    }

    #[test]
    fn test_missing_schema() {
        let mut registry = ToolRegistry::new();
        registry.register(Arc::new(DummyTool {
            name: "unknown_dummy_tool".to_string(),
            description: "Desc".to_string(),
            permissions: ToolPermissions::default(),
        }));

        let specs = ToolSpecExporter::from_registry(&registry);
        assert_eq!(specs.len(), 1);
        assert_eq!(specs[0].parameters_schema, serde_json::json!({}));
    }

    #[test]
    fn test_invalid_json_schema() {
        let malformed = "{invalid json}";
        let parsed = ToolSpecExporter::parse_schema_safely(malformed);
        assert_eq!(parsed, serde_json::json!({}));

        let valid = "{\"key\": \"val\"}";
        let parsed_valid = ToolSpecExporter::parse_schema_safely(valid);
        assert_eq!(parsed_valid, serde_json::json!({"key": "val"}));
    }

    #[test]
    fn test_risk_level_mapping() {
        let test_cases = vec![
            (PermissionLevel::Allow, false, RiskLevel::Low),
            (PermissionLevel::Allow, true, RiskLevel::Medium),
            (PermissionLevel::Ask, false, RiskLevel::Medium),
            (PermissionLevel::Ask, true, RiskLevel::High),
            (PermissionLevel::Deny, false, RiskLevel::Critical),
            (PermissionLevel::Deny, true, RiskLevel::Critical),
        ];

        for (level, requires_confirm, expected) in test_cases {
            let mut registry = ToolRegistry::new();
            registry.register(Arc::new(DummyTool {
                name: "test_risk".to_string(),
                description: "desc".to_string(),
                permissions: ToolPermissions {
                    default_level: level,
                    requires_confirmation: requires_confirm,
                    allowed_paths: None,
                },
            }));

            let specs = ToolSpecExporter::from_registry(&registry);
            assert_eq!(specs[0].risk_level, expected);
        }
    }
}
