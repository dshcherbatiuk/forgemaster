//! A2A tool executor for inter-agent communication.
//!
//! Implements the `ToolExecutor` trait to expose A2A operations
//! as tools available to Claude during the conversation loop.
//! SSE streaming is handled transparently — `send_message` automatically
//! subscribes to task updates and returns the completed result.

use std::time::Duration;

use a2a_rs_client::A2aClient;
use a2a_rs_core::{Role, SendMessageResponse, StreamResponse, new_message};
use anyhow::{Result, bail};
use async_trait::async_trait;
use tracing::debug;

use crate::claude_api::tool_definition::ToolDefinition;
use crate::tool_executor::{ToolCallResult, ToolExecutor};

/// Tool names for A2A operations.
const TOOL_SEND_MESSAGE: &str = "a2a_send_message";
const TOOL_GET_AGENT_CARD: &str = "a2a_get_agent_card";

/// Default timeout for waiting on SSE task completion.
const SSE_TIMEOUT: Duration = Duration::from_secs(30);

/// Executes A2A operations as tools in the conversation loop.
///
/// Exposes two tools to Claude:
/// - `a2a_send_message` — Send a message to a peer agent (auto-subscribes via SSE)
/// - `a2a_get_agent_card` — Discover peer capabilities via Agent Card
pub struct A2aToolExecutor;

impl Default for A2aToolExecutor {
    fn default() -> Self {
        Self
    }
}

impl A2aToolExecutor {
    /// Creates a new A2A tool executor.
    #[must_use]
    pub const fn new() -> Self {
        Self
    }

    /// Sends a message to a peer agent and waits for completion via SSE.
    ///
    /// 1. Sends the message via JSON-RPC
    /// 2. If the response is a non-terminal task, auto-subscribes via SSE
    /// 3. Returns the final completed result
    async fn send_message(&self, input: &serde_json::Value) -> Result<ToolCallResult> {
        let agent_url = require_field(input, "agent_url")?;
        let message_text = require_field(input, "message")?;

        debug!("📨 A2A send_message to {agent_url}");

        let client = A2aClient::with_server(agent_url)?;
        let message = new_message(Role::User, message_text, None);
        let response = client.send_message(message, None).await?;

        let content = match response {
            SendMessageResponse::Task(task) => {
                if task.status.state.is_terminal() {
                    serde_json::to_string_pretty(&task)?
                } else {
                    // Auto-subscribe via SSE and wait for completion
                    debug!("📡 Auto-subscribing to task {} via SSE", task.id);
                    let events = super::sse_stream::subscribe(
                        agent_url,
                        &task.id,
                        Some(SSE_TIMEOUT),
                    )
                    .await?;

                    // Return the last terminal event, or the collected events
                    last_terminal_task(&events)
                        .unwrap_or_else(|| serde_json::to_string_pretty(&events)
                            .unwrap_or_default())
                }
            }
            SendMessageResponse::Message(msg) => serde_json::to_string_pretty(&msg)?,
        };

        Ok(ToolCallResult {
            content,
            is_error: false,
        })
    }

    /// Fetches a peer agent's Agent Card.
    async fn get_agent_card(&self, input: &serde_json::Value) -> Result<ToolCallResult> {
        let agent_url = require_field(input, "agent_url")?;

        debug!("📇 A2A get_agent_card from {agent_url}");

        let client = A2aClient::with_server(agent_url)?;
        let card = client.fetch_agent_card().await?;

        Ok(ToolCallResult {
            content: serde_json::to_string_pretty(&card)?,
            is_error: false,
        })
    }
}

/// Extracts the last terminal task from SSE events as pretty JSON.
fn last_terminal_task(events: &[StreamResponse]) -> Option<String> {
    events
        .iter()
        .rev()
        .find_map(|event| match event {
            StreamResponse::Task(t) if t.status.state.is_terminal() => {
                serde_json::to_string_pretty(t).ok()
            }
            StreamResponse::StatusUpdate(e) if e.status.state.is_terminal() => {
                serde_json::to_string_pretty(e).ok()
            }
            _ => None,
        })
}

/// Extracts a required string field from JSON input.
fn require_field<'a>(input: &'a serde_json::Value, field: &str) -> Result<&'a str> {
    input[field]
        .as_str()
        .ok_or_else(|| anyhow::anyhow!("{field} is required"))
}

/// Builds the tool definition for `a2a_send_message`.
fn send_message_tool() -> ToolDefinition {
    ToolDefinition::builder()
        .name(TOOL_SEND_MESSAGE.to_string())
        .description(
            "Send a message to a peer agent via A2A protocol. \
             Waits for the peer to process the message and returns the completed result."
                .to_string(),
        )
        .input_schema(serde_json::json!({
            "type": "object",
            "properties": {
                "agent_url": {
                    "type": "string",
                    "description": "Full URL of the peer agent (e.g. http://agent-name.namespace.svc.cluster.local:9090)"
                },
                "message": {
                    "type": "string",
                    "description": "Text message to send to the peer agent"
                }
            },
            "required": ["agent_url", "message"]
        }))
        .build()
}

