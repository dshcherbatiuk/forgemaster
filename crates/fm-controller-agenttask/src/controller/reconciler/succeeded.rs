//! Reconciliation strategy for Succeeded phase.

use std::sync::Arc;

use async_trait::async_trait;
use kube::runtime::controller::Action;
use kube::ResourceExt;
use tracing::info;

use crate::crd::AgentTask;

use super::super::context::ControllerContext;
use super::super::error::ReconcileResult;
use super::ReconcileStrategy;

/// Strategy for reconciling tasks in Succeeded phase.
pub struct SucceededStrategy {
    ctx: Arc<ControllerContext>,
}

impl SucceededStrategy {
    /// Creates a new succeeded strategy.
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
impl ReconcileStrategy for SucceededStrategy {
    async fn reconcile(&self, task: &AgentTask) -> ReconcileResult<Action> {
        let name = task.name_any();

        info!("✅ Task {} succeeded", name);

        // Terminal state, no requeue needed
        Ok(Action::await_change())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::crd::{AgentTaskCrd, AgentTaskPhase, AgentTaskStatus};
    use kube::core::ObjectMeta;

    fn create_succeeded_task() -> AgentTask {
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
                    .phase(AgentTaskPhase::Succeeded)
                    .build(),
            ),
        }
    }

    #[test]
    fn succeeded_task_has_correct_phase() {
        let task = create_succeeded_task();
        let phase = task.status.as_ref().map(|s| &s.phase);
        assert_eq!(phase, Some(&AgentTaskPhase::Succeeded));
    }
}
