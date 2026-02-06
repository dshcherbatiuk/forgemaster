//! AgentTask phase enumeration.

use std::hash::Hash;

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// Represents the current phase of an AgentTask.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Serialize, Deserialize, JsonSchema)]
pub enum AgentTaskPhase {
    /// Task is waiting to be processed.
    #[default]
    Pending,
    /// Task is waiting for clarification.
    Clarifying,
    /// Task is currently being executed.
    Running,
    /// Task completed successfully.
    Succeeded,
    /// Task failed after exhausting retries.
    Failed,
}

impl std::fmt::Display for AgentTaskPhase {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Pending => write!(f, "Pending"),
            Self::Clarifying => write!(f, "Clarifying"),
            Self::Running => write!(f, "Running"),
            Self::Succeeded => write!(f, "Succeeded"),
            Self::Failed => write!(f, "Failed"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_phase_is_pending() {
        let phase = AgentTaskPhase::default();
        assert_eq!(phase, AgentTaskPhase::Pending);
    }

    #[test]
    fn phase_display() {
        assert_eq!(AgentTaskPhase::Pending.to_string(), "Pending");
        assert_eq!(AgentTaskPhase::Clarifying.to_string(), "Clarifying");
        assert_eq!(AgentTaskPhase::Running.to_string(), "Running");
        assert_eq!(AgentTaskPhase::Succeeded.to_string(), "Succeeded");
        assert_eq!(AgentTaskPhase::Failed.to_string(), "Failed");
    }

    #[test]
    fn phase_serialization() {
        let phase = AgentTaskPhase::Running;
        let json = serde_json::to_string(&phase).expect("serialize");
        assert_eq!(json, "\"Running\"");
    }

    #[test]
    fn phase_deserialization() {
        let phase: AgentTaskPhase = serde_json::from_str("\"Clarifying\"").expect("deserialize");
        assert_eq!(phase, AgentTaskPhase::Clarifying);
    }
}
