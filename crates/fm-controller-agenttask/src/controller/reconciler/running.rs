//! Reconciliation strategy for Running phase.

use std::sync::Arc;
use std::time::Duration;

use async_trait::async_trait;
use kube::runtime::controller::Action;
use kube::ResourceExt;
use tracing::debug;

use crate::crd::AgentTask;

use super::super::context::ControllerContext;
use super::super::error::ReconcileResult;
use super::ReconcileStrategy;

/// Requeue duration for running tasks.
const REQUEUE_DURATION: Duration = Duration::from_secs(10);

/// Strategy for reconciling tasks in Running phase.
pub struct RunningStrategy {
    ctx: Arc<ControllerContext>,
}

impl RunningStrategy {
    /// Creates a new running strategy.
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
impl ReconcileStrategy for RunningStrategy {
    async fn reconcile(&self, task: &AgentTask) -> ReconcileResult<Action> {
        let name = task.name_any();

        debug!("🏃 Task {} is running, monitoring progress", name);

        // TODO: Check agent pods status
        // TODO: Update iteration count
        // TODO: Check for completion or failure

        // Emit current state so the UI stays live (age, iteration, etc.)
        self.ctx.emit_state(task);

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
