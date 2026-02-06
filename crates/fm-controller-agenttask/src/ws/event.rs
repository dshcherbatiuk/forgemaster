//! Events sent from the controller to the UI via WebSocket.

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// Event pushed from the WebSocket server to connected UI clients.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, JsonSchema)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum WsEvent {
    /// Connection acknowledgment with assigned client ID.
    Connected {
        /// Assigned client ID.
        client_id: String,
    },

    /// Full data update pushed to the UI.
    Data {
        /// The data payload.
        data: serde_json::Value,
    },
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn connected_serializes_with_correct_tag() {
        let event = WsEvent::Connected {
            client_id: "abc-123".to_string(),
        };
        let serialized = serde_json::to_value(&event).unwrap();
        assert_eq!(serialized["type"], "connected");
        assert_eq!(serialized["client_id"], "abc-123");
    }

    #[test]
    fn connected_roundtrips_through_json() {
        let original = WsEvent::Connected {
            client_id: "abc-123".to_string(),
        };
        let json_str = serde_json::to_string(&original).unwrap();
        let deserialized: WsEvent = serde_json::from_str(&json_str).unwrap();
        assert_eq!(original, deserialized);
    }

    #[test]
    fn data_serializes_with_correct_tag() {
        let event = WsEvent::Data {
            data: serde_json::json!({"status": "Running"}),
        };
        let serialized = serde_json::to_value(&event).unwrap();
        assert_eq!(serialized["type"], "data");
        assert_eq!(serialized["data"]["status"], "Running");
    }

    #[test]
    fn data_roundtrips_through_json() {
        let original = WsEvent::Data {
            data: serde_json::json!({"status": "Ready", "tcp": {"value": "0.40"}}),
        };
        let json_str = serde_json::to_string(&original).unwrap();
        let deserialized: WsEvent = serde_json::from_str(&json_str).unwrap();
        assert_eq!(original, deserialized);
    }

    #[test]
    fn connected_deserializes_from_raw_json() {
        let json_str = r#"{"type":"connected","client_id":"xyz-789"}"#;
        let event: WsEvent = serde_json::from_str(json_str).unwrap();
        assert_eq!(
            event,
            WsEvent::Connected {
                client_id: "xyz-789".to_string()
            }
        );
    }
}
