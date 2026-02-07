//! SSE (Server-Sent Events) parsing with strategy pattern.
//!
//! Each SSE event type has its own parser implementing `EventParser`.
//! The `SseDispatcher` routes raw events to the appropriate parser.

mod content_block_delta_parser;
mod content_block_start_parser;
mod content_block_stop_parser;
pub mod dispatcher;
mod error_parser;
mod message_delta_parser;
mod message_start_parser;

pub use dispatcher::SseDispatcher;

use anyhow::Result;

use crate::claude_api::response::{StopReason, Usage};

/// Trait for parsing a specific SSE event type from its JSON data.
pub trait EventParser: Send + Sync {
    /// Parses the JSON data payload into an `SseEvent`.
    fn parse(&self, data: &str) -> Result<SseEvent>;
}

/// A parsed SSE event from the Claude streaming API.
#[derive(Debug)]
pub enum SseEvent {
    /// Stream started, contains message ID and initial usage.
    MessageStart {
        /// Unique message identifier.
        message_id: String,
        /// Initial usage (input tokens).
        usage: Usage,
    },
    /// A new content block is starting.
    ContentBlockStart {
        /// Index of this block in the content array.
        index: usize,
    },
    /// Incremental text within a content block.
    ContentBlockDelta {
        /// Index of the content block.
        index: usize,
        /// New text fragment.
        text: String,
    },
    /// A content block has finished.
    ContentBlockStop {
        /// Index of the completed block.
        index: usize,
    },
    /// Top-level message metadata update (stop reason, final usage).
    MessageDelta {
        /// Why the model stopped (if finished).
        stop_reason: Option<StopReason>,
        /// Output token usage.
        usage: Usage,
    },
    /// Stream complete — no more events will follow.
    MessageStop,
    /// Heartbeat event.
    Ping,
    /// Error received during streaming.
    Error {
        /// Error type from the API.
        error_type: String,
        /// Error message.
        message: String,
    },
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn message_start_is_constructible() {
        let event = SseEvent::MessageStart {
            message_id: "msg_123".to_string(),
            usage: Usage {
                input_tokens: 10,
                output_tokens: 0,
            },
        };
        assert!(matches!(event, SseEvent::MessageStart { .. }));
    }

    #[test]
    fn content_block_delta_holds_text() {
        let event = SseEvent::ContentBlockDelta {
            index: 0,
            text: "Hello".to_string(),
        };
        if let SseEvent::ContentBlockDelta { index, text } = event {
            assert_eq!(index, 0);
            assert_eq!(text, "Hello");
        } else {
            panic!("expected ContentBlockDelta");
        }
    }

    #[test]
    fn message_delta_with_stop_reason() {
        let event = SseEvent::MessageDelta {
            stop_reason: Some(StopReason::EndTurn),
            usage: Usage {
                input_tokens: 0,
                output_tokens: 42,
            },
        };
        if let SseEvent::MessageDelta { stop_reason, usage } = event {
            assert_eq!(stop_reason, Some(StopReason::EndTurn));
            assert_eq!(usage.output_tokens, 42);
        } else {
            panic!("expected MessageDelta");
        }
    }

    #[test]
    fn error_event_holds_details() {
        let event = SseEvent::Error {
            error_type: "overloaded_error".to_string(),
            message: "Server is busy".to_string(),
        };
        if let SseEvent::Error {
            error_type,
            message,
        } = event
        {
            assert_eq!(error_type, "overloaded_error");
            assert_eq!(message, "Server is busy");
        } else {
            panic!("expected Error");
        }
    }
}
