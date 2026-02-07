//! Lightweight agent summary for the task status UI.

use serde::{Deserialize, Serialize};
use smallvec::SmallVec;

/// Stack-allocated list of agent summaries.
/// Inline capacity of 8 covers the typical agent set
/// (orchestrator + test-generator + code-generator + test-runner + feedback + reviewer + extras).
pub type AgentInfoList = SmallVec<[AgentInfo; 8]>;

/// Minimal agent data needed for the task status display.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AgentInfo {
    /// Agent CR name (e.g. "orchestrator-task-abc123").
    pub name: String,

    /// Agent type from spec (e.g. "orchestrator", "code-generator").
    pub agent_type: String,

    /// Current phase as string (e.g. "Pending", "Running", "Succeeded", "Failed").
    pub phase: String,
}

#[cfg(test)]
mod tests {
    use super::*;
    use smallvec::smallvec;

    fn sample_agent() -> AgentInfo {
        AgentInfo {
            name: "orchestrator-task-abc123".to_string(),
            agent_type: "orchestrator".to_string(),
            phase: "Running".to_string(),
        }
    }

    #[test]
    fn serialization_roundtrip() {
        let agent = sample_agent();
        let json = serde_json::to_string(&agent).unwrap();
        let deserialized: AgentInfo = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized, agent);
    }

    #[test]
    fn json_contains_all_fields() {
        let agent = sample_agent();
        let json = serde_json::to_value(&agent).unwrap();
        assert_eq!(json["name"], "orchestrator-task-abc123");
        assert_eq!(json["agent_type"], "orchestrator");
        assert_eq!(json["phase"], "Running");
    }

    #[test]
    fn clone_preserves_fields() {
        let agent = sample_agent();
        let cloned = agent.clone();
        assert_eq!(cloned, agent);
    }

    #[test]
    fn agent_info_list_inline_for_typical_counts() {
        let agents: AgentInfoList = smallvec![
            sample_agent(),
            AgentInfo {
                name: "code-gen-task-abc123".to_string(),
                agent_type: "code-generator".to_string(),
                phase: "Pending".to_string(),
            },
            AgentInfo {
                name: "test-gen-task-abc123".to_string(),
                agent_type: "test-generator".to_string(),
                phase: "Succeeded".to_string(),
            },
            AgentInfo {
                name: "test-runner-task-abc123".to_string(),
                agent_type: "test-runner".to_string(),
                phase: "Pending".to_string(),
            },
            AgentInfo {
                name: "feedback-task-abc123".to_string(),
                agent_type: "feedback".to_string(),
                phase: "Pending".to_string(),
            },
        ];
        assert_eq!(agents.len(), 5);
        assert!(!agents.spilled());
    }

    #[test]
    fn agent_info_list_empty() {
        let agents = AgentInfoList::new();
        assert!(agents.is_empty());
    }

    #[test]
    fn serialize_agent_list() {
        let agents: AgentInfoList = smallvec![sample_agent()];
        let json = serde_json::to_value(&agents).unwrap();
        assert!(json.is_array());
        assert_eq!(json.as_array().unwrap().len(), 1);
        assert_eq!(json[0]["name"], "orchestrator-task-abc123");
    }
}
