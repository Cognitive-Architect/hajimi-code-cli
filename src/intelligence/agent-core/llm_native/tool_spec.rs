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

                let tool_name = tool.name();
                let (namespace, display_name) = if tool_name.contains("__") {
                    let parts: Vec<&str> = tool_name.splitn(2, "__").collect();
                    (Some(parts[0].to_string()), parts[1].to_string())
                } else {
                    (None, tool_name.to_string())
                };

                let schema =
                    Self::normalize_parameters_schema(Self::get_tool_schema(&display_name));

                specs.push(ModelVisibleToolSpec {
                    name: display_name,
                    namespace,
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

    /// Returns the default fallback schema for tools following DeepSeek / OpenAI specs.
    pub fn default_object_schema() -> Value {
        serde_json::json!({
            "type": "object",
            "properties": {},
            "additionalProperties": true
        })
    }

    /// Normalizes a parameter schema value to ensure it meets DeepSeek expectations.
    ///
    /// It forces a top-level `type: "object"`, guarantees a `properties` object, and handles
    /// graceful fallbacks for missing or malformed schemas.
    pub fn normalize_parameters_schema(schema: Value) -> Value {
        if let Value::Object(mut map) = schema {
            if map.is_empty() {
                return Self::default_object_schema();
            }
            let type_val = map.get("type");
            match type_val {
                Some(Value::String(s)) if s == "object" => {
                    // Valid object type
                }
                None => {
                    map.insert("type".to_string(), Value::String("object".to_string()));
                }
                _ => {
                    return Self::default_object_schema();
                }
            }
            if !map.get("properties").is_some_and(|v| v.is_object()) {
                map.insert(
                    "properties".to_string(),
                    Value::Object(serde_json::Map::new()),
                );
            }
            Value::Object(map)
        } else {
            Self::default_object_schema()
        }
    }

    /// Safely parse a raw JSON string into a serde_json::Value.
    /// In case of syntax issues, it yields a safe default empty object.
    pub fn parse_schema_safely(raw_json: &str) -> Value {
        let val = serde_json::from_str(raw_json).unwrap_or_else(|_| serde_json::json!({}));
        Self::normalize_parameters_schema(val)
    }

    /// Dynamically or statically resolve schema based on tool name.
    /// If no static schema matches, yields a default empty object.
    fn get_tool_schema(name: &str) -> Value {
        match name {
            "web_search" => serde_json::json!({
                "type": "object",
                "properties": {
                    "query": {
                        "type": "string",
                        "description": "Search query"
                    }
                },
                "required": ["query"],
                "additionalProperties": false
            }),
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
        assert_eq!(
            specs[0].parameters_schema,
            ToolSpecExporter::default_object_schema()
        );
    }

    #[test]
    fn test_invalid_json_schema() {
        let malformed = "{invalid json}";
        let parsed = ToolSpecExporter::parse_schema_safely(malformed);
        assert_eq!(parsed, ToolSpecExporter::default_object_schema());

        let valid = "{\"type\": \"object\", \"properties\": {\"key\": {\"type\": \"string\"}}}";
        let parsed_valid = ToolSpecExporter::parse_schema_safely(valid);
        assert_eq!(
            parsed_valid.get("type").and_then(|v| v.as_str()),
            Some("object")
        );
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

    #[test]
    fn test_exporter_namespace_isolation() {
        let mut registry = ToolRegistry::new();
        registry.register(Arc::new(DummyTool {
            name: "mcp__github__create_issue".to_string(),
            description: "Create github issue".to_string(),
            permissions: ToolPermissions::default(),
        }));

        let specs = ToolSpecExporter::from_registry(&registry);
        assert_eq!(specs.len(), 1);
        assert_eq!(specs[0].name, "github__create_issue");
        assert_eq!(specs[0].namespace, Some("mcp".to_string()));
    }

    #[test]
    fn test_exporter_non_empty_schema_assertion() {
        let mut registry = ToolRegistry::new();
        registry.register(Arc::new(DummyTool {
            name: "read_file".to_string(),
            description: "read file content".to_string(),
            permissions: ToolPermissions::default(),
        }));

        let specs = ToolSpecExporter::from_registry(&registry);
        assert_eq!(specs.len(), 1);
        assert_eq!(specs[0].name, "read_file");
        assert!(specs[0].parameters_schema.get("properties").is_some());
    }

    #[test]
    fn test_web_search_schema_is_deepseek_compatible_object() {
        let schema = ToolSpecExporter::get_tool_schema("web_search");
        let normalized = ToolSpecExporter::normalize_parameters_schema(schema);

        assert_eq!(
            normalized.get("type").and_then(|v| v.as_str()),
            Some("object")
        );
        let properties = normalized.get("properties").expect("properties missing");
        let query = properties.get("query").expect("query missing");
        assert_eq!(query.get("type").and_then(|v| v.as_str()), Some("string"));

        let required = normalized
            .get("required")
            .and_then(|v| v.as_array())
            .expect("required not array");
        assert!(required.contains(&serde_json::json!("query")));
        assert_eq!(
            normalized
                .get("additionalProperties")
                .and_then(|v| v.as_bool()),
            Some(false)
        );
    }

    #[test]
    fn test_unknown_tool_schema_normalizes_to_object() {
        let mut registry = ToolRegistry::new();
        registry.register(Arc::new(DummyTool {
            name: "unknown_dummy_tool".to_string(),
            description: "Desc".to_string(),
            permissions: ToolPermissions::default(),
        }));

        let specs = ToolSpecExporter::from_registry(&registry);
        assert_eq!(specs.len(), 1);
        let schema = &specs[0].parameters_schema;
        assert_eq!(schema.get("type").and_then(|v| v.as_str()), Some("object"));
        assert!(schema
            .get("properties")
            .and_then(|v| v.as_object())
            .is_some());
        assert_eq!(
            schema.get("additionalProperties").and_then(|v| v.as_bool()),
            Some(true)
        );
    }

    #[test]
    fn test_parse_schema_safely_normalizes_malformed_json() {
        let malformed = "{invalid json}";
        let parsed = ToolSpecExporter::parse_schema_safely(malformed);
        assert_eq!(parsed.get("type").and_then(|v| v.as_str()), Some("object"));
        assert!(parsed
            .get("properties")
            .and_then(|v| v.as_object())
            .is_some());
        assert_eq!(
            parsed.get("additionalProperties").and_then(|v| v.as_bool()),
            Some(true)
        );
    }

    #[test]
    fn test_exported_registry_schemas_are_all_objects() {
        let mut registry = ToolRegistry::new();
        registry.register(Arc::new(DummyTool {
            name: "web_search".to_string(),
            description: "desc".to_string(),
            permissions: ToolPermissions::default(),
        }));
        registry.register(Arc::new(DummyTool {
            name: "read_file".to_string(),
            description: "desc".to_string(),
            permissions: ToolPermissions::default(),
        }));
        registry.register(Arc::new(DummyTool {
            name: "write_file".to_string(),
            description: "desc".to_string(),
            permissions: ToolPermissions::default(),
        }));
        registry.register(Arc::new(DummyTool {
            name: "unknown_dummy_tool".to_string(),
            description: "desc".to_string(),
            permissions: ToolPermissions::default(),
        }));

        let specs = ToolSpecExporter::from_registry(&registry);
        assert_eq!(specs.len(), 4);
        for spec in specs {
            assert_eq!(
                spec.parameters_schema.get("type").and_then(|v| v.as_str()),
                Some("object"),
                "tool {} exported non-object schema: {}",
                spec.name,
                spec.parameters_schema
            );
        }
    }

    #[test]
    fn test_existing_read_file_schema_preserved() {
        let schema = ToolSpecExporter::get_tool_schema("read_file");
        let normalized = ToolSpecExporter::normalize_parameters_schema(schema);
        assert_eq!(
            normalized.get("type").and_then(|v| v.as_str()),
            Some("object")
        );
        let properties = normalized
            .get("properties")
            .and_then(|v| v.as_object())
            .unwrap();
        assert!(properties.contains_key("path"));
        let required = normalized
            .get("required")
            .and_then(|v| v.as_array())
            .unwrap();
        assert!(required.contains(&serde_json::json!("path")));
    }
}
