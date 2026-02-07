//! Agent execution orchestrator.

use anyhow::{Result, bail};
use tracing::{debug, info};

use crate::claude_api::client::ClaudeClient;
use crate::claude_api::request::{Message, MessagesRequest};
use crate::claude_api::response::Usage;
use crate::claude_api::sse::SseEvent;
use crate::config::RuntimeConfig;
use crate::conversation::Conversation;
use crate::output_writer::OutputWriter;
use crate::status_updater::StatusUpdater;

/// Orchestrates the full agent execution lifecycle.
pub struct AgentExecution {
    config: RuntimeConfig,
    k8s_client: kube::Client,
}

impl AgentExecution {
    /// Creates a new execution for the given config and K8s client.
    pub fn new(config: RuntimeConfig, k8s_client: kube::Client) -> Self {
        Self { config, k8s_client }
    }

    /// Runs the agent: calls Claude API, logs output, updates Agent CR status.
    pub async fn run(&self) -> Result<()> {
        let status_updater = StatusUpdater::new(
            self.k8s_client.clone(),
            &self.config.namespace,
            &self.config.agent_name,
        );
        let output_writer = OutputWriter::new(&self.config.agent_name);

        info!(
            "📋 Agent: {}, model: {}",
            self.config.agent_name, self.config.model_name
        );

        // 1. Transition to Running
        status_updater.transition_to_running().await?;

        // 2. Build conversation
        let mut conversation = Conversation::new(self.config.system_prompt.clone());
        conversation.add_user_message(self.config.task_prompt.clone());

        // 3. Call Claude API with streaming
        let client = ClaudeClient::new(&self.config.api_key, &self.config.api_base_url);
        let request = MessagesRequest::builder()
            .model(self.config.model_name.clone())
            .max_tokens(self.config.model_max_tokens)
            .messages(vec![Message::user(&self.config.task_prompt)])
            .system(self.config.system_prompt.clone())
            .temperature(self.config.model_temperature)
            .build();

        debug!("🤖 Sending request to Claude API");
        let events = client.send_streaming(&request).await?;

        // 4. Extract text and usage from SSE events
        let (assistant_text, usage) = collect_response(&events)?;

        conversation.add_assistant_message(assistant_text.clone());
        conversation.add_usage(usage);

        // 5. Log the output
        output_writer.write(&assistant_text);

        // 6. Transition to Succeeded
        status_updater
            .transition_to_succeeded(conversation.total_tokens(), 1)
            .await?;

        info!(
            "✅ Agent {} completed — {} tokens used",
            self.config.agent_name,
            conversation.total_tokens()
        );

        Ok(())
    }
}

/// Extracts the accumulated text and combined usage from SSE events.
fn collect_response(events: &[SseEvent]) -> Result<(String, Usage)> {
    let mut text = String::new();
    let mut combined_usage = Usage::default();

    for event in events {
        match event {
            SseEvent::MessageStart { usage, .. } => {
                combined_usage.input_tokens += usage.input_tokens;
                combined_usage.output_tokens += usage.output_tokens;
            }
            SseEvent::ContentBlockDelta { text: delta, .. } => {
                text.push_str(delta);
            }
            SseEvent::MessageDelta { usage, .. } => {
                combined_usage.input_tokens += usage.input_tokens;
                combined_usage.output_tokens += usage.output_tokens;
            }
            SseEvent::Error {
                error_type,
                message,
            } => {
                bail!("Claude API error: {error_type} — {message}");
            }
            _ => {}
        }
    }

    Ok((text, combined_usage))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::claude_api::response::StopReason;

    #[test]
    fn collect_response_empty_events() {
        let (text, usage) = collect_response(&[]).unwrap();
        assert!(text.is_empty());
        assert_eq!(usage.input_tokens, 0);
        assert_eq!(usage.output_tokens, 0);
    }

    #[test]
    fn collect_response_accumulates_text() {
        let events = vec![
            SseEvent::MessageStart {
                message_id: "msg_1".to_string(),
                usage: Usage {
                    input_tokens: 100,
                    output_tokens: 0,
                },
            },
            SseEvent::ContentBlockStart { index: 0 },
            SseEvent::ContentBlockDelta {
                index: 0,
                text: "Hello".to_string(),
            },
            SseEvent::ContentBlockDelta {
                index: 0,
                text: " World".to_string(),
            },
            SseEvent::ContentBlockStop { index: 0 },
            SseEvent::MessageDelta {
                stop_reason: Some(StopReason::EndTurn),
                usage: Usage {
                    input_tokens: 0,
                    output_tokens: 25,
                },
            },
            SseEvent::MessageStop,
        ];

        let (text, usage) = collect_response(&events).unwrap();
        assert_eq!(text, "Hello World");
        assert_eq!(usage.input_tokens, 100);
        assert_eq!(usage.output_tokens, 25);
    }

    #[test]
    fn collect_response_error_event_fails() {
        let events = vec![SseEvent::Error {
            error_type: "overloaded_error".to_string(),
            message: "Server busy".to_string(),
        }];

        let result = collect_response(&events);
        assert!(result.is_err());
        let err = result.unwrap_err().to_string();
        assert!(err.contains("overloaded_error"));
        assert!(err.contains("Server busy"));
    }

    #[test]
    fn collect_response_multiple_content_blocks() {
        let events = vec![
            SseEvent::ContentBlockDelta {
                index: 0,
                text: "First block. ".to_string(),
            },
            SseEvent::ContentBlockDelta {
                index: 1,
                text: "Second block.".to_string(),
            },
        ];

        let (text, _) = collect_response(&events).unwrap();
        assert_eq!(text, "First block. Second block.");
    }

    #[test]
    fn collect_response_ping_ignored() {
        let events = vec![
            SseEvent::Ping,
            SseEvent::ContentBlockDelta {
                index: 0,
                text: "data".to_string(),
            },
            SseEvent::Ping,
        ];

        let (text, _) = collect_response(&events).unwrap();
        assert_eq!(text, "data");
    }
}
