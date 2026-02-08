//! Claude Messages API request types.

use serde::Serialize;
use typed_builder::TypedBuilder;

use super::content_block::RequestContentBlock;
use super::tool_definition::ToolDefinition;

/// Who sent a message in the Claude Messages API conversation.
///
/// The API requires alternating `User` / `Assistant` turns.
/// Not related to the agent's persona — that's in the `system` prompt.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, serde::Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum MessageRole {
    /// Caller's turn (prompt text or `tool_result` responses).
    User,
    /// Claude's turn (text responses or `tool_use` requests).
    Assistant,
}

/// Message content — either plain text or structured content blocks.
///
/// Plain text serializes as a JSON string. Blocks serialize as a JSON array.
/// This matches the Claude Messages API which accepts both formats.
#[derive(Debug, Clone, Serialize, serde::Deserialize)]
#[serde(untagged)]
pub enum MessageContent {
    /// Simple text content (serialized as a plain string).
    Text(String),
    /// Array of content blocks (for tool_result and tool_use messages).
    Blocks(Vec<RequestContentBlock>),
}

/// A single message in the conversation.
#[derive(Debug, Clone, Serialize, serde::Deserialize)]
pub struct Message {
    /// Who sent this message.
    pub role: MessageRole,
    /// Message content (text or structured blocks).
    pub content: MessageContent,
}

impl Message {
    /// Creates a new user message with text content.
    pub fn user(content: impl Into<String>) -> Self {
        Self {
            role: MessageRole::User,
            content: MessageContent::Text(content.into()),
        }
    }

    /// Creates a new assistant message with text content.
    pub fn assistant(content: impl Into<String>) -> Self {
        Self {
            role: MessageRole::Assistant,
            content: MessageContent::Text(content.into()),
        }
    }

    /// Creates an assistant message with structured content blocks.
    ///
    /// Used to echo back tool_use blocks when sending tool results.
    pub fn assistant_blocks(blocks: Vec<RequestContentBlock>) -> Self {
        Self {
            role: MessageRole::Assistant,
            content: MessageContent::Blocks(blocks),
        }
    }

    /// Creates a user message with tool result blocks.
    pub fn tool_results(results: Vec<RequestContentBlock>) -> Self {
        Self {
            role: MessageRole::User,
            content: MessageContent::Blocks(results),
        }
    }
}

/// Request body for the Claude Messages API.
#[derive(Debug, Serialize, TypedBuilder)]
pub struct MessagesRequest {
    /// Model identifier (e.g. `claude-sonnet-4-20250514`).
    pub model: String,
    /// Maximum tokens to generate.
    pub max_tokens: i32,
    /// Conversation messages.
    #[builder(default)]
    pub messages: Vec<Message>,
    /// System prompt.
    #[builder(default, setter(strip_option))]
    pub system: Option<String>,
    /// Whether to stream the response via SSE.
    #[builder(default = true)]
    pub stream: bool,
    /// Sampling temperature (0.0-1.0).
    #[builder(default, setter(strip_option))]
    pub temperature: Option<f64>,
    /// Tool definitions available to the model.
    #[builder(default)]
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub tools: Vec<ToolDefinition>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn role_serializes_lowercase() {
        let json = serde_json::to_string(&MessageRole::User).expect("serialize role");
        assert_eq!(json, "\"user\"");

        let json = serde_json::to_string(&MessageRole::Assistant).expect("serialize role");
        assert_eq!(json, "\"assistant\"");
    }

    #[test]
    fn message_user_factory() {
        let msg = Message::user("hello");
        assert_eq!(msg.role, MessageRole::User);
        assert!(matches!(msg.content, MessageContent::Text(ref t) if t == "hello"));
    }

    #[test]
    fn message_assistant_factory() {
        let msg = Message::assistant("hi there");
        assert_eq!(msg.role, MessageRole::Assistant);
        assert!(matches!(msg.content, MessageContent::Text(ref t) if t == "hi there"));
    }

