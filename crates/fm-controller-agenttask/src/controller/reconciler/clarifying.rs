//! Reconciliation strategy for Clarifying phase.

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

/// Requeue duration for clarifying tasks.
const REQUEUE_DURATION: Duration = Duration::from_secs(5);

/// Strategy for reconciling tasks in Clarifying phase.
pub struct ClarifyingStrategy {
    ctx: Arc<ControllerContext>,
}

impl ClarifyingStrategy {
    /// Creates a new clarifying strategy.
    pub fn new(ctx: Arc<ControllerContext>) -> Self {
        Self { ctx }
    }

    /// Returns count of pending clarifications.
    fn pending_count(task: &AgentTask) -> usize {
        task.status
            .as_ref()
            .map(|s| s.pending_clarifications.len())
            .unwrap_or(0)
    }
}

#[async_trait]
impl ReconcileStrategy for ClarifyingStrategy {
    async fn reconcile(&self, task: &AgentTask) -> ReconcileResult<Action> {
        let name = task.name_any();
        let namespace = task.namespace().ok_or_else(|| {
            ReconcileError::MissingField("namespace".to_string())
        })?;

        debug!("❓ Task {} is clarifying, checking pending clarifications", name);

        let pending = Self::pending_count(task);

        if pending == 0 {
            self.ctx.update_phase(&namespace, &name, AgentTaskPhase::Running).await?;
            info!("✅ Task {} clarifications resolved, transitioning to Running", name);
        } else {
            debug!("⏳ Task {} waiting for {} clarifications", name, pending);
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
    fn pending_count_empty() {
        let task = create_test_task(AgentTaskPhase::Clarifying);
        assert_eq!(ClarifyingStrategy::pending_count(&task), 0);
    }

    #[test]
    fn pending_count_with_clarifications() {
        let mut task = create_test_task(AgentTaskPhase::Clarifying);

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

        assert_eq!(ClarifyingStrategy::pending_count(&task), 1);
    }

    #[test]
    fn pending_count_no_status() {
        let mut task = create_test_task(AgentTaskPhase::Clarifying);
        task.status = None;
        assert_eq!(ClarifyingStrategy::pending_count(&task), 0);
    }

    #[test]
    fn requeue_duration_value() {
        assert_eq!(REQUEUE_DURATION.as_secs(), 5);
    }
}
