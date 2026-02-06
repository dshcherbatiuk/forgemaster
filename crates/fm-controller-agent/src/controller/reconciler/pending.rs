//! Reconciliation strategy for Pending phase.

use std::sync::Arc;
use std::time::Duration;

use async_trait::async_trait;
use kube::runtime::controller::Action;
use kube::ResourceExt;
use tracing::info;

use crate::crd::Agent;

use super::super::context::ControllerContext;
use super::super::error::ReconcileResult;
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
        let _ctx = &self.ctx;

        info!("⏳ Agent {} is pending, waiting for pod scheduling", name);

        // Skeleton: log and requeue. Pod creation will be added later.
        Ok(Action::requeue(REQUEUE_DURATION))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn requeue_duration_value() {
        assert_eq!(REQUEUE_DURATION.as_secs(), 5);
    }
}
