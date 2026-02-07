//! Claude Messages API response types.

use serde::Deserialize;

/// Stop reason for a completed message.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum StopReason {
    /// Model finished naturally.
    EndTurn,
    /// Hit the max tokens limit.
    MaxTokens,
    /// Model wants to use a tool.
    ToolUse,
}

/// Token usage statistics.
#[derive(Debug, Clone, Copy, Default, Deserialize, PartialEq, Eq)]
pub struct Usage {
    /// Number of input tokens consumed.
    #[serde(default)]
    pub input_tokens: i64,
    /// Number of output tokens generated.
    #[serde(default)]
    pub output_tokens: i64,
}

impl Usage {
    /// Total tokens (input + output).
    pub fn total(&self) -> i64 {
        self.input_tokens + self.output_tokens
    }
}

/// A content block in the response.
#[derive(Debug, Deserialize)]
pub struct ContentBlock {
    /// Block type (e.g. "text", "tool_use").
    #[serde(rename = "type")]
    pub block_type: String,
    /// Text content (present for "text" blocks).
    pub text: Option<String>,
}

/// Full (non-streaming) response from the Messages API.
#[derive(Debug, Deserialize)]
pub struct MessagesResponse {
    /// Unique message ID.
    pub id: String,
    /// Content blocks.
    pub content: Vec<ContentBlock>,
    /// Why the model stopped generating.
    pub stop_reason: Option<StopReason>,
    /// Token usage.
    pub usage: Usage,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn usage_default_is_zero() {
        let usage = Usage::default();
        assert_eq!(usage.input_tokens, 0);
        assert_eq!(usage.output_tokens, 0);
        assert_eq!(usage.total(), 0);
    }

    #[test]
    fn usage_total() {
        let usage = Usage {
            input_tokens: 100,
            output_tokens: 50,
        };
        assert_eq!(usage.total(), 150);
    }

    #[test]
    fn stop_reason_deserializes() {
        let json = "\"end_turn\"";
        let reason: StopReason = serde_json::from_str(json).expect("deserialize stop_reason");
        assert_eq!(reason, StopReason::EndTurn);

        let json = "\"max_tokens\"";
        let reason: StopReason = serde_json::from_str(json).expect("deserialize stop_reason");
        assert_eq!(reason, StopReason::MaxTokens);

        let json = "\"tool_use\"";
        let reason: StopReason = serde_json::from_str(json).expect("deserialize stop_reason");
        assert_eq!(reason, StopReason::ToolUse);
    }

    #[test]
    fn content_block_text_deserializes() {
        let json = r#"{"type":"text","text":"Hello, world!"}"#;
        let block: ContentBlock = serde_json::from_str(json).expect("deserialize content block");
        assert_eq!(block.block_type, "text");
        assert_eq!(block.text.as_deref(), Some("Hello, world!"));
    }

    #[test]
    fn content_block_without_text() {
        let json = r#"{"type":"tool_use","id":"toolu_123","name":"get_weather","input":{}}"#;
        let block: ContentBlock = serde_json::from_str(json).expect("deserialize content block");
        assert_eq!(block.block_type, "tool_use");
        assert!(block.text.is_none());
    }

    #[test]
    fn full_response_deserializes() {
        let json = r#"{
            "id": "msg_01XFDUDYJgAACzvnptvVoYEL",
            "type": "message",
            "role": "assistant",
            "content": [
                {"type": "text", "text": "Hello!"}
            ],
            "model": "claude-sonnet-4-20250514",
            "stop_reason": "end_turn",
            "usage": {
                "input_tokens": 12,
                "output_tokens": 6
            }
        }"#;

        let response: MessagesResponse =
            serde_json::from_str(json).expect("deserialize response");
        assert_eq!(response.id, "msg_01XFDUDYJgAACzvnptvVoYEL");
        assert_eq!(response.content.len(), 1);
        assert_eq!(response.content[0].text.as_deref(), Some("Hello!"));
        assert_eq!(response.stop_reason, Some(StopReason::EndTurn));
        assert_eq!(response.usage.input_tokens, 12);
        assert_eq!(response.usage.output_tokens, 6);
    }

    #[test]
    fn usage_deserializes_with_missing_fields() {
        let json = r#"{}"#;
        let usage: Usage = serde_json::from_str(json).expect("deserialize empty usage");
        assert_eq!(usage.input_tokens, 0);
        assert_eq!(usage.output_tokens, 0);
    }
}
