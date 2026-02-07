//! Event emitted when the reconciler transitions a task phase.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::agent_info::AgentInfoList;
use crate::crd::AgentTaskPhase;

/// Emitted by the controller when a task state changes.
/// Consumed by the WS broadcaster to push state to connected UI clients.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskStateChanged {
    /// Task CR name.
    pub task_name: String,
    /// Namespace where the task lives.
    pub namespace: String,
    /// User-provided task description.
    pub description: String,
    /// When the task was created.
    pub created_at: Option<DateTime<Utc>>,
    /// Current task phase.
    pub phase: AgentTaskPhase,
    /// Current iteration number.
    pub iteration: i32,
    /// TCP error signal (0.0 - 1.0).
    pub error: f64,
    /// Total number of tests.
    pub tests_total: i32,
    /// Number of passing tests.
    pub tests_passed: i32,
    /// Agents working on this task.
    pub agents: AgentInfoList,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_event() -> TaskStateChanged {
        TaskStateChanged {
            task_name: "task-abc12345".to_string(),
            namespace: "forgemaster-system".to_string(),
            description: "Build a REST API".to_string(),
            created_at: Some(Utc::now()),
            phase: AgentTaskPhase::Running,
            iteration: 2,
            error: 0.4,
            tests_total: 5,
            tests_passed: 3,
            agents: AgentInfoList::new(),
        }
    }

    #[test]
    fn serialization_roundtrip() {
        let event = sample_event();
        let json = serde_json::to_string(&event).unwrap();
        let deserialized: TaskStateChanged = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.task_name, "task-abc12345");
        assert_eq!(deserialized.phase, AgentTaskPhase::Running);
        assert_eq!(deserialized.iteration, 2);
        assert!((deserialized.error - 0.4).abs() < f64::EPSILON);
    }

    #[test]
    fn json_contains_all_fields() {
        let event = sample_event();
        let json = serde_json::to_value(&event).unwrap();
        assert!(json.get("task_name").is_some());
        assert!(json.get("namespace").is_some());
        assert!(json.get("description").is_some());
        assert!(json.get("created_at").is_some());
        assert!(json.get("phase").is_some());
        assert!(json.get("iteration").is_some());
        assert!(json.get("error").is_some());
        assert!(json.get("tests_total").is_some());
        assert!(json.get("tests_passed").is_some());
        assert!(json.get("agents").is_some());
    }

    #[test]
    fn clone_produces_equal_event() {
        let event = sample_event();
        let cloned = event.clone();
        assert_eq!(cloned.task_name, event.task_name);
        assert_eq!(cloned.phase, event.phase);
    }
}
