//! Events emitted by the controller to notify WebSocket clients.
//!
//! The broadcast channel carries [`TaskEvent`] variants for both
//! state changes and task deletions.

use crate::task_state_changed::TaskStateChanged;

/// Event sent through the broadcast channel from controller to WS broadcaster.
#[derive(Debug, Clone)]
pub enum TaskEvent {
    /// Task state changed (phase transition, live update).
    StateChanged(TaskStateChanged),

    /// Task was deleted from Kubernetes.
    Deleted {
        /// Name of the deleted task.
        task_name: String,
    },
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::agent_info::AgentInfoList;
    use crate::crd::AgentTaskPhase;

    fn sample_state_changed() -> TaskStateChanged {
        TaskStateChanged {
            task_name: "task-abc123".to_string(),
            namespace: "forgemaster-system".to_string(),
            description: "Build an API".to_string(),
            created_at: None,
            phase: AgentTaskPhase::Running,
            iteration: 1,
            error: 0.5,
            tests_total: 10,
            tests_passed: 5,
            agents: AgentInfoList::new(),
        }
    }

    #[test]
    fn state_changed_variant_contains_event() {
        let event = TaskEvent::StateChanged(sample_state_changed());
        if let TaskEvent::StateChanged(inner) = event {
            assert_eq!(inner.task_name, "task-abc123");
            assert_eq!(inner.phase, AgentTaskPhase::Running);
        } else {
            panic!("expected StateChanged variant");
        }
    }

    #[test]
    fn deleted_variant_contains_task_name() {
        let event = TaskEvent::Deleted {
            task_name: "task-xyz789".to_string(),
        };
        if let TaskEvent::Deleted { task_name } = event {
            assert_eq!(task_name, "task-xyz789");
        } else {
            panic!("expected Deleted variant");
        }
    }

    #[test]
    fn clone_preserves_variant() {
        let event = TaskEvent::Deleted {
            task_name: "task-clone".to_string(),
        };
        let cloned = event.clone();
        if let TaskEvent::Deleted { task_name } = cloned {
            assert_eq!(task_name, "task-clone");
        } else {
            panic!("expected Deleted variant after clone");
        }
    }
}
