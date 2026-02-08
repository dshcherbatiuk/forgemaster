//! Collects structured response data from Claude streaming SSE events.
//!
//! Processes `Vec<SseEvent>` into a `CollectedResponse` containing
//! text blocks, tool_use blocks, usage, and stop reason.

use anyhow::{Result, bail};

use super::response::{StopReason, Usage};
use super::sse::SseEvent;
use super::tool_use_block::ToolUseBlock;

/// Structured result from a streaming Claude API response.
#[derive(Debug)]
pub struct CollectedResponse {
    /// Accumulated text from text content blocks.
    pub text: String,
    /// Tool use blocks (populated when stop_reason = ToolUse).
    pub tool_use_blocks: Vec<ToolUseBlock>,
    /// Combined token usage (input + output).
    pub usage: Usage,
    /// Why the model stopped generating.
    pub stop_reason: Option<StopReason>,
}

/// Tracks in-progress tool_use block state during streaming.
struct ToolUseBuilder {
    id: String,
    name: String,
    json_parts: String,
}

/// Collects structured response data from SSE events.
///
/// Handles both text content blocks and tool_use content blocks,
/// accumulating partial JSON input for tool calls.
pub fn collect_response(events: &[SseEvent]) -> Result<CollectedResponse> {
    let mut text = String::new();
    let mut tool_use_blocks = Vec::new();
    let mut combined_usage = Usage::default();
    let mut stop_reason = None;

    // Track active tool_use builders by block index.
    let mut active_tool_uses: Vec<(usize, ToolUseBuilder)> = Vec::new();

    for event in events {
        match event {
            SseEvent::MessageStart { usage, .. } => {
                combined_usage.input_tokens += usage.input_tokens;
                combined_usage.output_tokens += usage.output_tokens;
            }
            SseEvent::ToolUseStart { index, id, name } => {
                active_tool_uses.push((
                    *index,
                    ToolUseBuilder {
                        id: id.clone(),
                        name: name.clone(),
                        json_parts: String::new(),
                    },
                ));
            }
            SseEvent::InputJsonDelta {
                index,
                partial_json,
            } => {
                if let Some((_, builder)) =
                    active_tool_uses.iter_mut().find(|(i, _)| i == index)
                {
                    builder.json_parts.push_str(partial_json);
                }
            }
            SseEvent::ContentBlockDelta { text: delta, .. } => {
                text.push_str(delta);
            }
            SseEvent::ContentBlockStop { index } => {
                if let Some(pos) = active_tool_uses.iter().position(|(i, _)| i == index) {
                    let (_, builder) = active_tool_uses.remove(pos);
                    let input: serde_json::Value = if builder.json_parts.is_empty() {
                        serde_json::Value::Object(serde_json::Map::new())
                    } else {
                        serde_json::from_str(&builder.json_parts)?
                    };
                    tool_use_blocks.push(ToolUseBlock {
                        id: builder.id,
                        name: builder.name,
                        input,
                    });
                }
            }
            SseEvent::MessageDelta {
                stop_reason: sr,
                usage,
            } => {
                stop_reason = *sr;
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

    Ok(CollectedResponse {
        text,
        tool_use_blocks,
        usage: combined_usage,
        stop_reason,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_events_returns_empty_response() {
        let response = collect_response(&[]).unwrap();
        assert!(response.text.is_empty());
        assert!(response.tool_use_blocks.is_empty());
        assert_eq!(response.usage.input_tokens, 0);
        assert!(response.stop_reason.is_none());
    }

    #[test]
    fn collects_text_from_deltas() {
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
        ];

        let response = collect_response(&events).unwrap();
        assert_eq!(response.text, "Hello World");
        assert_eq!(response.usage.input_tokens, 100);
        assert_eq!(response.usage.output_tokens, 25);
        assert_eq!(response.stop_reason, Some(StopReason::EndTurn));
    }

    #[test]
    fn collects_tool_use_block() {
        let events = vec![
            SseEvent::ContentBlockStart { index: 0 },
            SseEvent::ContentBlockDelta {
                index: 0,
                text: "I'll create an agent.".to_string(),
            },
            SseEvent::ContentBlockStop { index: 0 },
            SseEvent::ToolUseStart {
                index: 1,
                id: "toolu_123".to_string(),
                name: "create_agent".to_string(),
            },
            SseEvent::InputJsonDelta {
                index: 1,
                partial_json: r#"{"agent"#.to_string(),
            },
            SseEvent::InputJsonDelta {
                index: 1,
                partial_json: r#"_type":"code-gen"}"#.to_string(),
            },
            SseEvent::ContentBlockStop { index: 1 },
            SseEvent::MessageDelta {
                stop_reason: Some(StopReason::ToolUse),
                usage: Usage {
                    input_tokens: 0,
                    output_tokens: 50,
                },
            },
        ];

        let response = collect_response(&events).unwrap();
        assert_eq!(response.text, "I'll create an agent.");
        assert_eq!(response.tool_use_blocks.len(), 1);
        assert_eq!(response.tool_use_blocks[0].id, "toolu_123");
        assert_eq!(response.tool_use_blocks[0].name, "create_agent");
        assert_eq!(
            response.tool_use_blocks[0].input["agent_type"],
            "code-gen"
        );
        assert_eq!(response.stop_reason, Some(StopReason::ToolUse));
    }

    #[test]
    fn tool_use_with_empty_input() {
        let events = vec![
            SseEvent::ToolUseStart {
                index: 0,
                id: "toolu_456".to_string(),
                name: "list_agents".to_string(),
            },
            SseEvent::ContentBlockStop { index: 0 },
        ];

        let response = collect_response(&events).unwrap();
        assert_eq!(response.tool_use_blocks.len(), 1);
        assert_eq!(response.tool_use_blocks[0].name, "list_agents");
        assert!(response.tool_use_blocks[0].input.is_object());
    }

    #[test]
    fn error_event_fails() {
        let events = vec![SseEvent::Error {
            error_type: "overloaded_error".to_string(),
            message: "Server busy".to_string(),
        }];

        let result = collect_response(&events);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("overloaded_error"));
    }

    #[test]
    fn ping_events_ignored() {
        let events = vec![
            SseEvent::Ping,
            SseEvent::ContentBlockDelta {
                index: 0,
                text: "data".to_string(),
            },
            SseEvent::Ping,
        ];

        let response = collect_response(&events).unwrap();
        assert_eq!(response.text, "data");
    }

    #[test]
    fn multiple_tool_use_blocks() {
        let events = vec![
            SseEvent::ToolUseStart {
                index: 0,
                id: "toolu_1".to_string(),
                name: "create_agent".to_string(),
            },
            SseEvent::InputJsonDelta {
                index: 0,
                partial_json: r#"{"type":"code-gen"}"#.to_string(),
            },
            SseEvent::ContentBlockStop { index: 0 },
            SseEvent::ToolUseStart {
                index: 1,
                id: "toolu_2".to_string(),
                name: "create_agent".to_string(),
            },
            SseEvent::InputJsonDelta {
                index: 1,
                partial_json: r#"{"type":"reviewer"}"#.to_string(),
            },
            SseEvent::ContentBlockStop { index: 1 },
        ];

        let response = collect_response(&events).unwrap();
        assert_eq!(response.tool_use_blocks.len(), 2);
        assert_eq!(response.tool_use_blocks[0].id, "toolu_1");
        assert_eq!(response.tool_use_blocks[1].id, "toolu_2");
    }
}
