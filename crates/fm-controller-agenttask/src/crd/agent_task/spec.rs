//! AgentTask spec struct.

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use typed_builder::TypedBuilder;

use crate::crd::{Clarification, ResourceQuota};

/// Specification for an AgentTask.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema, TypedBuilder)]
#[serde(rename_all = "camelCase")]
pub struct AgentTaskSpec {
    /// Task description from user.
    pub description: String,

    /// Maximum execution time (e.g., "30m", "1h").
    #[builder(default = "30m".to_string())]
    #[serde(default = "default_timeout")]
    pub timeout: String,

    /// Answers to clarification questions.
    #[builder(default)]
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub clarifications: Vec<Clarification>,

    /// Resource limits.
    #[builder(default, setter(strip_option))]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub resource_quota: Option<ResourceQuota>,
}

fn default_timeout() -> String {
    "30m".to_string()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::crd::{ClarificationSource, Clarification};

    #[test]
    fn build_agent_task_spec_minimal() {
        let spec = AgentTaskSpec::builder()
            .description("Create an e-commerce API".to_string())
            .build();

        assert_eq!(spec.description, "Create an e-commerce API");
        assert_eq!(spec.timeout, "30m");
        assert!(spec.clarifications.is_empty());
        assert!(spec.resource_quota.is_none());
    }

    #[test]
    fn build_agent_task_spec_full() {
        let spec = AgentTaskSpec::builder()
            .description("Build a REST API".to_string())
            .timeout("1h".to_string())
            .clarifications(vec![Clarification::builder()
                .question_id("db".to_string())
                .answer("PostgreSQL".to_string())
                .source(ClarificationSource::User)
                .build()])
            .resource_quota(ResourceQuota::builder().max_agents(10).build())
            .build();

        assert_eq!(spec.timeout, "1h");
        assert_eq!(spec.clarifications.len(), 1);
        assert_eq!(spec.resource_quota.as_ref().unwrap().max_agents, 10);
    }

    #[test]
    fn agent_task_spec_serialization() {
        let spec = AgentTaskSpec::builder()
            .description("Test task".to_string())
            .build();

        let json = serde_json::to_string(&spec).expect("serialize");
        assert!(json.contains("\"description\":\"Test task\""));
        assert!(!json.contains("clarifications")); // Empty vec skipped
    }

    #[test]
    fn agent_task_spec_deserialization() {
        let json = r#"{
            "description": "Test task",
            "timeout": "2h"
        }"#;

        let spec: AgentTaskSpec = serde_json::from_str(json).expect("deserialize");
        assert_eq!(spec.description, "Test task");
        assert_eq!(spec.timeout, "2h");
    }
}
