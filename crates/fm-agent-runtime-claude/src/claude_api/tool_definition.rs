//! Tool definition for the Claude Messages API `tools` parameter.

use serde::{Deserialize, Serialize};
use typed_builder::TypedBuilder;

/// A tool definition passed to Claude via the `tools` parameter.
///
/// Claude uses this to understand what tools are available and their input schemas.
#[derive(Debug, Clone, Serialize, Deserialize, TypedBuilder)]
pub struct ToolDefinition {
    /// Tool name (e.g. `create_agent`).
    pub name: String,
    /// Human-readable description of what the tool does.
    #[builder(default, setter(strip_option))]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    /// JSON Schema describing the tool's input parameters.
    pub input_schema: serde_json::Value,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn serializes_with_all_fields() {
        let tool = ToolDefinition::builder()
            .name("create_agent".to_string())
            .description("Create a child agent".to_string())
            .input_schema(serde_json::json!({
                "type": "object",
                "properties": {
                    "agent_type": { "type": "string" }
                },
                "required": ["agent_type"]
            }))
            .build();

        let json: serde_json::Value = serde_json::to_value(&tool).expect("serialize");
        assert_eq!(json["name"], "create_agent");
        assert_eq!(json["description"], "Create a child agent");
        assert_eq!(json["input_schema"]["type"], "object");
    }

    #[test]
    fn serializes_without_description() {
        let tool = ToolDefinition::builder()
            .name("list_agents".to_string())
            .input_schema(serde_json::json!({"type": "object"}))
            .build();

        let json: serde_json::Value = serde_json::to_value(&tool).expect("serialize");
        assert_eq!(json["name"], "list_agents");
        assert!(json.get("description").is_none());
    }

    #[test]
    fn deserializes() {
        let json = r#"{
            "name": "get_status",
            "description": "Get agent status",
            "input_schema": {"type": "object"}
        }"#;

        let tool: ToolDefinition = serde_json::from_str(json).expect("deserialize");
        assert_eq!(tool.name, "get_status");
        assert_eq!(tool.description.as_deref(), Some("Get agent status"));
    }

    #[test]
    fn is_cloneable() {
        let tool = ToolDefinition::builder()
            .name("test".to_string())
            .input_schema(serde_json::json!({}))
            .build();
        let cloned = tool.clone();
        assert_eq!(cloned.name, "test");
    }
}
