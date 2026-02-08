//! A2A message handler for incoming peer agent messages.
//!
//! Implements the `MessageHandler` trait from `a2a-rs-server`,
//! processing incoming messages from peer agents with SSE streaming support.
//! When conversation deps are provided, messages are processed through
//! the Claude conversation loop for real responses.

use std::sync::{Arc, OnceLock};

use a2a_rs_core::{
    AgentCard, Message, Role, SendMessageResponse, StreamResponse, Task, TaskState, TaskStatus,
    TaskStatusUpdateEvent, new_message,
};
use a2a_rs_server::{AuthContext, HandlerResult, MessageHandler};
use async_trait::async_trait;
use tokio::sync::broadcast;
use tracing::{error, info};
use uuid::Uuid;

use super::a2a_message_converter::A2aMessageConverter;
use super::agent_card_builder::AgentCardBuilder;
use super::conversation_deps::ConversationDeps;

/// Type alias for the lazily-initialized event sender.
pub type EventSender = Arc<OnceLock<broadcast::Sender<StreamResponse>>>;

/// Handles incoming A2A messages from peer agents.
///
/// Implements the [`MessageHandler`] trait from `a2a-rs-server`.
/// Returns a `Submitted` task immediately, then processes the message
/// in the background via the Claude conversation loop (if configured)
/// or a stub acknowledgment (if not).
pub struct AgentMessageHandler {
    /// Name of this agent (from Agent CR `metadata.name`).
    agent_name: String,
    /// Type of this agent (e.g., "code-generator", "test-generator").
    agent_type: String,
    /// Broadcast sender for streaming task status updates.
    /// Set after server creation via [`EventSender`] `OnceLock`.
    event_sender: EventSender,
    /// Shared dependencies for running the conversation loop.
    /// When `None`, the handler returns a stub acknowledgment.
    conversation_deps: Option<ConversationDeps>,
}

