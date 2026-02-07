//! Dispatches reconciliation to the appropriate strategy based on agent phase.

use std::sync::Arc;
use std::time::Duration;

use dashmap::DashMap;
use kube::ResourceExt;
use kube::runtime::controller::Action;
use tracing::{error, info};

use crate::crd::{Agent, AgentPhase};

use super::context::ControllerContext;
use super::error::{ReconcileError, ReconcileResult};
use super::reconciler::{
    FailedStrategy, PendingStrategy, ReconcileStrategy, RunningStrategy, SucceededStrategy,
};

/// Default requeue duration after errors.
const REQUEUE_ERROR: Duration = Duration::from_secs(30);

/// Dispatches reconciliation to the appropriate strategy.
pub struct Dispatcher {
    strategies: DashMap<AgentPhase, Arc<dyn ReconcileStrategy>>,
}

impl Dispatcher {
    /// Creates a new dispatcher with all strategies.
    pub fn new(ctx: Arc<ControllerContext>) -> Self {
        let strategies: DashMap<AgentPhase, Arc<dyn ReconcileStrategy>> = DashMap::new();

        strategies.insert(
            AgentPhase::Pending,
            Arc::new(PendingStrategy::new(Arc::clone(&ctx))),
        );
        strategies.insert(
            AgentPhase::Running,
            Arc::new(RunningStrategy::new(Arc::clone(&ctx))),
        );
        strategies.insert(
            AgentPhase::Succeeded,
            Arc::new(SucceededStrategy::new(Arc::clone(&ctx))),
        );
        strategies.insert(AgentPhase::Failed, Arc::new(FailedStrategy::new(ctx)));

        Self { strategies }
    }

    /// Reconciles the agent by dispatching to the appropriate strategy.
    pub async fn reconcile(&self, agent: Arc<Agent>) -> ReconcileResult<Action> {
        let name = agent.name_any();
        let namespace = agent.namespace().unwrap_or_default();

        info!("🔄 Reconciling Agent {}/{}", namespace, name);

        let phase = agent.status.as_ref().map(|s| s.phase).unwrap_or_default();

        let strategy = self.strategies.get(&phase).ok_or_else(|| {
            ReconcileError::InvalidState(format!("no strategy for phase {phase}"))
        })?;

        strategy.reconcile(&agent).await
    }

    /// Error policy for failed reconciliations.
    pub fn error_policy(agent: Arc<Agent>, error: &ReconcileError) -> Action {
        let name = agent.name_any();
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
