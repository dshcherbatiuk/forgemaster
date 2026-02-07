//! Agent phase enumeration.

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// Represents the current phase of an Agent.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Serialize, Deserialize, JsonSchema)]
pub enum AgentPhase {
    /// Agent CR created, waiting for pod to be scheduled.
    #[default]
    Pending,
    /// Pod started, agent is processing.
    Running,
    /// Work completed successfully, output stored.
    Succeeded,
    /// Error occurred or timeout reached.
    Failed,
}

impl std::fmt::Display for AgentPhase {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Pending => write!(f, "Pending"),
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
        assert_eq!(AgentPhase::default(), AgentPhase::Pending);
    }

    #[test]
    fn display() {
        assert_eq!(AgentPhase::Pending.to_string(), "Pending");
        assert_eq!(AgentPhase::Running.to_string(), "Running");
        assert_eq!(AgentPhase::Succeeded.to_string(), "Succeeded");
        assert_eq!(AgentPhase::Failed.to_string(), "Failed");
    }

    #[test]
    fn serialization() {
        let json = serde_json::to_string(&AgentPhase::Running).unwrap_or_default();
        assert_eq!(json, "\"Running\"");
    }

    #[test]
    fn deserialization() {
        let phase: AgentPhase = serde_json::from_str("\"Failed\"").unwrap_or(AgentPhase::Pending);
        assert_eq!(phase, AgentPhase::Failed);
    }
}
