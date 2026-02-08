//! Multi-turn conversation loop with tool calling support.
//!
//! Sends messages to Claude, handles `tool_use` responses by executing tools
//! via the `ToolExecutor`, and feeds results back until Claude returns `end_turn`.

use anyhow::{Result, bail};
use tracing::{debug, info, warn};

use crate::audit_logger::AuditLogger;
use crate::claude_api::client::ClaudeClient;
use crate::claude_api::content_block::RequestContentBlock;
use crate::claude_api::request::{Message, MessagesRequest};
use crate::claude_api::response::{StopReason, Usage};
use crate::claude_api::response_collector::{self, CollectedResponse};
use crate::claude_api::tool_definition::ToolDefinition;
use crate::tool_executor::ToolExecutor;

const DEFAULT_MAX_ITERATIONS: u32 = 50;

/// Configuration for the conversation loop.
#[derive(Debug, Clone)]
pub struct ConversationLoopConfig {
    /// Claude model identifier.
    pub model: String,
    /// Maximum tokens per response.
    pub max_tokens: i32,
    /// System prompt (agent role + task description).
    pub system: Option<String>,
    /// Sampling temperature.
    pub temperature: Option<f64>,
    /// Safety limit on tool calling rounds.
    pub max_iterations: u32,
}

impl ConversationLoopConfig {
    /// Creates config with default max_iterations.
    pub fn new(model: String, max_tokens: i32) -> Self {
        Self {
            model,
            max_tokens,
            system: None,
            temperature: None,
            max_iterations: DEFAULT_MAX_ITERATIONS,
        }
    }
}

/// Result of a completed conversation loop.
#[derive(Debug)]
pub struct ConversationResult {
    /// Final text output from Claude.
    pub final_text: String,
    /// Total token usage across all iterations.
    pub total_usage: Usage,
    /// Number of API round-trips.
    pub iterations: u32,
}

/// Runs a multi-turn conversation with tool calling.
///
/// Discovers tools from the executor, sends them to Claude, and handles
/// tool_use/tool_result cycles until Claude returns `end_turn`.
///
/// When `audit_logger` is provided, every request and response is logged
/// to the agent's audit markdown file.
pub async fn run(
    client: &ClaudeClient,
    executor: &dyn ToolExecutor,
    config: &ConversationLoopConfig,
    initial_messages: Vec<Message>,
    audit_logger: Option<&AuditLogger>,
) -> Result<ConversationResult> {
    let tools = executor.discover_tools().await?;

    if tools.is_empty() {
        debug!("📝 No tools available, running single-turn conversation");
    } else {
        info!(
            "🔧 Discovered {} tool(s): {}",
            tools.len(),
            format_tool_names(&tools)
        );
    }

    if let Some(logger) = audit_logger {
        logger.log_header(&config.model, config.system.as_deref())?;
    }

    let mut messages = initial_messages;
    let mut total_usage = Usage::default();
    let mut final_text = String::new();

    for iteration in 1..=config.max_iterations {
        debug!("🔄 Conversation iteration {iteration}");

        if let Some(logger) = audit_logger {
            logger.log_request(iteration, &messages)?;
        }

        let request = build_request(config, &messages, &tools);
        let events = client.send_streaming(&request).await?;
        let response = response_collector::collect_response(&events)?;

        accumulate_usage(&mut total_usage, &response.usage);

        if !response.text.is_empty() {
            final_text = response.text.clone();
        }

        if let Some(logger) = audit_logger {
            logger.log_response(
                iteration,
                &response.text,
                &response.tool_use_blocks,
                &response.usage,
            )?;
        }

        match response.stop_reason {
            Some(StopReason::ToolUse) => {
                if response.tool_use_blocks.is_empty() {
                    bail!("Claude returned stop_reason=tool_use but no tool_use blocks");
                }

                // Build assistant message echoing text + tool_use blocks
                let assistant_blocks = build_assistant_blocks(&response);
                messages.push(Message::assistant_blocks(assistant_blocks));

                // Execute each tool and collect results
                let tool_result_blocks =
                    execute_tools(executor, &response).await;

                if let Some(logger) = audit_logger {
                    logger.log_tool_results(iteration, &tool_result_blocks)?;
                }

                messages.push(Message::tool_results(tool_result_blocks));
            }
            Some(StopReason::EndTurn | StopReason::MaxTokens) | None => {
                info!(
                    "✅ Conversation complete after {iteration} iteration(s), {} tokens",
                    total_usage.total()
                );
                return Ok(ConversationResult {
                    final_text,
                    total_usage,
                    iterations: iteration,
                });
            }
        }
    }

    warn!(
        "⚠️ Conversation hit max iterations ({}), returning partial result",
        config.max_iterations
    );
    Ok(ConversationResult {
        final_text,
        total_usage,
        iterations: config.max_iterations,
    })
}

