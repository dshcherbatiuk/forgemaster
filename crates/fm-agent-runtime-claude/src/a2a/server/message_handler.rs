//! A2A message handler for incoming peer agent messages.
//!
//! Implements the `MessageHandler` trait from `a2a-rs-server`,
//! processing incoming messages from peer agents.

use a2a_rs_core::{AgentCard, Message, SendMessageResponse, completed_task_with_text};
use a2a_rs_server::{AuthContext, HandlerResult, MessageHandler};
use async_trait::async_trait;
use tracing::info;

use super::agent_card_builder::AgentCardBuilder;

/// Handles incoming A2A messages from peer agents.
///
/// Implements the [`MessageHandler`] trait from `a2a-rs-server`.
/// Currently acknowledges messages immediately; future versions
/// will route messages to the conversation loop for processing.
pub struct AgentMessageHandler {
    /// Name of this agent (from Agent CR `metadata.name`).
    agent_name: String,
    /// Type of this agent (e.g., "code-generator", "test-generator").
    agent_type: String,
}

impl AgentMessageHandler {
    /// Creates a new handler for the given agent.
    #[must_use]
    pub fn new(agent_name: String, agent_type: String) -> Self {
        Self {
            agent_name,
            agent_type,
        }
    }
}

#[async_trait]
impl MessageHandler for AgentMessageHandler {
    async fn handle_message(
        &self,
        message: Message,
        _auth: Option<AuthContext>,
    ) -> HandlerResult<SendMessageResponse> {
        let sender_text = extract_text(&message);

        info!(
            "📨 A2A message received by {}: {}",
            self.agent_name,
            truncate(&sender_text, 100)
        );

        let task = completed_task_with_text(
            message,
            &format!("Message received by {}", self.agent_name),
        );

        Ok(SendMessageResponse::Task(task))
    }

    fn agent_card(&self, base_url: &str) -> AgentCard {
        AgentCardBuilder::new(&self.agent_name, &self.agent_type).build(base_url)
    }
}

/// Extracts text content from message parts.
fn extract_text(message: &Message) -> String {
    message
        .parts
        .iter()
        .filter_map(|p| p.text.as_deref())
        .collect::<Vec<_>>()
        .join("\n")
}

/// Truncates a string to the given max length, appending "..." if truncated.
fn truncate(text: &str, max_len: usize) -> String {
    if text.len() <= max_len {
        text.to_string()
    } else {
        // Find a safe truncation point at a char boundary
        let mut end = max_len;
        while !text.is_char_boundary(end) && end > 0 {
            end -= 1;
        }
        format!("{}...", &text[..end])
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use a2a_rs_core::{Part, Role};

    #[test]
    fn creates_handler() {
        let handler =
            AgentMessageHandler::new("test-gen-task-abc".to_string(), "test-generator".to_string());
        assert_eq!(handler.agent_name, "test-gen-task-abc");
        assert_eq!(handler.agent_type, "test-generator");
    }

    #[test]
    fn agent_card_uses_builder() {
        let handler =
            AgentMessageHandler::new("code-gen-task-abc".to_string(), "code-generator".to_string());
        let card =
            handler.agent_card("http://code-gen-task-abc.task-abc.svc.cluster.local:9090");

        assert_eq!(card.name, "code-gen-task-abc");
        assert!(!card.supported_interfaces.is_empty());
    }

    #[test]
    fn extract_text_single_part() {
        let message = Message {
            message_id: "msg-1".to_string(),
            role: Role::User,
            parts: vec![Part::text("Hello agent!")],
            context_id: None,
            task_id: None,
            extensions: vec![],
            reference_task_ids: None,
            metadata: None,
        };
        assert_eq!(extract_text(&message), "Hello agent!");
    }

    #[test]
    fn extract_text_multiple_parts() {
        let message = Message {
            message_id: "msg-2".to_string(),
            role: Role::User,
            parts: vec![Part::text("Line 1"), Part::text("Line 2")],
            context_id: None,
            task_id: None,
            extensions: vec![],
            reference_task_ids: None,
            metadata: None,
        };
        assert_eq!(extract_text(&message), "Line 1\nLine 2");
    }

    #[test]
    fn extract_text_empty_parts() {
        let message = Message {
            message_id: "msg-3".to_string(),
            role: Role::User,
            parts: vec![],
            context_id: None,
            task_id: None,
            extensions: vec![],
            reference_task_ids: None,
            metadata: None,
        };
        assert_eq!(extract_text(&message), "");
    }

    #[test]
    fn truncate_short_text() {
        assert_eq!(truncate("short", 10), "short");
    }

    #[test]
    fn truncate_long_text() {
        let long = "a".repeat(200);
        let result = truncate(&long, 100);
        assert_eq!(result.len(), 103); // 100 + "..."
        assert!(result.ends_with("..."));
    }

    #[test]
    fn truncate_exact_length() {
        let exact = "a".repeat(100);
        assert_eq!(truncate(&exact, 100), exact);
    }

    #[tokio::test]
    async fn handle_message_returns_completed_task() {
        let handler =
            AgentMessageHandler::new("test-gen".to_string(), "test-generator".to_string());

        let message = Message {
            message_id: "msg-test".to_string(),
            role: Role::User,
            parts: vec![Part::text("Run tests please")],
            context_id: None,
            task_id: None,
            extensions: vec![],
            reference_task_ids: None,
            metadata: None,
        };

        let response = handler
            .handle_message(message, None)
            .await
            .expect("should succeed");

        match response {
            SendMessageResponse::Task(task) => {
                assert_eq!(task.status.state, a2a_rs_core::TaskState::Completed);
            }
            SendMessageResponse::Message(_) => panic!("Expected Task, got Message"),
        }
    }

    #[test]
    fn agent_card_has_correct_rpc_endpoint() {
        let handler =
            AgentMessageHandler::new("agent".to_string(), "code-generator".to_string());
        let card = handler.agent_card("http://agent.ns:9090");
        assert_eq!(card.supported_interfaces[0].url, "http://agent.ns:9090/v1/rpc");
    }
}
