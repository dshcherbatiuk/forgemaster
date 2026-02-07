//! Routes SSE event types to registered parsers using the strategy pattern.

use anyhow::Result;
use dashmap::DashMap;

use super::content_block_delta_parser::ContentBlockDeltaParser;
use super::content_block_start_parser::ContentBlockStartParser;
use super::content_block_stop_parser::ContentBlockStopParser;
use super::error_parser::ErrorParser;
use super::SseEvent;
use super::message_delta_parser::MessageDeltaParser;
use super::message_start_parser::MessageStartParser;
use super::EventParser;

/// Dispatches raw SSE event types to registered parsers.
pub struct SseDispatcher {
    parsers: DashMap<String, Box<dyn EventParser>>,
}

impl SseDispatcher {
    /// Creates a dispatcher with all standard Claude API event parsers registered.
    pub fn new() -> Self {
        let parsers: DashMap<String, Box<dyn EventParser>> = DashMap::new();

        parsers.insert("message_start".to_string(), Box::new(MessageStartParser));
        parsers.insert(
            "content_block_start".to_string(),
            Box::new(ContentBlockStartParser),
        );
        parsers.insert(
            "content_block_delta".to_string(),
            Box::new(ContentBlockDeltaParser),
        );
        parsers.insert(
            "content_block_stop".to_string(),
            Box::new(ContentBlockStopParser),
        );
        parsers.insert(
            "message_delta".to_string(),
            Box::new(MessageDeltaParser),
        );
        parsers.insert("error".to_string(), Box::new(ErrorParser));

        Self { parsers }
    }

    /// Parses an SSE event by dispatching to the registered parser.
    ///
    /// Handles `message_stop` and `ping` directly (no JSON parsing needed).
    pub fn parse(&self, event_type: &str, data: &str) -> Result<SseEvent> {
        match event_type {
            "message_stop" => return Ok(SseEvent::MessageStop),
            "ping" => return Ok(SseEvent::Ping),
            _ => {}
        }

        let parser = self
            .parsers
            .get(event_type)
            .ok_or_else(|| anyhow::anyhow!("unknown SSE event type: {event_type}"))?;

        parser.parse(data)
    }
}

impl Default for SseDispatcher {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::claude_api::response::StopReason;

    #[test]
    fn parses_message_start() {
        let dispatcher = SseDispatcher::new();
        let data = r#"{"type":"message_start","message":{"id":"msg_abc123","usage":{"input_tokens":25,"output_tokens":0}}}"#;

        let event = dispatcher.parse("message_start", data).expect("parse");
        assert!(matches!(event, SseEvent::MessageStart { .. }));
    }

    #[test]
    fn parses_content_block_delta() {
        let dispatcher = SseDispatcher::new();
        let data = r#"{"type":"content_block_delta","index":0,"delta":{"type":"text_delta","text":"Hello"}}"#;

        let event = dispatcher.parse("content_block_delta", data).expect("parse");
        if let SseEvent::ContentBlockDelta { text, .. } = event {
            assert_eq!(text, "Hello");
        } else {
            panic!("expected ContentBlockDelta");
        }
    }

    #[test]
    fn parses_message_stop() {
        let dispatcher = SseDispatcher::new();
        let event = dispatcher.parse("message_stop", "{}").expect("parse");
        assert!(matches!(event, SseEvent::MessageStop));
    }

    #[test]
    fn parses_ping() {
        let dispatcher = SseDispatcher::new();
        let event = dispatcher.parse("ping", "{}").expect("parse");
        assert!(matches!(event, SseEvent::Ping));
    }

    #[test]
    fn parses_message_delta() {
        let dispatcher = SseDispatcher::new();
        let data = r#"{"type":"message_delta","delta":{"stop_reason":"end_turn"},"usage":{"output_tokens":42}}"#;

        let event = dispatcher.parse("message_delta", data).expect("parse");
        if let SseEvent::MessageDelta { stop_reason, usage } = event {
            assert_eq!(stop_reason, Some(StopReason::EndTurn));
            assert_eq!(usage.output_tokens, 42);
        } else {
            panic!("expected MessageDelta");
        }
    }

    #[test]
    fn parses_error() {
        let dispatcher = SseDispatcher::new();
        let data = r#"{"type":"error","error":{"type":"overloaded_error","message":"Overloaded"}}"#;

        let event = dispatcher.parse("error", data).expect("parse");
        if let SseEvent::Error { error_type, .. } = event {
            assert_eq!(error_type, "overloaded_error");
        } else {
            panic!("expected Error");
        }
    }

    #[test]
    fn unknown_event_type_fails() {
        let dispatcher = SseDispatcher::new();
        let result = dispatcher.parse("unknown_type", "{}");
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("unknown_type"));
    }

    #[test]
    fn malformed_json_fails() {
        let dispatcher = SseDispatcher::new();
        let result = dispatcher.parse("message_start", "not json");
        assert!(result.is_err());
    }
}