impl AgentMessageHandler {
    /// Creates a new handler for the given agent.
    #[must_use]
    pub fn new(
        agent_name: String,
        agent_type: String,
        event_sender: EventSender,
        conversation_deps: Option<ConversationDeps>,
    ) -> Self {
        Self {
            agent_name,
            agent_type,
            event_sender,
            conversation_deps,
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
        let sender_text = super::a2a_message_converter::extract_text(&message);

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

        // Clone message for the background task (original goes into task history)
        let message_for_processing = message.clone();

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

        // Spawn background processing: Submitted → Working → Completed/Failed
        let event_sender = self.event_sender.clone();
        let agent_name = self.agent_name.clone();
        let conversation_deps = self.conversation_deps.clone();

        tokio::spawn(async move {
            broadcast_status_update(
                &event_sender,
                &task_id,
                &context_id,
                TaskState::Working,
                None,
            );

            let result = process_message(
                &agent_name,
                &task_id,
                &message_for_processing,
                conversation_deps.as_ref(),
            )
            .await;

            match result {
                Ok(response_text) => {
                    let reply = new_message(
                        Role::Agent,
                        &response_text,
                        Some(context_id.clone()),
                    );
                    broadcast_status_update(
                        &event_sender,
                        &task_id,
                        &context_id,
                        TaskState::Completed,
                        Some(reply),
                    );
                }
                Err(err) => {
                    error!("❌ A2A processing failed for {agent_name}: {err:#}");
                    let error_reply = new_message(
                        Role::Agent,
                        &format!("Processing failed: {err:#}"),
                        Some(context_id.clone()),
                    );
                    broadcast_status_update(
                        &event_sender,
                        &task_id,
                        &context_id,
                        TaskState::Failed,
                        Some(error_reply),
                    );
                }
            }
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

/// Processes an incoming A2A message.
///
/// When conversation deps are available, runs the message through the
/// Claude conversation loop. Otherwise returns a stub acknowledgment.
///
/// Each A2A message gets its own audit file (named `{agent}-a2a-{task_id}`)
/// to avoid truncating the main conversation's audit log.
async fn process_message(
    agent_name: &str,
    task_id: &str,
    message: &Message,
    conversation_deps: Option<&ConversationDeps>,
) -> anyhow::Result<String> {
    match conversation_deps {
        Some(deps) => {
            let claude_message = A2aMessageConverter::to_claude_message(message)?;

            info!("🧠 Processing A2A message via conversation loop for {agent_name}");

            let audit_logger = deps
                .workspace_dir
                .as_ref()
                .map(|dir| {
                    let short_id = &task_id[..8.min(task_id.len())];
                    let audit_name = format!("{agent_name}-a2a-{short_id}");
                    crate::audit_logger::AuditLogger::new(dir, &audit_name)
                })
                .transpose()?;

            let result = crate::conversation_loop::run(
                &deps.client,
                deps.executor.as_ref(),
                &deps.loop_config,
                vec![claude_message],
                audit_logger.as_ref(),
            )
            .await?;

            info!(
                "🧠 A2A conversation completed for {agent_name}: {} iteration(s), {} tokens",
                result.iterations,
                result.total_usage.total()
            );

            Ok(result.final_text)
        }
        None => Ok(format!("Message received by {agent_name}")),
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

/// Truncates a string to the given max length, appending "..." if truncated.
fn truncate(text: &str, max_len: usize) -> String {
    if text.len() <= max_len {
        text.to_string()
    } else {
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

    fn test_message(text: &str) -> Message {
        Message {
            message_id: "msg-test".to_string(),
            role: Role::User,
            parts: vec![Part::text(text)],
            context_id: None,
            task_id: None,
            extensions: vec![],
            reference_task_ids: None,
            metadata: None,
        }
    }

    #[test]
    fn creates_handler_without_deps() {
        let handler = AgentMessageHandler::new(
            "test-gen-task-abc".to_string(),
            "test-generator".to_string(),
            test_event_sender(),
            None,
        );
        assert_eq!(handler.agent_name, "test-gen-task-abc");
        assert_eq!(handler.agent_type, "test-generator");
        assert!(handler.conversation_deps.is_none());
    }

    #[test]
    fn agent_card_uses_builder() {
        let handler = AgentMessageHandler::new(
            "code-gen-task-abc".to_string(),
            "code-generator".to_string(),
            test_event_sender(),
            None,
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
            None,
        );
        assert!(handler.supports_streaming());
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
            None,
        );

        let response = handler
            .handle_message(test_message("Run tests please"), None)
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
            None,
        );

        let response = handler
            .handle_message(test_message("Hello"), None)
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

        // Should receive Completed update (stub path)
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

    #[tokio::test]
    async fn process_message_stub_returns_acknowledgment() {
        let result = process_message("test-gen", "task-123", &test_message("Hello"), None)
            .await
            .expect("stub should succeed");
        assert_eq!(result, "Message received by test-gen");
    }

    #[tokio::test]
    async fn process_message_with_empty_parts_fails() {
        let empty_message = Message {
            message_id: "msg-empty".to_string(),
            role: Role::User,
            parts: vec![],
            context_id: None,
            task_id: None,
            extensions: vec![],
            reference_task_ids: None,
            metadata: None,
        };

        // With deps but empty message → conversion error
        let deps = ConversationDeps {
            client: Arc::new(crate::claude_api::client::ClaudeClient::new(
                "key",
                "http://localhost",
            )),
            executor: Arc::new(crate::tool_executor::NoOpToolExecutor),
            loop_config: Arc::new(crate::conversation_loop::ConversationLoopConfig::new(
                "model".to_string(),
                4096,
            )),
            workspace_dir: None,
        };

        let result =
            process_message("test-gen", "task-456", &empty_message, Some(&deps)).await;
        assert!(result.is_err());
        assert!(
            result
                .expect_err("empty")
                .to_string()
                .contains("no text content")
        );
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
            None,
        );
        let card = handler.agent_card("http://agent.ns:9090");
        assert_eq!(
            card.supported_interfaces[0].url,
            "http://agent.ns:9090/v1/rpc"
        );
    }
}
