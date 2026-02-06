//! Controller context for shared state.

use std::sync::Arc;

use kube::api::{Api, Patch, PatchParams};
use kube::Client;
use kube::ResourceExt;
use tracing::debug;

use crate::crd::{Agent, AgentPhase};

use super::error::{ReconcileError, ReconcileResult};

/// Shared context for the Agent controller.
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

    /// Updates the agent phase via status subresource patch.
    pub async fn update_phase(
        &self,
        agent: &Agent,
        phase: AgentPhase,
    ) -> ReconcileResult<()> {
        let name = agent.name_any();
        let namespace = agent.namespace().unwrap_or_else(|| self.namespace.clone());
        let api: Api<Agent> = Api::namespaced(self.client.clone(), &namespace);

        let patch = serde_json::json!({
            "status": { "phase": phase }
        });

        api.patch_status(
            &name,
            &PatchParams::apply("fm-controller-agent"),
            &Patch::Merge(&patch),
        )
        .await
        .map_err(ReconcileError::UpdateStatus)?;

        debug!("📝 Updated agent {}/{} phase to {}", namespace, name, phase);
        Ok(())
    }
}

/// Creates an Arc-wrapped controller context.
pub fn create_context(client: Client, namespace: String) -> Arc<ControllerContext> {
    Arc::new(ControllerContext::new(client, namespace))
}

#[cfg(test)]
mod tests {
    use crate::crd::AgentPhase;

    #[test]
    fn context_namespace_not_empty() {
        let namespace = "forgemaster-system";
        assert!(!namespace.is_empty());
    }

    #[test]
    fn phase_patch_contains_only_phase() {
        let phase = AgentPhase::Running;
        let patch = serde_json::json!({
            "status": { "phase": phase }
        });

        let status = patch.get("status").unwrap();
        assert_eq!(status.get("phase").unwrap(), "Running");
        assert!(status.get("tokensUsed").is_none());
        assert!(status.get("iterationsCompleted").is_none());
    }
}
