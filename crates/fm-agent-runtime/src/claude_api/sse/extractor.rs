//! Extracts raw SSE event-type and data pairs from text.

/// Extracts SSE event-type and data pairs from raw text lines.
///
/// Each SSE event consists of an `event:` line followed by a `data:` line.
/// Returns pairs of (event_type, data) ready for parsing.
pub fn extract_events(text: &str) -> Vec<(String, String)> {
    let mut events = Vec::new();
    let mut current_event_type: Option<String> = None;

    for line in text.lines() {
        if let Some(stripped) = line.strip_prefix("event: ") {
            current_event_type = Some(stripped.trim().to_string());
        } else if let Some(stripped) = line.strip_prefix("data: ") {
            if let Some(event_type) = current_event_type.take() {
                events.push((event_type, stripped.trim().to_string()));
            }
        }
    }

    events
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn extracts_multiple_events() {
        let text = "\
event: message_start\n\
data: {\"type\":\"message_start\",\"message\":{\"id\":\"msg_1\"}}\n\
\n\
event: content_block_delta\n\
data: {\"type\":\"content_block_delta\",\"index\":0,\"delta\":{\"type\":\"text_delta\",\"text\":\"Hi\"}}\n\
\n\
event: message_stop\n\
data: {}\n\
";

        let events = extract_events(text);
        assert_eq!(events.len(), 3);
        assert_eq!(events[0].0, "message_start");
        assert_eq!(events[1].0, "content_block_delta");
        assert_eq!(events[2].0, "message_stop");
    }

    #[test]
    fn skips_data_without_event() {
        let text = "data: {\"orphaned\": true}\n";
        let events = extract_events(text);
        assert!(events.is_empty());
    }

    #[test]
    fn handles_ping() {
        let text = "event: ping\ndata: {}\n";
        let events = extract_events(text);
        assert_eq!(events.len(), 1);
        assert_eq!(events[0].0, "ping");
    }

    #[test]
    fn empty_text_returns_empty() {
        let events = extract_events("");
        assert!(events.is_empty());
    }

    #[test]
    fn trims_whitespace() {
        let text = "event:  message_stop \ndata:  {} \n";
        let events = extract_events(text);
        assert_eq!(events.len(), 1);
        assert_eq!(events[0].0, "message_stop");
        assert_eq!(events[0].1, "{}");
    }
}
