//! Task namespace lifecycle — creation, deletion, and finalizer management.

use k8s_openapi::api::core::v1::Namespace;
use kube::Client;
use kube::ResourceExt;
use kube::api::{Api, DeleteParams, ObjectMeta, Patch, PatchParams, PostParams};
use tracing::{debug, info};

use crate::crd::AgentTask;

use super::error::{ReconcileError, ReconcileResult};

/// Finalizer name added to AgentTask when a task namespace is created.
pub const TASK_NAMESPACE_FINALIZER: &str = "forgemaster.io/task-namespace";

/// Manages task namespace lifecycle (create, delete, finalizer).
pub struct NamespaceLifecycle;

impl NamespaceLifecycle {
    /// Returns the task namespace name.
    ///
    /// Task names are already formatted as `task-{uuid}` by TaskCreator,
    /// so we use the name directly (truncated to 63 chars for K8s limit).
    pub fn namespace_name(task_name: &str) -> String {
        if task_name.len() > 63 {
            task_name[..63].to_string()
        } else {
            task_name.to_string()
        }
    }

    /// Creates a namespace for the given task and adds a finalizer to the AgentTask.
    ///
    /// Idempotent — returns Ok if the namespace already exists.
    pub async fn create(client: &Client, task: &AgentTask) -> ReconcileResult<String> {
        let task_name = task.name_any();
        let ns_name = Self::namespace_name(&task_name);
        let ns_api: Api<Namespace> = Api::all(client.clone());

        let namespace = Namespace {
            metadata: ObjectMeta {
                name: Some(ns_name.clone()),
                labels: Some(
                    [
                        ("forgemaster.io/task".to_string(), task_name.clone()),
                        (
                            "app.kubernetes.io/part-of".to_string(),
                            "forgemaster".to_string(),
                        ),
                        (
                            "app.kubernetes.io/managed-by".to_string(),
                            "fm-controller-agenttask".to_string(),
                        ),
                    ]
                    .into_iter()
                    .collect(),
                ),
                ..Default::default()
            },
            ..Default::default()
        };

        match ns_api.create(&PostParams::default(), &namespace).await {
            Ok(_) => {
                info!("📦 Created task namespace: {}", ns_name);
            }
            Err(kube::Error::Api(ref err)) if err.code == 409 => {
                debug!("📦 Task namespace already exists: {}", ns_name);
            }
            Err(err) => {
                return Err(ReconcileError::CreateResource {
                    resource: format!("Namespace/{ns_name}"),
                    source: err,
                });
            }
        }

        Self::add_finalizer(client, task).await?;

        Ok(ns_name)
    }

    /// Deletes the task namespace and removes the finalizer from the AgentTask.
    ///
    /// Called when the AgentTask is being deleted (has deletion_timestamp).
    pub async fn cleanup(client: &Client, task: &AgentTask) -> ReconcileResult<()> {
        let task_name = task.name_any();
        let ns_name = Self::namespace_name(&task_name);
        let ns_api: Api<Namespace> = Api::all(client.clone());

        match ns_api.delete(&ns_name, &DeleteParams::default()).await {
            Ok(_) => {
                info!("🗑️ Deleted task namespace: {}", ns_name);
            }
            Err(kube::Error::Api(ref err)) if err.code == 404 => {
                debug!("🗑️ Task namespace already gone: {}", ns_name);
            }
            Err(err) => {
                return Err(ReconcileError::DeleteResource {
                    resource: format!("Namespace/{ns_name}"),
                    source: err,
                });
            }
        }

        Self::remove_finalizer(client, task).await?;

        Ok(())
    }

    /// Returns true if the AgentTask is being deleted.
    pub fn is_deleting(task: &AgentTask) -> bool {
        task.metadata.deletion_timestamp.is_some()
    }

    /// Returns true if the AgentTask has our finalizer.
    pub fn has_finalizer(task: &AgentTask) -> bool {
        task.metadata
            .finalizers
            .as_ref()
            .map(|finalizers| finalizers.iter().any(|f| f == TASK_NAMESPACE_FINALIZER))
            .unwrap_or(false)
    }

    /// Adds the task-namespace finalizer to the AgentTask.
    async fn add_finalizer(client: &Client, task: &AgentTask) -> ReconcileResult<()> {
        if Self::has_finalizer(task) {
            return Ok(());
        }

        let task_name = task.name_any();
        let task_namespace = task.namespace().unwrap_or_default();
        let api: Api<AgentTask> = Api::namespaced(client.clone(), &task_namespace);

        let patch = serde_json::json!({
            "metadata": {
                "finalizers": [TASK_NAMESPACE_FINALIZER]
            }
        });

        api.patch(
            &task_name,
            &PatchParams::apply("fm-controller-agenttask"),
            &Patch::Merge(&patch),
        )
        .await
        .map_err(ReconcileError::UpdateStatus)?;

        debug!("🔒 Added finalizer to AgentTask: {}", task_name);
        Ok(())
    }

