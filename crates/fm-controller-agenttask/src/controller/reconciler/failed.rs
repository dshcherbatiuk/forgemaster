//! Reconciliation strategy for Failed phase.

use std::sync::Arc;

use async_trait::async_trait;
use kube::ResourceExt;
use kube::runtime::controller::Action;
use tracing::warn;

use crate::crd::AgentTask;

use super::super::context::ControllerContext;
use super::super::error::ReconcileResult;
use super::ReconcileStrategy;

/// Strategy for reconciling tasks in Failed phase.
pub struct FailedStrategy {
    ctx: Arc<ControllerContext>,
}

impl FailedStrategy {
    /// Creates a new failed strategy.
    pub fn new(ctx: Arc<ControllerContext>) -> Self {
        Self { ctx }
    }

    /// Returns the context.
    #[allow(dead_code)]
    pub fn context(&self) -> &ControllerContext {
        &self.ctx
    }
}

#[async_trait]
impl ReconcileStrategy for FailedStrategy {
    async fn reconcile(&self, task: &AgentTask) -> ReconcileResult<Action> {
        let name = task.name_any();

        warn!("❌ Task {} failed", name);

        // Terminal state, no requeue needed
        Ok(Action::await_change())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::crd::{AgentTaskCrd, AgentTaskPhase, AgentTaskStatus};
    use kube::core::ObjectMeta;

    fn create_failed_task() -> AgentTask {
        AgentTask {
            metadata: ObjectMeta {
                name: Some("test-task".to_string()),
                namespace: Some("test-ns".to_string()),
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
                    .phase(AgentTaskPhase::Failed)
                    .build(),
            ),
        }
    }

    #[test]
    fn failed_task_has_correct_phase() {
        let task = create_failed_task();
        let phase = task.status.as_ref().map(|s| &s.phase);
        assert_eq!(phase, Some(&AgentTaskPhase::Failed));
    }
}
