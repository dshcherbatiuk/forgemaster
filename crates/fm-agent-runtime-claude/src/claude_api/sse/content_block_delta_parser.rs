//! Parser for `content_block_delta` SSE events.

use anyhow::Result;

use super::EventParser;
use super::SseEvent;

/// Parses `content_block_delta` events.
///
/// Returns `ContentBlockDelta` for text deltas, or `InputJsonDelta` for
/// tool_use input JSON fragments.
pub struct ContentBlockDeltaParser;

impl EventParser for ContentBlockDeltaParser {
    fn parse(&self, data: &str) -> Result<SseEvent> {
        let parsed: serde_json::Value = serde_json::from_str(data)?;
        let index = parsed["index"].as_u64().unwrap_or_default() as usize;

        let delta_type = parsed["delta"]["type"].as_str().unwrap_or("text_delta");

        if delta_type == "input_json_delta" {
            let partial_json = parsed["delta"]["partial_json"]
                .as_str()
                .unwrap_or_default()
                .to_string();
            Ok(SseEvent::InputJsonDelta {
                index,
                partial_json,
            })
        } else {
            let text = parsed["delta"]["text"]
                .as_str()
                .unwrap_or_default()
                .to_string();
            Ok(SseEvent::ContentBlockDelta { index, text })
        }
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

    #[test]
    fn parses_input_json_delta() {
        let parser = ContentBlockDeltaParser;
        let data = r#"{"type":"content_block_delta","index":1,"delta":{"type":"input_json_delta","partial_json":"{\"agent_type\":"}}"#;

        let event = parser.parse(data).expect("parse");
        if let SseEvent::InputJsonDelta {
            index,
            partial_json,
        } = event
        {
            assert_eq!(index, 1);
            assert_eq!(partial_json, r#"{"agent_type":"#);
        } else {
            panic!("expected InputJsonDelta, got {event:?}");
        }
    }

    #[test]
    fn empty_input_json_delta() {
        let parser = ContentBlockDeltaParser;
        let data = r#"{"index":1,"delta":{"type":"input_json_delta"}}"#;

        let event = parser.parse(data).expect("parse");
        if let SseEvent::InputJsonDelta { partial_json, .. } = event {
            assert!(partial_json.is_empty());
        } else {
            panic!("expected InputJsonDelta");
        }
    }
}
