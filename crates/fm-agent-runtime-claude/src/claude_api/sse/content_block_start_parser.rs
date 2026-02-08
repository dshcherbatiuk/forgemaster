//! Parser for `content_block_start` SSE events.

use anyhow::Result;

use super::EventParser;
use super::SseEvent;

/// Parses `content_block_start` events.
///
/// Returns `ToolUseStart` for tool_use blocks (with id and name),
/// or `ContentBlockStart` for text blocks.
pub struct ContentBlockStartParser;

impl EventParser for ContentBlockStartParser {
    fn parse(&self, data: &str) -> Result<SseEvent> {
        let parsed: serde_json::Value = serde_json::from_str(data)?;
        let index = parsed["index"].as_u64().unwrap_or_default() as usize;

        let block_type = parsed["content_block"]["type"].as_str().unwrap_or("text");

        if block_type == "tool_use" {
            let id = parsed["content_block"]["id"]
                .as_str()
                .unwrap_or_default()
                .to_string();
            let name = parsed["content_block"]["name"]
                .as_str()
                .unwrap_or_default()
                .to_string();
            Ok(SseEvent::ToolUseStart { index, id, name })
        } else {
            Ok(SseEvent::ContentBlockStart { index })
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_text_block_start() {
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

    #[test]
    fn parses_tool_use_block_start() {
        let parser = ContentBlockStartParser;
        let data = r#"{"type":"content_block_start","index":1,"content_block":{"type":"tool_use","id":"toolu_01A09q90qw90lq917835lq9","name":"create_agent","input":{}}}"#;

        let event = parser.parse(data).expect("parse");
        if let SseEvent::ToolUseStart { index, id, name } = event {
            assert_eq!(index, 1);
            assert_eq!(id, "toolu_01A09q90qw90lq917835lq9");
            assert_eq!(name, "create_agent");
        } else {
            panic!("expected ToolUseStart, got {event:?}");
        }
    }

    #[test]
    fn text_block_without_explicit_type_defaults_to_content_block_start() {
        let parser = ContentBlockStartParser;
        let data = r#"{"index":0,"content_block":{}}"#;

        let event = parser.parse(data).expect("parse");
        assert!(matches!(event, SseEvent::ContentBlockStart { index: 0 }));
    }
}
