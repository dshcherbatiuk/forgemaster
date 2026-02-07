//! Reconciliation strategy for Pending phase.
//!
//! Ensures the LLM provider secret exists in the task namespace,
//! creates a runtime pod for the agent if one does not already exist,
//! then transitions the agent to Running.

use std::sync::Arc;
use std::time::Duration;

use async_trait::async_trait;
use k8s_openapi::api::core::v1::Pod;
use kube::ResourceExt;
use kube::api::{Api, PostParams};
use kube::runtime::controller::Action;
use tracing::info;

use crate::crd::{Agent, AgentPhase, PodRef};

use super::super::context::ControllerContext;
use super::super::error::{ReconcileError, ReconcileResult};
use super::super::pod_builder;
use super::super::rbac_propagator;
use super::super::secret_propagator;
use super::ReconcileStrategy;

/// Requeue duration for pending agents.
const REQUEUE_DURATION: Duration = Duration::from_secs(5);

/// Strategy for reconciling agents in Pending phase.
pub struct PendingStrategy {
    ctx: Arc<ControllerContext>,
}

impl PendingStrategy {
    /// Creates a new pending strategy.
    pub fn new(ctx: Arc<ControllerContext>) -> Self {
        Self { ctx }
    }
}

#[async_trait]
impl ReconcileStrategy for PendingStrategy {
    async fn reconcile(&self, agent: &Agent) -> ReconcileResult<Action> {
        let name = agent.name_any();
        let namespace = agent
            .namespace()
            .ok_or_else(|| ReconcileError::MissingField("metadata.namespace".to_string()))?;

        // Ensure RBAC and LLM provider secret exist in task namespace
        rbac_propagator::ensure_rbac(&self.ctx, &namespace).await?;
        secret_propagator::ensure_secret(&self.ctx, &namespace).await?;

        let pod_api: Api<Pod> = Api::namespaced(self.ctx.client().clone(), &namespace);

        // Check if pod already exists
        let existing_pod = pod_api
            .get_opt(&name)
            .await
            .map_err(ReconcileError::GetAgent)?;

        if let Some(pod) = existing_pod {
            info!("📦 Pod already exists for agent {}, transitioning to Running", name);
            let pod_uid = pod.metadata.uid.clone().unwrap_or_default();
            update_status_with_pod_ref(&self.ctx, agent, &name, &pod_uid).await?;
            return Ok(Action::requeue(REQUEUE_DURATION));
        }

        // Build and create the pod
        let pod = pod_builder::build(agent, &self.ctx);
        info!("🚀 Creating runtime pod for agent {}", name);

        let created_pod = pod_api
            .create(&PostParams::default(), &pod)
            .await
            .map_err(|source| ReconcileError::CreateResource {
                resource: format!("Pod/{name}"),
                source,
            })?;

        let pod_uid = created_pod.metadata.uid.clone().unwrap_or_default();
        update_status_with_pod_ref(&self.ctx, agent, &name, &pod_uid).await?;

        info!("✅ Created runtime pod for agent {}", name);
        Ok(Action::requeue(REQUEUE_DURATION))
    }
}

/// Updates Agent status with pod reference and transitions to Running.
async fn update_status_with_pod_ref(
    ctx: &ControllerContext,
    agent: &Agent,
    pod_name: &str,
    pod_uid: &str,
) -> ReconcileResult<()> {
    let name = agent.name_any();
    let namespace = agent.namespace().unwrap_or_default();
    let api: Api<Agent> = Api::namespaced(ctx.client().clone(), &namespace);

    let patch = serde_json::json!({
        "status": {
            "phase": AgentPhase::Running,
            "podRef": PodRef::builder()
                .name(pod_name.to_string())
                .uid(pod_uid.to_string())
                .build(),
        }
    });

    api.patch_status(
        &name,
        &kube::api::PatchParams::apply("fm-controller-agent"),
        &kube::api::Patch::Merge(&patch),
    )
    .await
    .map_err(ReconcileError::UpdateStatus)?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::crd::{AgentCrd, ModelConfig};
    use kube::api::ObjectMeta;

    #[test]
    fn requeue_duration_value() {
        assert_eq!(REQUEUE_DURATION.as_secs(), 5);
    }

    #[test]
    fn status_patch_contains_pod_ref() {
        let patch = serde_json::json!({
            "status": {
                "phase": AgentPhase::Running,
                "podRef": PodRef::builder()
                    .name("test-pod".to_string())
                    .uid("uid-123".to_string())
                    .build(),
            }
        });

        let status = patch.get("status").unwrap();
        assert_eq!(status["phase"], "Running");
        assert_eq!(status["podRef"]["name"], "test-pod");
        assert_eq!(status["podRef"]["uid"], "uid-123");
    }

    #[test]
    fn pending_agent_fixture() {
        let agent = Agent {
            metadata: ObjectMeta {
                name: Some("test-agent".to_string()),
                namespace: Some("task-ns".to_string()),
                uid: Some("agent-uid".to_string()),
                ..Default::default()
            },
            spec: AgentCrd {
                agent_type: "orchestrator".to_string(),
                model: ModelConfig::builder()
                    .name("claude-sonnet-4-20250514".to_string())
                    .build(),
                task_prompt: "Build an API".to_string(),
                mcp_servers: vec![],
                resources: None,
            },
            status: None,
        };

        assert_eq!(agent.name_any(), "test-agent");
        assert_eq!(agent.namespace().unwrap(), "task-ns");
    }
}