fn build_request(
    config: &ConversationLoopConfig,
    messages: &[Message],
    tools: &[ToolDefinition],
) -> MessagesRequest {
    MessagesRequest {
        model: config.model.clone(),
        max_tokens: config.max_tokens,
        messages: messages.to_vec(),
        system: config.system.clone(),
        stream: true,
        temperature: config.temperature,
        tools: tools.to_vec(),
    }
}

fn build_assistant_blocks(response: &CollectedResponse) -> Vec<RequestContentBlock> {
    let mut blocks = Vec::new();

    if !response.text.is_empty() {
        blocks.push(RequestContentBlock::Text {
            text: response.text.clone(),
        });
    }

    for tool_use in &response.tool_use_blocks {
        blocks.push(RequestContentBlock::ToolUse {
            id: tool_use.id.clone(),
            name: tool_use.name.clone(),
            input: tool_use.input.clone(),
        });
    }

    blocks
}

async fn execute_tools(
    executor: &dyn ToolExecutor,
    response: &CollectedResponse,
) -> Vec<RequestContentBlock> {
    let mut result_blocks = Vec::new();

    for tool_use in &response.tool_use_blocks {
        info!("🔧 Executing tool: {} ({})", tool_use.name, tool_use.id);

        let (content, is_error) = match executor
            .execute_tool(&tool_use.name, &tool_use.input)
            .await
        {
            Ok(result) => {
                if result.is_error {
                    warn!("⚠️ Tool {} returned error: {}", tool_use.name, result.content);
                }
                (result.content, result.is_error)
            }
            Err(err) => {
                warn!("❌ Tool {} execution failed: {err:#}", tool_use.name);
                (format!("Tool execution error: {err:#}"), true)
            }
        };

        result_blocks.push(RequestContentBlock::ToolResult {
            tool_use_id: tool_use.id.clone(),
            content,
            is_error: if is_error { Some(true) } else { None },
        });
    }

    result_blocks
}

fn accumulate_usage(total: &mut Usage, response_usage: &Usage) {
    total.input_tokens += response_usage.input_tokens;
    total.output_tokens += response_usage.output_tokens;
}

fn format_tool_names(tools: &[ToolDefinition]) -> String {
    tools
        .iter()
        .map(|t| t.name.as_str())
        .collect::<Vec<_>>()
        .join(", ")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn config_new_has_defaults() {
        let config = ConversationLoopConfig::new("claude-sonnet-4-20250514".to_string(), 4096);
        assert_eq!(config.model, "claude-sonnet-4-20250514");
        assert_eq!(config.max_tokens, 4096);
        assert_eq!(config.max_iterations, DEFAULT_MAX_ITERATIONS);
        assert!(config.system.is_none());
        assert!(config.temperature.is_none());
    }

    #[test]
    fn format_tool_names_empty() {
        assert_eq!(format_tool_names(&[]), "");
    }

    #[test]
    fn format_tool_names_multiple() {
        let tools = vec![
            ToolDefinition::builder()
                .name("create_agent".to_string())
                .input_schema(serde_json::json!({}))
                .build(),
            ToolDefinition::builder()
                .name("list_agents".to_string())
                .input_schema(serde_json::json!({}))
                .build(),
        ];
        assert_eq!(format_tool_names(&tools), "create_agent, list_agents");
    }

    #[test]
    fn accumulate_usage_adds_tokens() {
        let mut total = Usage::default();
        accumulate_usage(
            &mut total,
            &Usage {
                input_tokens: 100,
                output_tokens: 50,
            },
        );
        accumulate_usage(
            &mut total,
            &Usage {
                input_tokens: 200,
                output_tokens: 80,
            },
        );
        assert_eq!(total.input_tokens, 300);
        assert_eq!(total.output_tokens, 130);
    }

    #[test]
    fn build_assistant_blocks_text_only() {
        let response = CollectedResponse {
            text: "Hello".to_string(),
            tool_use_blocks: vec![],
            usage: Usage::default(),
            stop_reason: Some(StopReason::EndTurn),
        };
        let blocks = build_assistant_blocks(&response);
        assert_eq!(blocks.len(), 1);
        assert!(matches!(&blocks[0], RequestContentBlock::Text { text } if text == "Hello"));
    }

    #[test]
    fn build_assistant_blocks_with_tool_use() {
        use crate::claude_api::tool_use_block::ToolUseBlock;

        let response = CollectedResponse {
            text: "I'll create an agent.".to_string(),
            tool_use_blocks: vec![ToolUseBlock {
                id: "toolu_123".to_string(),
                name: "create_agent".to_string(),
                input: serde_json::json!({"type": "code-gen"}),
            }],
            usage: Usage::default(),
            stop_reason: Some(StopReason::ToolUse),
        };
        let blocks = build_assistant_blocks(&response);
        assert_eq!(blocks.len(), 2);
        assert!(matches!(&blocks[0], RequestContentBlock::Text { .. }));
        assert!(matches!(&blocks[1], RequestContentBlock::ToolUse { name, .. } if name == "create_agent"));
    }
}
