//! Creates AgentTask CRDs in Kubernetes from UI actions.

use anyhow::{Context, Result};
use kube::api::PostParams;
use kube::core::ObjectMeta;
use kube::{Api, Client};
use tracing::info;
use uuid::Uuid;

use crate::crd::{AgentTask, AgentTaskCrd};

/// Creates AgentTask custom resources in Kubernetes.
pub struct TaskCreator {
    client: Client,
    namespace: String,
}

impl TaskCreator {
    /// Creates a new task creator for the given namespace.
    pub fn new(client: Client, namespace: String) -> Self {
        Self { client, namespace }
    }

    /// Creates an AgentTask CR from a task description.
    ///
    /// Generates a unique name `task-{short-uuid}` and creates the resource
    /// in the configured namespace. Returns the generated task name.
    pub async fn create_from_description(&self, description: &str) -> Result<String> {
        let task_name = format!("task-{}", &Uuid::new_v4().to_string()[..8]);

        let task = build_task(&task_name, &self.namespace, description);

        let api: Api<AgentTask> = Api::namespaced(self.client.clone(), &self.namespace);
        api.create(&PostParams::default(), &task)
            .await
            .context("Failed to create AgentTask CRD")?;

        info!("✅ Created AgentTask: {task_name}");
        Ok(task_name)
    }
}

/// Builds an AgentTask struct with minimal required fields.
fn build_task(task_name: &str, namespace: &str, description: &str) -> AgentTask {
    AgentTask {
        metadata: ObjectMeta {
            name: Some(task_name.to_string()),
            namespace: Some(namespace.to_string()),
            ..Default::default()
        },
        spec: AgentTaskCrd {
            description: description.to_string(),
            timeout: "30m".to_string(),
            clarifications: vec![],
            resource_quota: None,
        },
        status: None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn build_task_sets_name_and_namespace() {
        let task = build_task("task-abc12345", "forgemaster-system", "Build an API");
        assert_eq!(task.metadata.name.unwrap(), "task-abc12345");
        assert_eq!(task.metadata.namespace.unwrap(), "forgemaster-system");
    }

    #[test]
    fn build_task_sets_description() {
        let task = build_task("task-abc12345", "test-ns", "Create e-commerce API");
        assert_eq!(task.spec.description, "Create e-commerce API");
    }

    #[test]
    fn build_task_uses_default_timeout() {
        let task = build_task("task-abc12345", "test-ns", "Test task");
        assert_eq!(task.spec.timeout, "30m");
    }

    #[test]
    fn build_task_has_empty_clarifications() {
        let task = build_task("task-abc12345", "test-ns", "Test task");
        assert!(task.spec.clarifications.is_empty());
    }

    #[test]
    fn build_task_has_no_status() {
        let task = build_task("task-abc12345", "test-ns", "Test task");
        assert!(task.status.is_none());
    }

    #[test]
    fn build_task_serializes_to_json() {
        let task = build_task("task-abc12345", "test-ns", "Build API");
        let json = serde_json::to_value(&task).unwrap();
        assert_eq!(json["spec"]["description"], "Build API");
        assert_eq!(json["spec"]["timeout"], "30m");
    }
}
