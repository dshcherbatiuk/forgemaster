//! Resource quota struct for task limits.

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use typed_builder::TypedBuilder;

/// Resource limits for a task.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema, TypedBuilder)]
#[serde(rename_all = "camelCase")]
pub struct ResourceQuota {
    /// Maximum number of agents.
    #[builder(default = 5)]
    #[serde(default = "default_max_agents")]
    pub max_agents: i32,

    /// Maximum memory (e.g., "4Gi").
    #[builder(default = "4Gi".to_string())]
    #[serde(default = "default_max_memory")]
    pub max_memory: String,

    /// Maximum CPU (e.g., "4").
    #[builder(default = "4".to_string())]
    #[serde(default = "default_max_cpu")]
    pub max_cpu: String,
}

const fn default_max_agents() -> i32 {
    5
}

fn default_max_memory() -> String {
    "4Gi".to_string()
}

fn default_max_cpu() -> String {
    "4".to_string()
}

impl Default for ResourceQuota {
    fn default() -> Self {
        Self {
            max_agents: default_max_agents(),
            max_memory: default_max_memory(),
            max_cpu: default_max_cpu(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_resource_quota() {
        let quota = ResourceQuota::default();
        assert_eq!(quota.max_agents, 5);
        assert_eq!(quota.max_memory, "4Gi");
        assert_eq!(quota.max_cpu, "4");
    }

    #[test]
    fn build_resource_quota_custom() {
        let quota = ResourceQuota::builder()
            .max_agents(10)
            .max_memory("8Gi".to_string())
            .max_cpu("8".to_string())
            .build();

        assert_eq!(quota.max_agents, 10);
        assert_eq!(quota.max_memory, "8Gi");
        assert_eq!(quota.max_cpu, "8");
    }

    #[test]
    fn resource_quota_serialization() {
        let quota = ResourceQuota::builder().max_agents(3).build();

        let json = serde_json::to_string(&quota).expect("serialize");
        assert!(json.contains("\"maxAgents\":3"));
    }

    #[test]
    fn resource_quota_deserialization() {
        let json = r#"{"maxAgents": 7, "maxMemory": "16Gi", "maxCpu": "16"}"#;

        let quota: ResourceQuota = serde_json::from_str(json).expect("deserialize");
        assert_eq!(quota.max_agents, 7);
        assert_eq!(quota.max_memory, "16Gi");
    }

    #[test]
    fn resource_quota_deserialization_defaults() {
        let json = r#"{}"#;

        let quota: ResourceQuota = serde_json::from_str(json).expect("deserialize");
        assert_eq!(quota.max_agents, 5);
        assert_eq!(quota.max_memory, "4Gi");
    }
}