    /// Removes the task-namespace finalizer from the AgentTask.
    async fn remove_finalizer(client: &Client, task: &AgentTask) -> ReconcileResult<()> {
        let task_name = task.name_any();
        let task_namespace = task.namespace().unwrap_or_default();
        let api: Api<AgentTask> = Api::namespaced(client.clone(), &task_namespace);

        let finalizers: Vec<&str> = task
            .metadata
            .finalizers
            .as_ref()
            .map(|fs| {
                fs.iter()
                    .map(|f| f.as_str())
                    .filter(|f| *f != TASK_NAMESPACE_FINALIZER)
                    .collect()
            })
            .unwrap_or_default();

        let patch = serde_json::json!({
            "metadata": {
                "finalizers": finalizers
            }
        });

        api.patch(
            &task_name,
            &PatchParams::apply("fm-controller-agenttask"),
            &Patch::Merge(&patch),
        )
        .await
        .map_err(ReconcileError::UpdateStatus)?;

        info!("🔓 Removed finalizer from AgentTask: {}", task_name);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::crd::{AgentTaskCrd, AgentTaskPhase, AgentTaskStatus};

    fn test_task(name: &str) -> AgentTask {
        AgentTask {
            metadata: ObjectMeta {
                name: Some(name.to_string()),
                namespace: Some("forgemaster-system".to_string()),
                ..Default::default()
            },
            spec: AgentTaskCrd {
                description: "Test task".to_string(),
                timeout: "30m".to_string(),
                clarifications: vec![],
                resource_quota: None,
            },
            status: Some(
                AgentTaskStatus::builder()
                    .phase(AgentTaskPhase::Pending)
                    .build(),
            ),
        }
    }

    #[test]
    fn namespace_name_uses_task_name_directly() {
        assert_eq!(
            NamespaceLifecycle::namespace_name("task-d4eca6c9"),
            "task-d4eca6c9"
        );
    }

    #[test]
    fn namespace_name_truncates_long_names() {
        let long_name = "a".repeat(100);
        let ns_name = NamespaceLifecycle::namespace_name(&long_name);
        assert_eq!(ns_name.len(), 63);
    }

    #[test]
    fn namespace_name_short_task() {
        assert_eq!(NamespaceLifecycle::namespace_name("task-abc"), "task-abc");
    }

    #[test]
    fn namespace_name_exactly_at_limit() {
        let name = "a".repeat(63);
        let ns_name = NamespaceLifecycle::namespace_name(&name);
        assert_eq!(ns_name.len(), 63);
    }

    #[test]
    fn namespace_name_one_over_limit() {
        let name = "a".repeat(64);
        let ns_name = NamespaceLifecycle::namespace_name(&name);
        assert_eq!(ns_name.len(), 63);
    }

    #[test]
    fn is_deleting_without_timestamp() {
        let task = test_task("test");
        assert!(!NamespaceLifecycle::is_deleting(&task));
    }

    #[test]
    fn is_deleting_with_timestamp() {
        let mut task = test_task("test");
        task.metadata.deletion_timestamp = Some(
            k8s_openapi::apimachinery::pkg::apis::meta::v1::Time(chrono::Utc::now()),
        );
        assert!(NamespaceLifecycle::is_deleting(&task));
    }

    #[test]
    fn has_finalizer_empty() {
        let task = test_task("test");
        assert!(!NamespaceLifecycle::has_finalizer(&task));
    }

    #[test]
    fn has_finalizer_present() {
        let mut task = test_task("test");
        task.metadata.finalizers = Some(vec![TASK_NAMESPACE_FINALIZER.to_string()]);
        assert!(NamespaceLifecycle::has_finalizer(&task));
    }

    #[test]
    fn has_finalizer_other_finalizers_only() {
        let mut task = test_task("test");
        task.metadata.finalizers = Some(vec!["other-finalizer".to_string()]);
        assert!(!NamespaceLifecycle::has_finalizer(&task));
    }

    #[test]
    fn finalizer_constant_is_correct() {
        assert_eq!(TASK_NAMESPACE_FINALIZER, "forgemaster.io/task-namespace");
    }
}
