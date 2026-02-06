//! Reconciliation strategy for Pending phase.

use std::sync::Arc;
use std::time::Duration;

use async_trait::async_trait;
use kube::runtime::controller::Action;
use kube::ResourceExt;
use tracing::{debug, info};

use crate::crd::{AgentTask, AgentTaskPhase};

use super::super::context::ControllerContext;
use super::super::error::{ReconcileError, ReconcileResult};
use super::ReconcileStrategy;

/// Requeue duration for pending tasks.
const REQUEUE_DURATION: Duration = Duration::from_secs(5);

/// Strategy for reconciling tasks in Pending phase.
pub struct PendingStrategy {
    ctx: Arc<ControllerContext>,
}

impl PendingStrategy {
    /// Creates a new pending strategy.
    pub fn new(ctx: Arc<ControllerContext>) -> Self {
        Self { ctx }
    }

    /// Checks if the task needs clarifications.
    fn needs_clarification(task: &AgentTask) -> bool {
        task.status
            .as_ref()
            .map(|s| !s.pending_clarifications.is_empty())
            .unwrap_or(false)
    }
}

#[async_trait]
impl ReconcileStrategy for PendingStrategy {
    async fn reconcile(&self, task: &AgentTask) -> ReconcileResult<Action> {
        let name = task.name_any();
        let namespace = task.namespace().ok_or_else(|| {
            ReconcileError::MissingField("namespace".to_string())
        })?;

        debug!("📋 Task {} is pending, checking if clarifications needed", name);

        if Self::needs_clarification(task) {
            self.ctx.update_phase(&namespace, &name, AgentTaskPhase::Clarifying).await?;
            info!("❓ Task {} needs clarifications, transitioning to Clarifying", name);
        } else {
            self.ctx.update_phase(&namespace, &name, AgentTaskPhase::Running).await?;
            info!("🚀 Task {} starting execution, transitioning to Running", name);
        }

        Ok(Action::requeue(REQUEUE_DURATION))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::crd::{AgentTaskCrd, AgentTaskStatus, PendingClarification};
    use kube::core::ObjectMeta;

    fn create_test_task(phase: AgentTaskPhase) -> AgentTask {
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
            status: Some(AgentTaskStatus::builder().phase(phase).build()),
        }
    }

    #[test]
    fn clarifications_not_needed_empty() {
        let task = create_test_task(AgentTaskPhase::Pending);
        assert!(!PendingStrategy::needs_clarification(&task));
    }

    #[test]
    fn clarifications_needed_with_pending() {
        let mut task = create_test_task(AgentTaskPhase::Pending);

        let pending = PendingClarification::builder()
            .id("db-choice".to_string())
            .question("Which database?".to_string())
            .build();

        task.status = Some(
            AgentTaskStatus::builder()
                .phase(AgentTaskPhase::Clarifying)
                .pending_clarifications(vec![pending])
                .build(),
        );

        assert!(PendingStrategy::needs_clarification(&task));
    }

    #[test]
    fn clarifications_no_status() {
        let mut task = create_test_task(AgentTaskPhase::Pending);
        task.status = None;
        assert!(!PendingStrategy::needs_clarification(&task));
    }

    #[test]
    fn requeue_duration_value() {
        assert_eq!(REQUEUE_DURATION.as_secs(), 5);
    }
}
