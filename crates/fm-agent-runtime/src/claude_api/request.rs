//! Claude Messages API request types.

use serde::Serialize;
use typed_builder::TypedBuilder;

/// Role in a conversation message.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, serde::Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Role {
    /// Human turn.
    User,
    /// Model turn.
    Assistant,
}

/// A single message in the conversation.
#[derive(Debug, Clone, Serialize, serde::Deserialize)]
pub struct Message {
    /// Who sent this message.
    pub role: Role,
    /// Text content of the message.
    pub content: String,
}

impl Message {
    /// Creates a new user message.
    pub fn user(content: impl Into<String>) -> Self {
        Self {
            role: Role::User,
            content: content.into(),
        }
    }

    /// Creates a new assistant message.
    pub fn assistant(content: impl Into<String>) -> Self {
        Self {
            role: Role::Assistant,
            content: content.into(),
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
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn role_serializes_lowercase() {
        let json = serde_json::to_string(&Role::User).expect("serialize role");
        assert_eq!(json, "\"user\"");

        let json = serde_json::to_string(&Role::Assistant).expect("serialize role");
        assert_eq!(json, "\"assistant\"");
    }

    #[test]
    fn message_user_factory() {
        let msg = Message::user("hello");
        assert_eq!(msg.role, Role::User);
        assert_eq!(msg.content, "hello");
    }

    #[test]
    fn message_assistant_factory() {
        let msg = Message::assistant("hi there");
        assert_eq!(msg.role, Role::Assistant);
        assert_eq!(msg.content, "hi there");
    }

    #[test]
    fn message_serializes() {
        let msg = Message::user("test");
        let json: serde_json::Value = serde_json::to_value(&msg).expect("serialize message");
        assert_eq!(json["role"], "user");
        assert_eq!(json["content"], "test");
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
    fn request_stream_defaults_true() {
        let request = MessagesRequest::builder()
            .model("test".to_string())
            .max_tokens(100)
            .build();
        assert!(request.stream);
    }
}
