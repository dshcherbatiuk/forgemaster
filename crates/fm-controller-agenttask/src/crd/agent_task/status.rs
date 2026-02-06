//! AgentTask status struct.

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use typed_builder::TypedBuilder;

use super::AgentTaskPhase;
use crate::crd::PendingClarification;

/// Status of an AgentTask.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema, TypedBuilder)]
#[serde(rename_all = "camelCase")]
pub struct AgentTaskStatus {
    /// Current phase.
    #[builder(default)]
    #[serde(default)]
    pub phase: AgentTaskPhase,

    /// Questions that need answers.
    #[builder(default)]
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub pending_clarifications: Vec<PendingClarification>,

    /// Current iteration number.
    #[builder(default)]
    #[serde(default)]
    pub iteration: i32,

    /// Current error signal (0.0 - 1.0).
    #[builder(default = 1.0)]
    #[serde(default = "default_error")]
    pub error: f64,

    /// Total number of tests.
    #[builder(default)]
    #[serde(default)]
    pub tests_total: i32,

    /// Number of tests passed.
    #[builder(default)]
    #[serde(default)]
    pub tests_passed: i32,

    /// Task start time (ISO 8601 format).
    #[builder(default, setter(strip_option))]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub start_time: Option<String>,

    /// Task completion time (ISO 8601 format).
    #[builder(default, setter(strip_option))]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub completion_time: Option<String>,
}

fn default_error() -> f64 {
    1.0
}

impl Default for AgentTaskStatus {
    fn default() -> Self {
        Self {
            phase: AgentTaskPhase::default(),
            pending_clarifications: Vec::new(),
            iteration: 0,
            error: 1.0,
            tests_total: 0,
            tests_passed: 0,
            start_time: None,
            completion_time: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_agent_task_status() {
        let status = AgentTaskStatus::default();
        assert_eq!(status.phase, AgentTaskPhase::Pending);
        assert_eq!(status.iteration, 0);
        assert!((status.error - 1.0).abs() < f64::EPSILON);
        assert!(status.pending_clarifications.is_empty());
    }

    #[test]
    fn build_agent_task_status() {
        let status = AgentTaskStatus::builder()
            .phase(AgentTaskPhase::Running)
            .iteration(3)
            .error(0.25)
            .tests_total(12)
            .tests_passed(9)
            .build();

        assert_eq!(status.phase, AgentTaskPhase::Running);
        assert_eq!(status.iteration, 3);
        assert!((status.error - 0.25).abs() < f64::EPSILON);
        assert_eq!(status.tests_total, 12);
        assert_eq!(status.tests_passed, 9);
    }

    #[test]
    fn agent_task_status_serialization() {
        let status = AgentTaskStatus::builder()
            .phase(AgentTaskPhase::Clarifying)
            .build();

        let json = serde_json::to_string(&status).expect("serialize");
        assert!(json.contains("\"phase\":\"Clarifying\""));
    }

    #[test]
    fn agent_task_status_deserialization() {
        let json = r#"{
            "phase": "Running",
            "iteration": 5,
            "error": 0.1,
            "testsTotal": 10,
            "testsPassed": 9
        }"#;

        let status: AgentTaskStatus = serde_json::from_str(json).expect("deserialize");
        assert_eq!(status.phase, AgentTaskPhase::Running);
        assert_eq!(status.iteration, 5);
        assert!((status.error - 0.1).abs() < f64::EPSILON);
    }
}
