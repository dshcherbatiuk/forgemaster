//! Kubernetes-style status condition.

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use typed_builder::TypedBuilder;

/// Kubernetes-style status condition for an agent.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema, TypedBuilder)]
#[serde(rename_all = "camelCase")]
pub struct Condition {
    /// Condition type (e.g., "Ready", "PodScheduled").
    #[serde(rename = "type")]
    pub condition_type: String,

    /// Condition status: "True", "False", or "Unknown".
    pub status: String,

    /// Machine-readable reason for the condition.
    pub reason: String,

    /// Human-readable message.
    pub message: String,

    /// Last time the condition transitioned.
    pub last_transition_time: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn build() {
        let condition = Condition::builder()
            .condition_type("Ready".to_string())
            .status("True".to_string())
            .reason("PodRunning".to_string())
            .message("Agent pod is running".to_string())
            .last_transition_time("2026-02-06T12:00:00Z".to_string())
            .build();

        assert_eq!(condition.condition_type, "Ready");
        assert_eq!(condition.status, "True");
    }

    #[test]
    fn serialization_renames_type() {
        let condition = Condition::builder()
            .condition_type("Ready".to_string())
            .status("True".to_string())
            .reason("PodRunning".to_string())
            .message("OK".to_string())
            .last_transition_time("2026-02-06T12:00:00Z".to_string())
            .build();

        let json = serde_json::to_value(&condition).unwrap_or_default();
        assert_eq!(json["type"], "Ready");
        assert!(json.get("conditionType").is_none());
        assert_eq!(json["lastTransitionTime"], "2026-02-06T12:00:00Z");
    }

    #[test]
    fn deserialization() {
        let json = r#"{
            "type": "Failed",
            "status": "True",
            "reason": "Timeout",
            "message": "Agent timed out",
            "lastTransitionTime": "2026-02-06T13:00:00Z"
        }"#;

        let condition: Condition = serde_json::from_str(json).unwrap_or_else(|_| {
            Condition::builder()
                .condition_type("fallback".to_string())
                .status("Unknown".to_string())
                .reason("Error".to_string())
                .message("Parse error".to_string())
                .last_transition_time("".to_string())
                .build()
        });

        assert_eq!(condition.condition_type, "Failed");
        assert_eq!(condition.reason, "Timeout");
    }
}
