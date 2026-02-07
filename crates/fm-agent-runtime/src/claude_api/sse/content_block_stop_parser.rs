//! Parser for `content_block_stop` SSE events.

use anyhow::Result;

use super::SseEvent;
use super::EventParser;

/// Parses `content_block_stop` events marking block completion.
pub struct ContentBlockStopParser;

impl EventParser for ContentBlockStopParser {
    fn parse(&self, data: &str) -> Result<SseEvent> {
        let parsed: serde_json::Value = serde_json::from_str(data)?;
        let index = parsed["index"].as_u64().unwrap_or_default() as usize;
        Ok(SseEvent::ContentBlockStop { index })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_block_stop() {
        let parser = ContentBlockStopParser;
        let data = r#"{"type":"content_block_stop","index":0}"#;

        let event = parser.parse(data).expect("parse");
        assert!(matches!(event, SseEvent::ContentBlockStop { index: 0 }));
    }
}
