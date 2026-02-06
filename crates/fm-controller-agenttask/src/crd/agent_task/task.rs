//! AgentTask Custom Resource Definition.

use kube::CustomResource;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use super::AgentTaskStatus;
use crate::crd::{Clarification, ResourceQuota};

/// AgentTask is the primary CRD for task management in ForgeMaster.
#[derive(CustomResource, Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[kube(
    group = "forgemaster.io",
    version = "v1alpha1",
    kind = "AgentTask",
    plural = "agenttasks",
    shortname = "at",
    status = "AgentTaskStatus",
    namespaced,
    printcolumn = r#"{"name": "Phase", "type": "string", "jsonPath": ".status.phase"}"#,
    printcolumn = r#"{"name": "Iteration", "type": "integer", "jsonPath": ".status.iteration"}"#,
    printcolumn = r#"{"name": "Error", "type": "number", "jsonPath": ".status.error"}"#,
    printcolumn = r#"{"name": "Age", "type": "date", "jsonPath": ".metadata.creationTimestamp"}"#
)]
#[serde(rename_all = "camelCase")]
pub struct AgentTaskCrd {
    /// Task description from user.
    pub description: String,

    /// Maximum execution time (e.g., "30m", "1h").
    #[serde(default = "default_timeout")]
    pub timeout: String,

    /// Answers to clarification questions.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub clarifications: Vec<Clarification>,

    /// Resource limits.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub resource_quota: Option<ResourceQuota>,
}

fn default_timeout() -> String {
    "30m".to_string()
}

#[cfg(test)]
mod tests {
    use super::*;
    use kube::core::ObjectMeta;

    #[test]
    fn create_agent_task() {
        let task = AgentTask {
            metadata: ObjectMeta {
                name: Some("test-task".to_string()),
                namespace: Some("forgemaster-system".to_string()),
                ..Default::default()
            },
            spec: AgentTaskCrd {
                description: "Create an e-commerce API".to_string(),
                timeout: "30m".to_string(),
                clarifications: vec![],
                resource_quota: None,
            },
            status: None,
        };

        assert_eq!(task.metadata.name, Some("test-task".to_string()));
        assert_eq!(task.spec.description, "Create an e-commerce API");
    }

    #[test]
    fn agent_task_api_resource() {
        use kube::Resource;

        assert_eq!(AgentTask::group(&()), "forgemaster.io");
        assert_eq!(AgentTask::version(&()), "v1alpha1");
        assert_eq!(AgentTask::kind(&()), "AgentTask");
        assert_eq!(AgentTask::plural(&()), "agenttasks");
    }

    #[test]
    fn agent_task_serialization() {
        let task = AgentTask {
            metadata: ObjectMeta {
                name: Some("test".to_string()),
                ..Default::default()
            },
            spec: AgentTaskCrd {
                description: "Test".to_string(),
                timeout: "1h".to_string(),
                clarifications: vec![],
                resource_quota: None,
            },
            status: None,
        };

        let json = serde_json::to_string(&task).expect("serialize");
        assert!(json.contains("\"description\":\"Test\""));
    }
}
