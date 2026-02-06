//! Events sent from the controller to the UI via WebSocket.

use serde::{Deserialize, Serialize};
use smallvec::SmallVec;

/// Event pushed from the WebSocket server to connected UI clients.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
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

    /// A2UI schema pushed from the controller.
    Schema {
        /// Root component ID.
        root: String,
        /// A2UI component definitions.
        components: SmallVec<[serde_json::Value; 16]>,
        /// Data to populate the schema.
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
    fn schema_serializes_with_correct_tag() {
        let event = WsEvent::Schema {
            root: "task-status-card".to_string(),
            components: smallvec::smallvec![serde_json::json!({"id": "c1", "component": {"Text": {"text": {"literalString": "Hello"}}}})],
            data: serde_json::json!({"task": {"phase": "Running"}}),
        };
        let serialized = serde_json::to_value(&event).unwrap();
        assert_eq!(serialized["type"], "schema");
        assert_eq!(serialized["root"], "task-status-card");
        assert!(serialized["components"].is_array());
        assert_eq!(serialized["data"]["task"]["phase"], "Running");
    }

    #[test]
    fn schema_roundtrips_through_json() {
        let original = WsEvent::Schema {
            root: "root-1".to_string(),
            components: smallvec::smallvec![serde_json::json!({"id": "t1"})],
            data: serde_json::json!({"key": "value"}),
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
