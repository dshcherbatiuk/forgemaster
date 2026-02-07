//! Agent status.

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use typed_builder::TypedBuilder;

use super::phase::AgentPhase;
use crate::crd::{Condition, OutputRef, PodRef};

/// Status of an Agent CR.
#[derive(
    Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize, JsonSchema, TypedBuilder,
)]
#[serde(rename_all = "camelCase")]
pub struct AgentStatus {
    /// Current phase.
    #[builder(default)]
    #[serde(default)]
    pub phase: AgentPhase,

    /// When the agent started (ISO 8601).
    #[builder(default, setter(strip_option))]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub start_time: Option<String>,

    /// When the agent completed (ISO 8601).
    #[builder(default, setter(strip_option))]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub completion_time: Option<String>,

    /// Total tokens consumed.
    #[builder(default)]
    #[serde(default)]
    pub tokens_used: i64,

    /// Iterations this agent participated in.
    #[builder(default)]
    #[serde(default)]
    pub iterations_completed: i32,

    /// Reference to the running Pod.
    #[builder(default, setter(strip_option))]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pod_ref: Option<PodRef>,

    /// Reference to agent output ConfigMap.
    #[builder(default, setter(strip_option))]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub output: Option<OutputRef>,

    /// Status conditions.
    #[builder(default)]
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub conditions: Vec<Condition>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_status() {
        let status = AgentStatus::default();
        assert_eq!(status.phase, AgentPhase::Pending);
        assert_eq!(status.tokens_used, 0);
        assert_eq!(status.iterations_completed, 0);
        assert!(status.pod_ref.is_none());
        assert!(status.output.is_none());
        assert!(status.conditions.is_empty());
    }

    #[test]
    fn build_with_pod_ref() {
        let status = AgentStatus::builder()
            .phase(AgentPhase::Running)
            .start_time("2026-02-06T12:00:00Z".to_string())
            .pod_ref(
                PodRef::builder()
                    .name("agent-pod".to_string())
                    .uid("uid-123".to_string())
                    .build(),
            )
            .build();

        assert_eq!(status.phase, AgentPhase::Running);
        assert!(status.pod_ref.is_some());
        assert!(status.start_time.is_some());
    }

    #[test]
    fn serialization() {
        let status = AgentStatus::builder()
            .phase(AgentPhase::Succeeded)
            .tokens_used(5000)
            .iterations_completed(3)
            .build();

        let json = serde_json::to_value(&status).unwrap_or_default();
        assert_eq!(json["phase"], "Succeeded");
        assert_eq!(json["tokensUsed"], 5000);
        assert_eq!(json["iterationsCompleted"], 3);
    }

    #[test]
    fn serialization_skips_empty() {
        let status = AgentStatus::default();
        let json = serde_json::to_value(&status).unwrap_or_default();
        assert!(json.get("podRef").is_none());
        assert!(json.get("output").is_none());
        assert!(json.get("conditions").is_none());
    }

    #[test]
    fn deserialization() {
        let json = r#"{
            "phase": "Failed",
            "tokensUsed": 1234,
            "iterationsCompleted": 1
        }"#;

        let status: AgentStatus = serde_json::from_str(json).unwrap_or_default();
        assert_eq!(status.phase, AgentPhase::Failed);
        assert_eq!(status.tokens_used, 1234);
    }
}
