//! Parser for `content_block_start` SSE events.

use anyhow::Result;

use super::SseEvent;
use super::EventParser;

/// Parses `content_block_start` events containing the block index.
pub struct ContentBlockStartParser;

impl EventParser for ContentBlockStartParser {
    fn parse(&self, data: &str) -> Result<SseEvent> {
        let parsed: serde_json::Value = serde_json::from_str(data)?;
        let index = parsed["index"].as_u64().unwrap_or_default() as usize;
        Ok(SseEvent::ContentBlockStart { index })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_block_start() {
        let parser = ContentBlockStartParser;
        let data = r#"{"type":"content_block_start","index":0,"content_block":{"type":"text","text":""}}"#;

        let event = parser.parse(data).expect("parse");
        assert!(matches!(event, SseEvent::ContentBlockStart { index: 0 }));
    }

    #[test]
    fn parses_second_block() {
        let parser = ContentBlockStartParser;
        let data = r#"{"index":1}"#;

        let event = parser.parse(data).expect("parse");
        assert!(matches!(event, SseEvent::ContentBlockStart { index: 1 }));
    }
}