/// Builds the tool definition for `a2a_get_agent_card`.
fn get_agent_card_tool() -> ToolDefinition {
    ToolDefinition::builder()
        .name(TOOL_GET_AGENT_CARD.to_string())
        .description(
            "Get a peer agent's Agent Card to discover its capabilities and skills".to_string(),
        )
        .input_schema(serde_json::json!({
            "type": "object",
            "properties": {
                "agent_url": {
                    "type": "string",
                    "description": "Full URL of the peer agent (e.g. http://agent-name.namespace.svc.cluster.local:9090)"
                }
            },
            "required": ["agent_url"]
        }))
        .build()
}

#[async_trait]
impl ToolExecutor for A2aToolExecutor {
    async fn discover_tools(&self) -> Result<Vec<ToolDefinition>> {
        Ok(vec![send_message_tool(), get_agent_card_tool()])
    }

    async fn execute_tool(
        &self,
        name: &str,
        input: &serde_json::Value,
    ) -> Result<ToolCallResult> {
        match name {
            TOOL_SEND_MESSAGE => self.send_message(input).await,
            TOOL_GET_AGENT_CARD => self.get_agent_card(input).await,
            _ => bail!("unknown A2A tool: {name}"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use a2a_rs_core::{Task, TaskState, TaskStatus, TaskStatusUpdateEvent};

    #[tokio::test]
    async fn discover_tools_returns_two() {
        let executor = A2aToolExecutor::new();
        let tools = executor.discover_tools().await.expect("discover");
        assert_eq!(tools.len(), 2);
    }

    #[tokio::test]
    async fn tool_names_are_prefixed() {
        let executor = A2aToolExecutor::new();
        let tools = executor.discover_tools().await.expect("discover");
        for tool in &tools {
            assert!(
                tool.name.starts_with("a2a_"),
                "tool name should start with a2a_: {}",
                tool.name
            );
        }
    }

    #[test]
    fn send_message_tool_has_required_fields() {
        let tool = send_message_tool();
        let required = tool.input_schema["required"]
            .as_array()
            .expect("required array");
        assert!(required.contains(&serde_json::json!("agent_url")));
        assert!(required.contains(&serde_json::json!("message")));
    }

    #[test]
    fn get_agent_card_tool_has_required_fields() {
        let tool = get_agent_card_tool();
        let required = tool.input_schema["required"]
            .as_array()
            .expect("required array");
        assert!(required.contains(&serde_json::json!("agent_url")));
    }

    #[test]
    fn require_field_present() {
        let input = serde_json::json!({"agent_url": "http://test:9090"});
        let result = require_field(&input, "agent_url");
        assert!(result.is_ok());
        assert_eq!(result.expect("field"), "http://test:9090");
    }

    #[test]
    fn require_field_missing() {
        let input = serde_json::json!({});
        let result = require_field(&input, "agent_url");
        assert!(result.is_err());
        assert!(
            result
                .expect_err("missing")
                .to_string()
                .contains("agent_url")
        );
    }

    #[tokio::test]
    async fn execute_unknown_tool_fails() {
        let executor = A2aToolExecutor::new();
        let result = executor
            .execute_tool("unknown", &serde_json::json!({}))
            .await;
        assert!(result.is_err());
        assert!(
            result
                .expect_err("unknown")
                .to_string()
                .contains("unknown")
        );
    }

    #[test]
    fn tool_descriptions_are_set() {
        let tools = vec![send_message_tool(), get_agent_card_tool()];
        for tool in &tools {
            assert!(
                tool.description.is_some(),
                "tool {} should have description",
                tool.name
            );
        }
    }

    #[test]
    fn last_terminal_task_finds_completed() {
        let events = vec![
            StreamResponse::StatusUpdate(TaskStatusUpdateEvent {
                task_id: "t1".to_string(),
                context_id: "c1".to_string(),
                status: TaskStatus {
                    state: TaskState::Working,
                    message: None,
                    timestamp: None,
                },
                metadata: None,
            }),
            StreamResponse::Task(Task {
                id: "t1".to_string(),
                context_id: "c1".to_string(),
                status: TaskStatus {
                    state: TaskState::Completed,
                    message: None,
                    timestamp: None,
                },
                artifacts: None,
                history: None,
                metadata: None,
            }),
        ];
        let result = last_terminal_task(&events);
        assert!(result.is_some());
        assert!(result.unwrap().contains("TASK_STATE_COMPLETED"));
    }

    #[test]
    fn last_terminal_task_returns_none_for_non_terminal() {
        let events = vec![StreamResponse::StatusUpdate(TaskStatusUpdateEvent {
            task_id: "t1".to_string(),
            context_id: "c1".to_string(),
            status: TaskStatus {
                state: TaskState::Working,
                message: None,
                timestamp: None,
            },
            metadata: None,
        })];
        assert!(last_terminal_task(&events).is_none());
    }

    #[test]
    fn last_terminal_task_empty_events() {
        assert!(last_terminal_task(&[]).is_none());
    }

    #[test]
    fn sse_timeout_is_30_seconds() {
        assert_eq!(SSE_TIMEOUT, Duration::from_secs(30));
    }
}
