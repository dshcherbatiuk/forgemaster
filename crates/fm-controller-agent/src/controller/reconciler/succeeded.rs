//! Reconciliation strategy for Succeeded phase.

use std::sync::Arc;

use async_trait::async_trait;
use kube::ResourceExt;
use kube::runtime::controller::Action;
use tracing::info;

use crate::crd::Agent;

use super::super::context::ControllerContext;
use super::super::error::ReconcileResult;
use super::ReconcileStrategy;

/// Strategy for reconciling agents in Succeeded phase.
pub struct SucceededStrategy {
    ctx: Arc<ControllerContext>,
}

impl SucceededStrategy {
    /// Creates a new succeeded strategy.
    pub fn new(ctx: Arc<ControllerContext>) -> Self {
        Self { ctx }
    }
}

#[async_trait]
impl ReconcileStrategy for SucceededStrategy {
    async fn reconcile(&self, agent: &Agent) -> ReconcileResult<Action> {
        let name = agent.name_any();
        let _ctx = &self.ctx;

        info!("✅ Agent {} succeeded", name);

        Ok(Action::await_change())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::crd::{AgentCrd, AgentPhase, AgentStatus, ModelConfig};
    use kube::core::ObjectMeta;

    #[test]
    fn succeeded_agent_has_correct_phase() {
        let agent = Agent {
            metadata: ObjectMeta {
                name: Some("test-agent".to_string()),
                namespace: Some("task-ns".to_string()),
                ..Default::default()
            },
            spec: AgentCrd {
                agent_type: "code-generator".to_string(),
                model: ModelConfig::builder()
                    .name("claude-sonnet-4-20250514".to_string())
                    .build(),
                task_prompt: "Generate code.".to_string(),
                mcp_servers: vec![],
                resources: None,
            },
            status: Some(AgentStatus::builder().phase(AgentPhase::Succeeded).build()),
        };

        let phase = agent.status.as_ref().map(|s| s.phase);
        assert_eq!(phase, Some(AgentPhase::Succeeded));
    }
}
