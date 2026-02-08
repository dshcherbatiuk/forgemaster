//! A2A tool executor for inter-agent communication.
//!
//! Implements the `ToolExecutor` trait to expose A2A operations
//! as tools available to Claude during the conversation loop.

use std::time::Duration;

use a2a_rs_client::A2aClient;
use a2a_rs_core::{Role, SendMessageResponse, new_message};
use anyhow::{Result, bail};
use async_trait::async_trait;
use tracing::debug;

use crate::claude_api::tool_definition::ToolDefinition;
use crate::tool_executor::{ToolCallResult, ToolExecutor};

/// Tool names for A2A operations.
const TOOL_SEND_MESSAGE: &str = "a2a_send_message";
const TOOL_GET_AGENT_CARD: &str = "a2a_get_agent_card";
const TOOL_GET_TASK_STATUS: &str = "a2a_get_task_status";
const TOOL_SUBSCRIBE: &str = "a2a_subscribe";

/// Executes A2A operations as tools in the conversation loop.
///
/// Exposes four tools to Claude:
/// - `a2a_send_message` — Send a message to a peer agent
/// - `a2a_get_agent_card` — Discover peer capabilities via Agent Card
/// - `a2a_get_task_status` — Check task status at a peer agent
/// - `a2a_subscribe` — Subscribe to real-time task updates via SSE
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

    /// Sends a message to a peer agent.
    async fn send_message(&self, input: &serde_json::Value) -> Result<ToolCallResult> {
        let agent_url = require_field(input, "agent_url")?;
        let message_text = require_field(input, "message")?;

        debug!("📨 A2A send_message to {agent_url}");

        let client = A2aClient::with_server(agent_url)?;
        let message = new_message(Role::User, message_text, None);
        let response = client.send_message(message, None).await?;

        let content = match response {
            SendMessageResponse::Task(task) => serde_json::to_string_pretty(&task)?,
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

    /// Polls task status from a peer agent.
    async fn get_task_status(&self, input: &serde_json::Value) -> Result<ToolCallResult> {
        let agent_url = require_field(input, "agent_url")?;
        let task_id = require_field(input, "task_id")?;

        debug!("📊 A2A get_task_status from {agent_url} task={task_id}");

        let client = A2aClient::with_server(agent_url)?;
        let task = client.poll_task(task_id, None).await?;

        Ok(ToolCallResult {
            content: serde_json::to_string_pretty(&task)?,
            is_error: false,
        })
    }

    /// Subscribes to real-time task updates via SSE streaming.
    async fn subscribe(&self, input: &serde_json::Value) -> Result<ToolCallResult> {
        let agent_url = require_field(input, "agent_url")?;
        let task_id = require_field(input, "task_id")?;
        let timeout_secs = input["timeout_secs"].as_u64().unwrap_or(30);

        debug!("📡 A2A subscribe to {agent_url} task={task_id} timeout={timeout_secs}s");

        let events = super::sse_stream::subscribe(
            agent_url,
            task_id,
            Some(Duration::from_secs(timeout_secs)),
        )
        .await?;

        Ok(ToolCallResult {
            content: serde_json::to_string_pretty(&events)?,
            is_error: false,
        })
    }
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
        .description("Send a message to a peer agent via A2A protocol".to_string())
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

/// Builds the tool definition for `a2a_get_task_status`.
fn get_task_status_tool() -> ToolDefinition {
    ToolDefinition::builder()
        .name(TOOL_GET_TASK_STATUS.to_string())
        .description("Check the status of a task at a peer agent".to_string())
        .input_schema(serde_json::json!({
            "type": "object",
            "properties": {
                "agent_url": {
                    "type": "string",
                    "description": "Full URL of the peer agent"
                },
                "task_id": {
                    "type": "string",
                    "description": "ID of the task to check"
                }
            },
            "required": ["agent_url", "task_id"]
        }))
        .build()
}

/// Builds the tool definition for `a2a_subscribe`.
fn subscribe_tool() -> ToolDefinition {
    ToolDefinition::builder()
        .name(TOOL_SUBSCRIBE.to_string())
        .description(
            "Subscribe to real-time task status updates from a peer agent via SSE streaming. \
             Returns all events until the task completes or timeout is reached."
                .to_string(),
        )
        .input_schema(serde_json::json!({
            "type": "object",
            "properties": {
                "agent_url": {
                    "type": "string",
                    "description": "Full URL of the peer agent"
                },
                "task_id": {
                    "type": "string",
                    "description": "ID of the task to subscribe to"
                },
                "timeout_secs": {
                    "type": "integer",
                    "description": "Maximum seconds to wait for updates (default: 30)"
                }
            },
            "required": ["agent_url", "task_id"]
        }))
        .build()
}

#[async_trait]
impl ToolExecutor for A2aToolExecutor {
    async fn discover_tools(&self) -> Result<Vec<ToolDefinition>> {
        Ok(vec![
            send_message_tool(),
            get_agent_card_tool(),
            get_task_status_tool(),
            subscribe_tool(),
        ])
    }

    async fn execute_tool(
        &self,
        name: &str,
        input: &serde_json::Value,
    ) -> Result<ToolCallResult> {
        match name {
            TOOL_SEND_MESSAGE => self.send_message(input).await,
            TOOL_GET_AGENT_CARD => self.get_agent_card(input).await,
            TOOL_GET_TASK_STATUS => self.get_task_status(input).await,
            TOOL_SUBSCRIBE => self.subscribe(input).await,
            _ => bail!("unknown A2A tool: {name}"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn discover_tools_returns_four() {
        let executor = A2aToolExecutor::new();
        let tools = executor.discover_tools().await.expect("discover");
        assert_eq!(tools.len(), 4);
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
    fn get_task_status_tool_has_required_fields() {
        let tool = get_task_status_tool();
        let required = tool.input_schema["required"]
            .as_array()
            .expect("required array");
        assert!(required.contains(&serde_json::json!("agent_url")));
        assert!(required.contains(&serde_json::json!("task_id")));
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
        assert!(result.expect_err("missing").to_string().contains("agent_url"));
    }

    #[tokio::test]
    async fn execute_unknown_tool_fails() {
        let executor = A2aToolExecutor::new();
        let result = executor
            .execute_tool("unknown", &serde_json::json!({}))
            .await;
        assert!(result.is_err());
        assert!(result.expect_err("unknown").to_string().contains("unknown"));
    }

    #[test]
    fn subscribe_tool_has_required_fields() {
        let tool = subscribe_tool();
        let required = tool.input_schema["required"]
            .as_array()
            .expect("required array");
        assert!(required.contains(&serde_json::json!("agent_url")));
        assert!(required.contains(&serde_json::json!("task_id")));
    }

    #[test]
    fn subscribe_tool_has_optional_timeout() {
        let tool = subscribe_tool();
        let properties = tool.input_schema["properties"].as_object().expect("props");
        assert!(properties.contains_key("timeout_secs"));
    }

    #[test]
    fn tool_descriptions_are_set() {
        let tools = vec![
            send_message_tool(),
            get_agent_card_tool(),
            get_task_status_tool(),
        ];
        for tool in &tools {
            assert!(
                tool.description.is_some(),
                "tool {} should have description",
                tool.name
            );
        }
    }
}
