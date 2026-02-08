//! Tool use content block from Claude's response.

use serde::{Deserialize, Serialize};

/// A `tool_use` content block from Claude's streaming response.
///
/// Built from `ToolUseStart` (id + name) and accumulated `InputJsonDelta` events.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolUseBlock {
    /// Unique ID for this tool use (e.g. `toolu_01A09q90qw90lq917835lq9`).
    pub id: String,
    /// Tool name (e.g. `create_agent`).
    pub name: String,
    /// Tool input arguments as JSON.
    pub input: serde_json::Value,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn serializes() {
        let block = ToolUseBlock {
            id: "toolu_123".to_string(),
            name: "create_agent".to_string(),
            input: serde_json::json!({"agent_type": "code-generator"}),
        };
        let json: serde_json::Value = serde_json::to_value(&block).expect("serialize");
        assert_eq!(json["id"], "toolu_123");
        assert_eq!(json["name"], "create_agent");
        assert_eq!(json["input"]["agent_type"], "code-generator");
    }

    #[test]
    fn deserializes() {
        let json = r#"{"id":"toolu_456","name":"list_agents","input":{}}"#;
        let block: ToolUseBlock = serde_json::from_str(json).expect("deserialize");
        assert_eq!(block.id, "toolu_456");
        assert_eq!(block.name, "list_agents");
    }

    #[test]
    fn is_cloneable() {
        let block = ToolUseBlock {
            id: "toolu_789".to_string(),
            name: "get_status".to_string(),
            input: serde_json::json!({}),
        };
        let cloned = block.clone();
        assert_eq!(cloned.id, block.id);
        assert_eq!(cloned.name, block.name);
    }
}
