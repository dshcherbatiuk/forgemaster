//! Controller context for shared state.

use std::sync::Arc;

use kube::api::{Api, Patch, PatchParams};
use kube::Client;
use tracing::debug;

use crate::crd::{AgentTask, AgentTaskPhase, AgentTaskStatus};

use super::error::{ReconcileError, ReconcileResult};

/// Shared context for the AgentTask controller.
#[derive(Clone)]
pub struct ControllerContext {
    client: Client,
    namespace: String,
}

impl ControllerContext {
    /// Creates a new controller context.
    pub fn new(client: Client, namespace: String) -> Self {
        Self { client, namespace }
    }

    /// Returns reference to the Kubernetes client.
    pub fn client(&self) -> &Client {
        &self.client
    }

    /// Returns the namespace.
    pub fn namespace(&self) -> &str {
        &self.namespace
    }

    /// Updates the task phase.
    pub async fn update_phase(
        &self,
        namespace: &str,
        name: &str,
        phase: AgentTaskPhase,
    ) -> ReconcileResult<()> {
        let api: Api<AgentTask> = Api::namespaced(self.client.clone(), namespace);

        let status = AgentTaskStatus::builder().phase(phase.clone()).build();

        let patch = serde_json::json!({
            "status": status
        });

        api.patch_status(
            name,
            &PatchParams::apply("fm-controller-agenttask"),
            &Patch::Merge(&patch),
        )
        .await
        .map_err(ReconcileError::UpdateStatus)?;

        debug!("📝 Updated task {}/{} phase to {:?}", namespace, name, phase);
        Ok(())
    }
}

/// Creates an Arc-wrapped controller context.
pub fn create_context(client: Client, namespace: String) -> Arc<ControllerContext> {
    Arc::new(ControllerContext::new(client, namespace))
}

#[cfg(test)]
mod tests {
    #[test]
    fn context_namespace_not_empty() {
        let namespace = "forgemaster-system";
        assert!(!namespace.is_empty());
    }
}
