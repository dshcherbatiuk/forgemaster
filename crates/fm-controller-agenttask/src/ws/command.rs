//! Commands sent from the UI to the controller via WebSocket.

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// Command sent from a UI client to the WebSocket server.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, JsonSchema)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum WsCommand {
    /// Initial connection handshake.
    Connect,

    /// Submit a new task for agent processing.
    SubmitTask {
        /// Task description from the user.
        description: String,
    },
}

#[cfg(test)]
mod tests {
    use super::*;

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
    fn submit_task_serializes_with_correct_tag() {
        let command = WsCommand::SubmitTask {
            description: "Build an API".to_string(),
        };
        let serialized = serde_json::to_value(&command).unwrap();
        assert_eq!(serialized["type"], "submit_task");
        assert_eq!(serialized["description"], "Build an API");
    }

    #[test]
    fn submit_task_deserializes_from_json() {
        let json_str = r#"{"type":"submit_task","description":"Create e-commerce API"}"#;
        let command: WsCommand = serde_json::from_str(json_str).unwrap();
        assert_eq!(
            command,
            WsCommand::SubmitTask {
                description: "Create e-commerce API".to_string(),
            }
        );
    }

    #[test]
    fn submit_task_roundtrips_through_json() {
        let original = WsCommand::SubmitTask {
            description: "Create e-commerce API".to_string(),
        };
        let json_str = serde_json::to_string(&original).unwrap();
        let deserialized: WsCommand = serde_json::from_str(&json_str).unwrap();
        assert_eq!(original, deserialized);
    }
}
