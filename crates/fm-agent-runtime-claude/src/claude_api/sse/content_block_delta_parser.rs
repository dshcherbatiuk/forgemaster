//! Parser for `content_block_delta` SSE events.

use anyhow::Result;

use super::SseEvent;
use super::EventParser;

/// Parses `content_block_delta` events containing incremental text.
pub struct ContentBlockDeltaParser;

impl EventParser for ContentBlockDeltaParser {
    fn parse(&self, data: &str) -> Result<SseEvent> {
        let parsed: serde_json::Value = serde_json::from_str(data)?;
        let index = parsed["index"].as_u64().unwrap_or_default() as usize;
        let text = parsed["delta"]["text"]
            .as_str()
            .unwrap_or_default()
            .to_string();
        Ok(SseEvent::ContentBlockDelta { index, text })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_text_delta() {
        let parser = ContentBlockDeltaParser;
        let data = r#"{"type":"content_block_delta","index":0,"delta":{"type":"text_delta","text":"Hello"}}"#;

        let event = parser.parse(data).expect("parse");
        if let SseEvent::ContentBlockDelta { index, text } = event {
            assert_eq!(index, 0);
            assert_eq!(text, "Hello");
        } else {
            panic!("expected ContentBlockDelta");
        }
    }

    #[test]
    fn empty_text_delta() {
        let parser = ContentBlockDeltaParser;
        let data = r#"{"index":0,"delta":{}}"#;

        let event = parser.parse(data).expect("parse");
        if let SseEvent::ContentBlockDelta { text, .. } = event {
            assert!(text.is_empty());
        } else {
            panic!("expected ContentBlockDelta");
        }
    }
}
