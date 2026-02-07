//! Reconciliation strategy for Failed phase.

use std::sync::Arc;

use async_trait::async_trait;
use kube::ResourceExt;
use kube::runtime::controller::Action;
use tracing::info;

use crate::crd::Agent;

use super::super::context::ControllerContext;
use super::super::error::ReconcileResult;
use super::ReconcileStrategy;

/// Strategy for reconciling agents in Failed phase.
pub struct FailedStrategy {
    ctx: Arc<ControllerContext>,
}

impl FailedStrategy {
    /// Creates a new failed strategy.
    pub fn new(ctx: Arc<ControllerContext>) -> Self {
        Self { ctx }
    }
}

#[async_trait]
impl ReconcileStrategy for FailedStrategy {
    async fn reconcile(&self, agent: &Agent) -> ReconcileResult<Action> {
        let name = agent.name_any();
        let _ctx = &self.ctx;

        info!("❌ Agent {} failed", name);

        Ok(Action::await_change())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::crd::{AgentCrd, AgentPhase, AgentStatus, ModelConfig};
    use kube::core::ObjectMeta;

    #[test]
    fn failed_agent_has_correct_phase() {
        let agent = Agent {
            metadata: ObjectMeta {
                name: Some("test-agent".to_string()),
                namespace: Some("task-ns".to_string()),
                ..Default::default()
            },
            spec: AgentCrd {
                agent_type: "test-runner".to_string(),
                model: ModelConfig::builder()
                    .name("claude-sonnet-4-20250514".to_string())
                    .build(),
                task_prompt: "Run tests.".to_string(),
                mcp_servers: vec![],
                resources: None,
            },
            status: Some(AgentStatus::builder().phase(AgentPhase::Failed).build()),
        };

        let phase = agent.status.as_ref().map(|s| s.phase);
        assert_eq!(phase, Some(AgentPhase::Failed));
    }
}
