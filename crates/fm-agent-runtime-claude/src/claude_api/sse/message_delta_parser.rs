//! Parser for `message_delta` SSE events.

use anyhow::Result;

use super::SseEvent;
use super::EventParser;
use crate::claude_api::response::{StopReason, Usage};

/// Parses `message_delta` events containing stop reason and final usage.
pub struct MessageDeltaParser;

impl EventParser for MessageDeltaParser {
    fn parse(&self, data: &str) -> Result<SseEvent> {
        let parsed: serde_json::Value = serde_json::from_str(data)?;

        let stop_reason: Option<StopReason> = parsed["delta"]["stop_reason"]
            .as_str()
            .and_then(|s| {
                serde_json::from_value(serde_json::Value::String(s.to_string())).ok()
            });

        let usage: Usage = serde_json::from_value(parsed["usage"].clone()).unwrap_or_default();

        Ok(SseEvent::MessageDelta { stop_reason, usage })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_end_turn() {
        let parser = MessageDeltaParser;
        let data = r#"{"type":"message_delta","delta":{"stop_reason":"end_turn"},"usage":{"output_tokens":42}}"#;

        let event = parser.parse(data).expect("parse");
        if let SseEvent::MessageDelta { stop_reason, usage } = event {
            assert_eq!(stop_reason, Some(StopReason::EndTurn));
            assert_eq!(usage.output_tokens, 42);
        } else {
            panic!("expected MessageDelta");
        }
    }

    #[test]
    fn parses_max_tokens() {
        let parser = MessageDeltaParser;
        let data = r#"{"delta":{"stop_reason":"max_tokens"},"usage":{"output_tokens":4096}}"#;

        let event = parser.parse(data).expect("parse");
        if let SseEvent::MessageDelta { stop_reason, .. } = event {
            assert_eq!(stop_reason, Some(StopReason::MaxTokens));
        } else {
            panic!("expected MessageDelta");
        }
    }

    #[test]
    fn parses_no_stop_reason() {
        let parser = MessageDeltaParser;
        let data = r#"{"delta":{},"usage":{"output_tokens":10}}"#;

        let event = parser.parse(data).expect("parse");
        if let SseEvent::MessageDelta { stop_reason, .. } = event {
            assert!(stop_reason.is_none());
        } else {
            panic!("expected MessageDelta");
        }
    }
}
