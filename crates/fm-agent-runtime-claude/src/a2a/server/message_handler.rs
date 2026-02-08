//! A2A message handler for incoming peer agent messages.
//!
//! Implements the `MessageHandler` trait from `a2a-rs-server`,
//! processing incoming messages from peer agents with SSE streaming support.

use std::sync::{Arc, OnceLock};

use a2a_rs_core::{
    AgentCard, Message, Role, SendMessageResponse, StreamResponse, Task, TaskState, TaskStatus,
    TaskStatusUpdateEvent, new_message,
};
use a2a_rs_server::{AuthContext, HandlerResult, MessageHandler};
use async_trait::async_trait;
use tokio::sync::broadcast;
use tracing::info;
use uuid::Uuid;

use super::agent_card_builder::AgentCardBuilder;

/// Type alias for the lazily-initialized event sender.
pub type EventSender = Arc<OnceLock<broadcast::Sender<StreamResponse>>>;

/// Handles incoming A2A messages from peer agents.
///
/// Implements the [`MessageHandler`] trait from `a2a-rs-server`.
/// Returns a `Submitted` task immediately, then transitions to
/// `Working` → `Completed` in the background via broadcast events.
pub struct AgentMessageHandler {
    /// Name of this agent (from Agent CR `metadata.name`).
    agent_name: String,
    /// Type of this agent (e.g., "code-generator", "test-generator").
    agent_type: String,
    /// Broadcast sender for streaming task status updates.
    /// Set after server creation via [`EventSender`] `OnceLock`.
    event_sender: EventSender,
}

impl AgentMessageHandler {
    /// Creates a new handler for the given agent.
    #[must_use]
    pub fn new(agent_name: String, agent_type: String, event_sender: EventSender) -> Self {
        Self {
            agent_name,
            agent_type,
            event_sender,
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

        let context_id = message
            .context_id
            .clone()
            .unwrap_or_else(|| Uuid::new_v4().to_string());
        let task_id = Uuid::new_v4().to_string();

        // Return task in Submitted state — SSE will stream transitions
        let task = Task {
            id: task_id.clone(),
            context_id: context_id.clone(),
            status: TaskStatus {
                state: TaskState::Submitted,
                message: None,
                timestamp: Some(chrono::Utc::now().to_rfc3339()),
            },
            history: Some(vec![message]),
            artifacts: None,
            metadata: None,
        };

        // Spawn background processing: Submitted → Working → Completed
        let event_sender = self.event_sender.clone();
        let agent_name = self.agent_name.clone();

        tokio::spawn(async move {
            broadcast_status_update(
                &event_sender,
                &task_id,
                &context_id,
                TaskState::Working,
                None,
            );

            // Acknowledge with completed status
            let reply = new_message(
                Role::Agent,
                &format!("Message received by {agent_name}"),
                Some(context_id.clone()),
            );

            broadcast_status_update(
                &event_sender,
                &task_id,
                &context_id,
                TaskState::Completed,
                Some(reply),
            );
        });

        Ok(SendMessageResponse::Task(task))
    }

    fn agent_card(&self, base_url: &str) -> AgentCard {
        AgentCardBuilder::new(&self.agent_name, &self.agent_type).build(base_url)
    }

    fn supports_streaming(&self) -> bool {
        true
    }
}

/// Broadcasts a task status update via the event sender.
fn broadcast_status_update(
    event_sender: &EventSender,
    task_id: &str,
    context_id: &str,
    state: TaskState,
    message: Option<Message>,
) {
    if let Some(tx) = event_sender.get() {
        let event = StreamResponse::StatusUpdate(TaskStatusUpdateEvent {
            task_id: task_id.to_string(),
            context_id: context_id.to_string(),
            status: TaskStatus {
                state,
                message,
                timestamp: Some(chrono::Utc::now().to_rfc3339()),
            },
            metadata: None,
        });
        let _ = tx.send(event);
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
    use a2a_rs_core::Part;

    fn test_event_sender() -> EventSender {
        Arc::new(OnceLock::new())
    }

    #[test]
    fn creates_handler() {
        let handler = AgentMessageHandler::new(
            "test-gen-task-abc".to_string(),
            "test-generator".to_string(),
            test_event_sender(),
        );
        assert_eq!(handler.agent_name, "test-gen-task-abc");
        assert_eq!(handler.agent_type, "test-generator");
    }

    #[test]
    fn agent_card_uses_builder() {
        let handler = AgentMessageHandler::new(
            "code-gen-task-abc".to_string(),
            "code-generator".to_string(),
            test_event_sender(),
        );
        let card =
            handler.agent_card("http://code-gen-task-abc.task-abc.svc.cluster.local:9090");

        assert_eq!(card.name, "code-gen-task-abc");
        assert!(!card.supported_interfaces.is_empty());
    }

    #[test]
    fn supports_streaming_returns_true() {
        let handler = AgentMessageHandler::new(
            "agent".to_string(),
            "type".to_string(),
            test_event_sender(),
        );
        assert!(handler.supports_streaming());
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
    async fn handle_message_returns_submitted_task() {
        let handler = AgentMessageHandler::new(
            "test-gen".to_string(),
            "test-generator".to_string(),
            test_event_sender(),
        );

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
                assert_eq!(task.status.state, TaskState::Submitted);
            }
            SendMessageResponse::Message(_) => panic!("Expected Task, got Message"),
        }
    }

    #[tokio::test]
    async fn handle_message_broadcasts_status_updates() {
        let event_sender: EventSender = Arc::new(OnceLock::new());
        let (tx, mut rx) = broadcast::channel::<StreamResponse>(16);
        event_sender.set(tx).ok();

        let handler = AgentMessageHandler::new(
            "test-gen".to_string(),
            "test-generator".to_string(),
            event_sender,
        );

        let message = Message {
            message_id: "msg-test".to_string(),
            role: Role::User,
            parts: vec![Part::text("Hello")],
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

        let task_id = match &response {
            SendMessageResponse::Task(task) => task.id.clone(),
            _ => panic!("Expected Task"),
        };

        // Give background task time to broadcast
        tokio::time::sleep(std::time::Duration::from_millis(50)).await;

        // Should receive Working update
        let event = rx.try_recv().expect("should receive Working event");
        match event {
            StreamResponse::StatusUpdate(e) => {
                assert_eq!(e.task_id, task_id);
                assert_eq!(e.status.state, TaskState::Working);
            }
            _ => panic!("Expected StatusUpdate(Working)"),
        }

        // Should receive Completed update
        let event = rx.try_recv().expect("should receive Completed event");
        match event {
            StreamResponse::StatusUpdate(e) => {
                assert_eq!(e.task_id, task_id);
                assert_eq!(e.status.state, TaskState::Completed);
                assert!(e.status.message.is_some());
            }
            _ => panic!("Expected StatusUpdate(Completed)"),
        }
    }

    #[test]
    fn broadcast_status_update_without_sender_is_noop() {
        let event_sender = test_event_sender();
        // No sender set — should not panic
        broadcast_status_update(
            &event_sender,
            "task-1",
            "ctx-1",
            TaskState::Working,
            None,
        );
    }

    #[test]
    fn agent_card_has_correct_rpc_endpoint() {
        let handler = AgentMessageHandler::new(
            "agent".to_string(),
            "code-generator".to_string(),
            test_event_sender(),
        );
        let card = handler.agent_card("http://agent.ns:9090");
        assert_eq!(
            card.supported_interfaces[0].url,
            "http://agent.ns:9090/v1/rpc"
        );
    }
}
