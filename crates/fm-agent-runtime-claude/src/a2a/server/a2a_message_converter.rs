//! Converts A2A protocol messages to Claude API messages.

use a2a_rs_core::Message as A2aMessage;
use anyhow::{Result, bail};

use crate::claude_api::request::Message as ClaudeMessage;

/// Converts A2A protocol messages to Claude API messages.
///
/// Extracts text content from A2A message parts and constructs
/// a Claude user message suitable for the conversation loop.
pub struct A2aMessageConverter;

impl A2aMessageConverter {
    /// Extracts text from A2A message parts and creates a Claude user message.
    ///
    /// Joins all text parts with newlines.
    ///
    /// # Errors
    ///
    /// Returns an error if no text content is found (fail fast).
    pub fn to_claude_message(a2a_message: &A2aMessage) -> Result<ClaudeMessage> {
        let text = extract_text(a2a_message);

        if text.is_empty() {
            bail!("A2A message contains no text content");
        }

        Ok(ClaudeMessage::user(text))
    }
}

/// Extracts text content from A2A message parts.
///
/// Joins all text parts with newlines.
pub fn extract_text(message: &A2aMessage) -> String {
    message
        .parts
        .iter()
        .filter_map(|p| p.text.as_deref())
        .collect::<Vec<_>>()
        .join("\n")
}

#[cfg(test)]
mod tests {
    use super::*;
    use a2a_rs_core::{Part, Role};

    fn message_with_parts(parts: Vec<Part>) -> A2aMessage {
        A2aMessage {
            message_id: "msg-1".to_string(),
            role: Role::User,
            parts,
            context_id: None,
            task_id: None,
            extensions: vec![],
            reference_task_ids: None,
            metadata: None,
        }
    }

    #[test]
    fn single_text_part_converts() {
        let message = message_with_parts(vec![Part::text("Hello agent!")]);
        let claude_msg = A2aMessageConverter::to_claude_message(&message).expect("convert");
        match &claude_msg.content {
            crate::claude_api::request::MessageContent::Text(t) => {
                assert_eq!(t, "Hello agent!");
            }
            _ => panic!("Expected Text content"),
        }
    }

    #[test]
    fn multiple_text_parts_joined_with_newline() {
        let message = message_with_parts(vec![Part::text("Line 1"), Part::text("Line 2")]);
        let claude_msg = A2aMessageConverter::to_claude_message(&message).expect("convert");
        match &claude_msg.content {
            crate::claude_api::request::MessageContent::Text(t) => {
                assert_eq!(t, "Line 1\nLine 2");
            }
            _ => panic!("Expected Text content"),
        }
    }

    #[test]
    fn empty_parts_returns_error() {
        let message = message_with_parts(vec![]);
        let result = A2aMessageConverter::to_claude_message(&message);
        assert!(result.is_err());
        assert!(
            result
                .expect_err("empty")
                .to_string()
                .contains("no text content")
        );
    }

    #[test]
    fn extract_text_from_message() {
        let message = message_with_parts(vec![Part::text("hello"), Part::text("world")]);
        assert_eq!(extract_text(&message), "hello\nworld");
    }

    #[test]
    fn extract_text_empty_parts() {
        let message = message_with_parts(vec![]);
        assert_eq!(extract_text(&message), "");
    }
}
