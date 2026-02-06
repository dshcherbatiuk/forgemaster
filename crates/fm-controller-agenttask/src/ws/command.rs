//! Commands sent from the UI to the controller via WebSocket.

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// Command sent from a UI client to the WebSocket server.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, JsonSchema)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum WsCommand {
    /// Initial connection handshake.
    Connect,

    /// User action from the UI (button click, form submit, navigation).
    Action {
        /// Identifies the action (e.g. "submitTask", "navigate").
        action_id: String,
        /// Action payload with context-specific data.
        data: serde_json::Value,
    },
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn connect_serializes_with_correct_tag() {
        let command = WsCommand::Connect;
        let serialized = serde_json::to_value(&command).unwrap();
        assert_eq!(serialized["type"], "connect");
    }

    #[test]
    fn connect_deserializes_from_json() {
        let json_str = r#"{"type":"connect"}"#;
        let command: WsCommand = serde_json::from_str(json_str).unwrap();
        assert_eq!(command, WsCommand::Connect);
    }

    #[test]
    fn connect_roundtrips_through_json() {
        let original = WsCommand::Connect;
        let json_str = serde_json::to_string(&original).unwrap();
        let deserialized: WsCommand = serde_json::from_str(&json_str).unwrap();
        assert_eq!(original, deserialized);
    }

    #[test]
    fn action_serializes_with_correct_tag() {
        let command = WsCommand::Action {
            action_id: "submitTask".to_string(),
            data: json!({"description": "Build an API"}),
        };
        let serialized = serde_json::to_value(&command).unwrap();
        assert_eq!(serialized["type"], "action");
        assert_eq!(serialized["action_id"], "submitTask");
        assert_eq!(serialized["data"]["description"], "Build an API");
    }

    #[test]
    fn action_deserializes_from_json() {
        let json_str = r#"{"type":"action","action_id":"navigate","data":{"view":"progress"}}"#;
        let command: WsCommand = serde_json::from_str(json_str).unwrap();
        assert_eq!(
            command,
            WsCommand::Action {
                action_id: "navigate".to_string(),
                data: json!({"view": "progress"}),
            }
        );
    }

    #[test]
    fn action_roundtrips_through_json() {
        let original = WsCommand::Action {
            action_id: "submitTask".to_string(),
            data: json!({"description": "Create e-commerce API"}),
        };
        let json_str = serde_json::to_string(&original).unwrap();
        let deserialized: WsCommand = serde_json::from_str(&json_str).unwrap();
        assert_eq!(original, deserialized);
    }
}
