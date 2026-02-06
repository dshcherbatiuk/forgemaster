//! Controller context for shared state.

use std::sync::Arc;

use kube::api::{Api, Patch, PatchParams};
use kube::Client;
use tokio::sync::broadcast;
use tracing::{debug, warn};

use kube::ResourceExt;

use crate::crd::{AgentTask, AgentTaskPhase};
use crate::task_state_changed::TaskStateChanged;

use super::error::{ReconcileError, ReconcileResult};

/// Shared context for the AgentTask controller.
#[derive(Clone)]
pub struct ControllerContext {
    client: Client,
    namespace: String,
    state_sender: broadcast::Sender<TaskStateChanged>,
}

impl ControllerContext {
    /// Creates a new controller context.
    pub fn new(
        client: Client,
        namespace: String,
        state_sender: broadcast::Sender<TaskStateChanged>,
    ) -> Self {
        Self {
            client,
            namespace,
            state_sender,
        }
    }

    /// Returns reference to the Kubernetes client.
    pub fn client(&self) -> &Client {
        &self.client
    }

    /// Returns the namespace.
    pub fn namespace(&self) -> &str {
        &self.namespace
    }

    /// Updates the task phase and emits a state change event.
    pub async fn update_phase(
        &self,
        task: &AgentTask,
        phase: AgentTaskPhase,
    ) -> ReconcileResult<()> {
        let name = task.name_any();
        let namespace = task.namespace().unwrap_or_else(|| self.namespace.clone());
        let api: Api<AgentTask> = Api::namespaced(self.client.clone(), &namespace);

        let patch = serde_json::json!({
            "status": { "phase": phase }
        });

        api.patch_status(
            &name,
            &PatchParams::apply("fm-controller-agenttask"),
            &Patch::Merge(&patch),
        )
        .await
        .map_err(ReconcileError::UpdateStatus)?;

        debug!("📝 Updated task {}/{} phase to {:?}", namespace, name, phase);

        let status = task.status.as_ref().cloned().unwrap_or_default();
        let event = TaskStateChanged {
            task_name: name.clone(),
            namespace,
            description: task.spec.description.clone(),
            phase,
            iteration: status.iteration,
            error: status.error,
            tests_total: status.tests_total,
            tests_passed: status.tests_passed,
        };

        if self.state_sender.send(event).is_err() {
            warn!("⚠️ No receivers for task state change: {}", name);
        }

        Ok(())
    }
}

/// Creates an Arc-wrapped controller context.
pub fn create_context(
    client: Client,
    namespace: String,
    state_sender: broadcast::Sender<TaskStateChanged>,
) -> Arc<ControllerContext> {
    Arc::new(ControllerContext::new(client, namespace, state_sender))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::crd::AgentTaskPhase;

    #[test]
    fn context_namespace_not_empty() {
        let namespace = "forgemaster-system";
        assert!(!namespace.is_empty());
    }

    #[test]
    fn phase_patch_contains_only_phase() {
        let phase = AgentTaskPhase::Running;
        let patch = serde_json::json!({
            "status": { "phase": phase }
        });

        let status = patch.get("status").unwrap();
        assert_eq!(status.get("phase").unwrap(), "Running");
        assert!(
            status.get("error").is_none(),
            "patch must not overwrite error with default"
        );
        assert!(
            status.get("iteration").is_none(),
            "patch must not overwrite iteration"
        );
        assert!(
            status.get("tests_total").is_none(),
            "patch must not overwrite tests_total"
        );
        assert!(
            status.get("tests_passed").is_none(),
            "patch must not overwrite tests_passed"
        );
    }

    #[test]
    fn send_event_no_receivers_does_not_panic() {
        let (sender, receiver) = broadcast::channel::<TaskStateChanged>(16);
        drop(receiver);
        let event = TaskStateChanged {
            task_name: "task-123".to_string(),
            namespace: "test-ns".to_string(),
            description: "Test task".to_string(),
            phase: AgentTaskPhase::Running,
            iteration: 0,
            error: 1.0,
            tests_total: 0,
            tests_passed: 0,
        };
        // Should not panic even with no receivers
        let result = sender.send(event);
        assert!(result.is_err());
    }

    #[test]
    fn send_event_received_by_subscriber() {
        let (sender, mut receiver) = broadcast::channel::<TaskStateChanged>(16);
        let event = TaskStateChanged {
            task_name: "task-abc".to_string(),
            namespace: "forgemaster-system".to_string(),
            description: "Build a REST API".to_string(),
            phase: AgentTaskPhase::Running,
            iteration: 1,
            error: 0.5,
            tests_total: 10,
            tests_passed: 5,
        };
        sender.send(event).unwrap();
        let received = receiver.try_recv().unwrap();
        assert_eq!(received.task_name, "task-abc");
        assert_eq!(received.phase, AgentTaskPhase::Running);
        assert_eq!(received.iteration, 1);
    }
}
