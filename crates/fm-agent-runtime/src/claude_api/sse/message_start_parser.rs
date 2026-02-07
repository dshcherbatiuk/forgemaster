//! Parser for `message_start` SSE events.

use anyhow::Result;

use super::SseEvent;
use super::EventParser;
use crate::claude_api::response::Usage;

/// Parses `message_start` events containing message ID and initial usage.
pub struct MessageStartParser;

impl EventParser for MessageStartParser {
    fn parse(&self, data: &str) -> Result<SseEvent> {
        let parsed: serde_json::Value = serde_json::from_str(data)?;
        let message = &parsed["message"];
        let message_id = message["id"]
            .as_str()
            .unwrap_or_default()
            .to_string();
        let usage: Usage = serde_json::from_value(message["usage"].clone()).unwrap_or_default();

        Ok(SseEvent::MessageStart { message_id, usage })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_full_message_start() {
        let parser = MessageStartParser;
        let data = r#"{"type":"message_start","message":{"id":"msg_abc123","type":"message","role":"assistant","content":[],"model":"claude-sonnet-4-20250514","usage":{"input_tokens":25,"output_tokens":0}}}"#;

        let event = parser.parse(data).expect("parse");
        if let SseEvent::MessageStart { message_id, usage } = event {
            assert_eq!(message_id, "msg_abc123");
            assert_eq!(usage.input_tokens, 25);
            assert_eq!(usage.output_tokens, 0);
        } else {
            panic!("expected MessageStart");
        }
    }

    #[test]
    fn parses_minimal_message_start() {
        let parser = MessageStartParser;
        let data = r#"{"message":{"id":"msg_1"}}"#;

        let event = parser.parse(data).expect("parse");
        if let SseEvent::MessageStart { message_id, usage } = event {
            assert_eq!(message_id, "msg_1");
            assert_eq!(usage.input_tokens, 0);
        } else {
            panic!("expected MessageStart");
        }
    }
}
