//! Reconciliation strategy for Running phase.

use std::sync::Arc;
use std::time::Duration;

use async_trait::async_trait;
use kube::ResourceExt;
use kube::runtime::controller::Action;
use tracing::info;

use crate::crd::Agent;

use super::super::context::ControllerContext;
use super::super::error::ReconcileResult;
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
        let _ctx = &self.ctx;

        info!("🔄 Agent {} is running", name);

        // Skeleton: log and requeue. Pod health checks will be added later.
        Ok(Action::requeue(REQUEUE_DURATION))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn requeue_duration_value() {
        assert_eq!(REQUEUE_DURATION.as_secs(), 10);
    }
}