    #[test]
    fn message_text_serializes_as_string() {
        let msg = Message::user("test");
        let json: serde_json::Value = serde_json::to_value(&msg).expect("serialize message");
        assert_eq!(json["role"], "user");
        assert_eq!(json["content"], "test");
    }

    #[test]
    fn message_tool_results_serializes_as_array() {
        let msg = Message::tool_results(vec![RequestContentBlock::ToolResult {
            tool_use_id: "toolu_123".to_string(),
            content: "Agent created".to_string(),
            is_error: None,
        }]);
        let json: serde_json::Value = serde_json::to_value(&msg).expect("serialize");
        assert_eq!(json["role"], "user");
        assert!(json["content"].is_array());
        assert_eq!(json["content"][0]["type"], "tool_result");
        assert_eq!(json["content"][0]["tool_use_id"], "toolu_123");
    }

    #[test]
    fn message_assistant_blocks_serializes() {
        let msg = Message::assistant_blocks(vec![
            RequestContentBlock::Text {
                text: "I'll create an agent".to_string(),
            },
            RequestContentBlock::ToolUse {
                id: "toolu_456".to_string(),
                name: "create_agent".to_string(),
                input: serde_json::json!({"agent_type": "code-generator"}),
            },
        ]);
        let json: serde_json::Value = serde_json::to_value(&msg).expect("serialize");
        assert_eq!(json["role"], "assistant");
        assert!(json["content"].is_array());
        assert_eq!(json["content"][0]["type"], "text");
        assert_eq!(json["content"][1]["type"], "tool_use");
    }

    #[test]
    fn request_serializes_with_defaults() {
        let request = MessagesRequest::builder()
            .model("claude-sonnet-4-20250514".to_string())
            .max_tokens(4096)
            .build();

        let json: serde_json::Value = serde_json::to_value(&request).expect("serialize request");
        assert_eq!(json["model"], "claude-sonnet-4-20250514");
        assert_eq!(json["max_tokens"], 4096);
        assert_eq!(json["stream"], true);
        assert!(json["messages"].as_array().expect("messages array").is_empty());
        assert!(json.get("system").is_none() || json["system"].is_null());
        assert!(json.get("temperature").is_none() || json["temperature"].is_null());
        // tools should be omitted when empty
        assert!(json.get("tools").is_none());
    }

    #[test]
    fn request_serializes_with_all_fields() {
        let request = MessagesRequest::builder()
            .model("claude-opus-4-20250514".to_string())
            .max_tokens(1024)
            .messages(vec![Message::user("Build an API")])
            .system("You are a code generator.".to_string())
            .stream(true)
            .temperature(0.7)
            .build();

        let json: serde_json::Value = serde_json::to_value(&request).expect("serialize request");
        assert_eq!(json["model"], "claude-opus-4-20250514");
        assert_eq!(json["max_tokens"], 1024);
        assert_eq!(json["stream"], true);
        assert_eq!(json["system"], "You are a code generator.");
        assert_eq!(json["temperature"], 0.7);
        assert_eq!(json["messages"][0]["role"], "user");
        assert_eq!(json["messages"][0]["content"], "Build an API");
    }

    #[test]
    fn request_serializes_with_tools() {
        let request = MessagesRequest::builder()
            .model("test".to_string())
            .max_tokens(100)
            .tools(vec![ToolDefinition::builder()
                .name("create_agent".to_string())
                .description("Create a child agent".to_string())
                .input_schema(serde_json::json!({"type": "object"}))
                .build()])
            .build();

        let json: serde_json::Value = serde_json::to_value(&request).expect("serialize");
        assert_eq!(json["tools"][0]["name"], "create_agent");
        assert_eq!(json["tools"][0]["description"], "Create a child agent");
    }

    #[test]
    fn request_stream_defaults_true() {
        let request = MessagesRequest::builder()
            .model("test".to_string())
            .max_tokens(100)
            .build();
        assert!(request.stream);
    }
}
