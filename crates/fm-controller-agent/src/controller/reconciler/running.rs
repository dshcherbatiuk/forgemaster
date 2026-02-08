//! Reconciliation strategy for Running phase.
//!
//! Watches the runtime pod and transitions the agent based on pod phase:
//! - Pod Succeeded → Agent Succeeded
//! - Pod Failed → Agent Failed
//! - Pod not found → Agent Pending (triggers pod recreation)
//! - Pod Running/Pending → requeue

use std::sync::Arc;
use std::time::Duration;

use async_trait::async_trait;
use k8s_openapi::api::core::v1::Pod;
use kube::ResourceExt;
use kube::api::Api;
use kube::runtime::controller::Action;
use tracing::info;

use kube::runtime::events::EventType;

use crate::crd::{Agent, AgentPhase, Condition};

use super::super::context::ControllerContext;
use super::super::error::{ReconcileError, ReconcileResult};
use crate::controller::event_recorder;
use super::ReconcileStrategy;

/// Requeue duration for running agents.
const REQUEUE_DURATION: Duration = Duration::from_secs(10);

/// Strategy for reconciling agents in Running phase.
pub struct RunningStrategy {
    ctx: Arc<ControllerContext>,
}

impl RunningStrategy {
    /// Creates a new running strategy.
    pub fn new(ctx: Arc<ControllerContext>) -> Self {
        Self { ctx }
    }
}

#[async_trait]
impl ReconcileStrategy for RunningStrategy {
    async fn reconcile(&self, agent: &Agent) -> ReconcileResult<Action> {
        let name = agent.name_any();
        let namespace = agent
            .namespace()
            .ok_or_else(|| ReconcileError::MissingField("metadata.namespace".to_string()))?;

        let pod_api: Api<Pod> = Api::namespaced(self.ctx.client().clone(), &namespace);

        let pod = pod_api
            .get_opt(&name)
            .await
            .map_err(ReconcileError::GetAgent)?;

        let Some(pod) = pod else {
            info!("🔄 Pod not found for agent {}, transitioning to Pending for recreation", name);
            event_recorder::publish(
                self.ctx.client(),
                agent,
                EventType::Warning,
                "PodNotFound",
                &format!("Pod not found for agent {name}, transitioning to Pending"),
            )
            .await;
            self.ctx.update_phase(agent, AgentPhase::Pending).await?;
            return Ok(Action::requeue(REQUEUE_DURATION));
        };

        let pod_phase = pod
            .status
            .as_ref()
            .and_then(|s| s.phase.as_deref())
            .unwrap_or("Unknown");

        match pod_phase {
            "Succeeded" => {
                info!("✅ Agent {} pod succeeded", name);
                event_recorder::publish(
                    self.ctx.client(),
                    agent,
                    EventType::Normal,
                    "AgentSucceeded",
                    "Agent completed successfully",
                )
                .await;
                self.ctx.update_phase(agent, AgentPhase::Succeeded).await?;
                Ok(Action::await_change())
            }
            "Failed" => {
                info!("❌ Agent {} pod failed", name);
                let message = extract_pod_failure_message(&pod);
                event_recorder::publish(
                    self.ctx.client(),
                    agent,
                    EventType::Warning,
                    "AgentFailed",
                    &format!("Pod failed: {message}"),
                )
                .await;
                update_phase_with_condition(
                    &self.ctx,
                    agent,
                    AgentPhase::Failed,
                    "PodFailed",
                    &message,
                )
                .await?;
                Ok(Action::await_change())
            }
            _ => {
                info!("🔄 Agent {} pod is {}, requeuing", name, pod_phase);
                Ok(Action::requeue(REQUEUE_DURATION))
            }
        }
    }
}

/// Extracts a failure message from the pod's container statuses.
fn extract_pod_failure_message(pod: &Pod) -> String {
    pod.status
        .as_ref()
        .and_then(|s| s.container_statuses.as_ref())
        .and_then(|statuses| statuses.first())
        .and_then(|cs| cs.state.as_ref())
        .and_then(|state| state.terminated.as_ref())
        .and_then(|term| term.message.clone())
        .unwrap_or_else(|| "Pod failed without message".to_string())
}

/// Updates Agent phase and adds a status condition.
async fn update_phase_with_condition(
    ctx: &ControllerContext,
    agent: &Agent,
    phase: AgentPhase,
    reason: &str,
    message: &str,
) -> ReconcileResult<()> {
    let name = agent.name_any();
    let namespace = agent.namespace().unwrap_or_default();
    let api: Api<Agent> = Api::namespaced(ctx.client().clone(), &namespace);

    let condition = Condition::builder()
        .condition_type("Ready".to_string())
        .status("False".to_string())
        .reason(reason.to_string())
        .message(message.to_string())
        .last_transition_time(chrono::Utc::now().to_rfc3339())
        .build();

    let patch = serde_json::json!({
        "status": {
            "phase": phase,
            "conditions": [condition],
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
    use crate::crd::{AgentCrd, AgentStatus, ModelConfig};
    use k8s_openapi::api::core::v1::{ContainerState, ContainerStateTerminated, ContainerStatus, PodStatus};
    use kube::api::ObjectMeta;

    fn running_agent() -> Agent {
        Agent {
            metadata: ObjectMeta {
                name: Some("test-agent".to_string()),
                namespace: Some("task-ns".to_string()),
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
            status: Some(AgentStatus::builder().phase(AgentPhase::Running).build()),
        }
    }

    #[test]
    fn requeue_duration_value() {
        assert_eq!(REQUEUE_DURATION.as_secs(), 10);
    }

    #[test]
    fn extract_failure_message_with_terminated() {
        let pod = Pod {
            metadata: ObjectMeta::default(),
            spec: None,
            status: Some(PodStatus {
                container_statuses: Some(vec![ContainerStatus {
                    name: "agent-runtime".to_string(),
                    state: Some(ContainerState {
                        terminated: Some(ContainerStateTerminated {
                            message: Some("OOMKilled".to_string()),
                            exit_code: 137,
                            ..Default::default()
                        }),
                        ..Default::default()
                    }),
                    ..Default::default()
                }]),
                ..Default::default()
            }),
        };

        assert_eq!(extract_pod_failure_message(&pod), "OOMKilled");
    }

    #[test]
    fn extract_failure_message_without_terminated() {
        let pod = Pod {
            metadata: ObjectMeta::default(),
            spec: None,
            status: Some(PodStatus::default()),
        };

        assert_eq!(
            extract_pod_failure_message(&pod),
            "Pod failed without message"
        );
    }

    #[test]
    fn condition_patch_structure() {
        let condition = Condition::builder()
            .condition_type("Ready".to_string())
            .status("False".to_string())
            .reason("PodFailed".to_string())
            .message("Container exited with code 1".to_string())
            .last_transition_time("2026-02-07T12:00:00Z".to_string())
            .build();

        let patch = serde_json::json!({
            "status": {
                "phase": AgentPhase::Failed,
                "conditions": [condition],
            }
        });

        let status = patch.get("status").unwrap();
        assert_eq!(status["phase"], "Failed");
        let conditions = status["conditions"].as_array().unwrap();
        assert_eq!(conditions.len(), 1);
        assert_eq!(conditions[0]["reason"], "PodFailed");
    }

    #[test]
    fn running_agent_fixture() {
        let agent = running_agent();
        let phase = agent.status.as_ref().map(|s| s.phase);
        assert_eq!(phase, Some(AgentPhase::Running));
    }
}
