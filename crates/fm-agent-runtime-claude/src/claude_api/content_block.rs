//! Request-side content blocks for the Claude Messages API.
//!
//! Used when sending tool results back to Claude in a multi-turn conversation.

use serde::{Deserialize, Serialize};

/// A content block in a request message.
///
/// Claude Messages API accepts `content` as either a plain string or
/// an array of typed content blocks. This enum represents the block types
/// needed for tool calling.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum RequestContentBlock {
    /// Plain text content.
    #[serde(rename = "text")]
    Text {
        /// The text content.
        text: String,
    },
    /// Result of a tool execution.
    #[serde(rename = "tool_result")]
    ToolResult {
        /// ID of the tool_use block this result corresponds to.
        tool_use_id: String,
        /// Text content of the result.
        content: String,
        /// Whether the tool execution resulted in an error.
        #[serde(skip_serializing_if = "Option::is_none")]
        is_error: Option<bool>,
    },
    /// A tool use block (for echoing back in assistant messages).
    #[serde(rename = "tool_use")]
    ToolUse {
        /// Unique ID for this tool use.
        id: String,
        /// Tool name.
        name: String,
        /// Tool input arguments.
        input: serde_json::Value,
    },
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn text_block_serializes() {
        let block = RequestContentBlock::Text {
            text: "Hello".to_string(),
        };
        let json: serde_json::Value = serde_json::to_value(&block).expect("serialize");
        assert_eq!(json["type"], "text");
        assert_eq!(json["text"], "Hello");
    }

    #[test]
    fn tool_result_serializes() {
        let block = RequestContentBlock::ToolResult {
            tool_use_id: "toolu_123".to_string(),
            content: "Agent created".to_string(),
            is_error: None,
        };
        let json: serde_json::Value = serde_json::to_value(&block).expect("serialize");
        assert_eq!(json["type"], "tool_result");
        assert_eq!(json["tool_use_id"], "toolu_123");
        assert_eq!(json["content"], "Agent created");
        assert!(json.get("is_error").is_none());
    }

    #[test]
    fn tool_result_with_error_serializes() {
        let block = RequestContentBlock::ToolResult {
            tool_use_id: "toolu_456".to_string(),
            content: "Connection refused".to_string(),
            is_error: Some(true),
        };
        let json: serde_json::Value = serde_json::to_value(&block).expect("serialize");
        assert_eq!(json["type"], "tool_result");
        assert_eq!(json["is_error"], true);
    }

    #[test]
    fn tool_use_block_serializes() {
        let block = RequestContentBlock::ToolUse {
            id: "toolu_789".to_string(),
            name: "create_agent".to_string(),
            input: serde_json::json!({"agent_type": "code-generator"}),
        };
        let json: serde_json::Value = serde_json::to_value(&block).expect("serialize");
        assert_eq!(json["type"], "tool_use");
        assert_eq!(json["id"], "toolu_789");
        assert_eq!(json["name"], "create_agent");
        assert_eq!(json["input"]["agent_type"], "code-generator");
    }

    #[test]
    fn tool_result_deserializes() {
        let json = r#"{"type":"tool_result","tool_use_id":"toolu_123","content":"ok"}"#;
        let block: RequestContentBlock = serde_json::from_str(json).expect("deserialize");
        assert!(matches!(block, RequestContentBlock::ToolResult { .. }));
    }
}
