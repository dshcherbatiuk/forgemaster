//! Parser for `error` SSE events.

use anyhow::Result;

use super::SseEvent;
use super::EventParser;

/// Parses `error` events from the streaming API.
pub struct ErrorParser;

impl EventParser for ErrorParser {
    fn parse(&self, data: &str) -> Result<SseEvent> {
        let parsed: serde_json::Value = serde_json::from_str(data)?;
        let error_type = parsed["error"]["type"]
            .as_str()
            .unwrap_or("unknown")
            .to_string();
        let message = parsed["error"]["message"]
            .as_str()
            .unwrap_or("unknown error")
            .to_string();
        Ok(SseEvent::Error {
            error_type,
            message,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_overloaded_error() {
        let parser = ErrorParser;
        let data = r#"{"type":"error","error":{"type":"overloaded_error","message":"Overloaded"}}"#;

        let event = parser.parse(data).expect("parse");
        if let SseEvent::Error {
            error_type,
            message,
        } = event
        {
            assert_eq!(error_type, "overloaded_error");
            assert_eq!(message, "Overloaded");
        } else {
            panic!("expected Error");
        }
    }

    #[test]
    fn parses_missing_fields_with_defaults() {
        let parser = ErrorParser;
        let data = r#"{"error":{}}"#;

        let event = parser.parse(data).expect("parse");
        if let SseEvent::Error {
            error_type,
            message,
        } = event
        {
            assert_eq!(error_type, "unknown");
            assert_eq!(message, "unknown error");
        } else {
            panic!("expected Error");
        }
    }
}
