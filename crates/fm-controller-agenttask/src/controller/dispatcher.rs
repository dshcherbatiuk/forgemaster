//! Dispatches reconciliation to the appropriate strategy based on task phase.

use std::sync::Arc;
use std::time::Duration;

use dashmap::DashMap;
use kube::ResourceExt;
use kube::runtime::controller::Action;
use tracing::{error, info};

use crate::crd::{AgentTask, AgentTaskPhase};

use super::context::ControllerContext;
use super::error::{ReconcileError, ReconcileResult};
use super::namespace_lifecycle::NamespaceLifecycle;
use super::reconciler::{
    ClarifyingStrategy, FailedStrategy, PendingStrategy, ReconcileStrategy, RunningStrategy,
    SucceededStrategy,
};

/// Default requeue duration after errors.
const REQUEUE_ERROR: Duration = Duration::from_secs(30);

/// Dispatches reconciliation to the appropriate strategy.
pub struct Dispatcher {
    ctx: Arc<ControllerContext>,
    strategies: DashMap<AgentTaskPhase, Arc<dyn ReconcileStrategy>>,
}

impl Dispatcher {
    /// Creates a new dispatcher with all strategies.
    pub fn new(ctx: Arc<ControllerContext>) -> Self {
        let strategies: DashMap<AgentTaskPhase, Arc<dyn ReconcileStrategy>> = DashMap::new();

        strategies.insert(
            AgentTaskPhase::Pending,
            Arc::new(PendingStrategy::new(Arc::clone(&ctx))),
        );
        strategies.insert(
            AgentTaskPhase::Clarifying,
            Arc::new(ClarifyingStrategy::new(Arc::clone(&ctx))),
        );
        strategies.insert(
            AgentTaskPhase::Running,
            Arc::new(RunningStrategy::new(Arc::clone(&ctx))),
        );
        strategies.insert(
            AgentTaskPhase::Succeeded,
            Arc::new(SucceededStrategy::new(Arc::clone(&ctx))),
        );
        strategies.insert(
            AgentTaskPhase::Failed,
            Arc::new(FailedStrategy::new(Arc::clone(&ctx))),
        );

        Self { ctx, strategies }
    }

    /// Reconciles the task by dispatching to the appropriate strategy.
    ///
    /// If the task is being deleted (has deletion_timestamp), handles cleanup
    /// instead of dispatching to the phase strategy.
    pub async fn reconcile(&self, task: Arc<AgentTask>) -> ReconcileResult<Action> {
        let name = task.name_any();
        let namespace = task.namespace().unwrap_or_default();

        info!("🔄 Reconciling AgentTask {}/{}", namespace, name);

        // Handle deletion — clean up namespace before removing finalizer
        if NamespaceLifecycle::is_deleting(&task) {
            if NamespaceLifecycle::has_finalizer(&task) {
                info!(
                    "🗑️ AgentTask {} is being deleted, cleaning up namespace",
                    name
                );
                NamespaceLifecycle::cleanup(self.ctx.client(), &task).await?;
            }
            return Ok(Action::await_change());
        }

        let phase = task.status.as_ref().map(|s| s.phase).unwrap_or_default();

        let strategy = self.strategies.get(&phase).ok_or_else(|| {
            ReconcileError::InvalidState(format!("no strategy for phase {:?}", phase))
        })?;

        strategy.reconcile(&task).await
    }

    /// Error policy for failed reconciliations.
    pub fn error_policy(task: Arc<AgentTask>, error: &ReconcileError) -> Action {
        let name = task.name_any();
        error!("❌ Reconciliation error for {}: {}", name, error);
        Action::requeue(REQUEUE_ERROR)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn requeue_error_duration() {
        assert_eq!(REQUEUE_ERROR.as_secs(), 30);
    }
}
