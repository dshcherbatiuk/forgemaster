//! Agent reconciliation strategies.
//!
//! Each phase has its own reconcile strategy implementing the `ReconcileStrategy` trait.

mod failed;
mod pending;
mod running;
mod succeeded;

use async_trait::async_trait;
use kube::runtime::controller::Action;

use crate::crd::Agent;

use super::error::ReconcileResult;

pub use failed::FailedStrategy;
pub use pending::PendingStrategy;
pub use running::RunningStrategy;
pub use succeeded::SucceededStrategy;

/// Trait for phase-specific reconciliation logic.
#[async_trait]
pub trait ReconcileStrategy: Send + Sync {
    /// Reconciles the agent according to the strategy.
    async fn reconcile(&self, agent: &Agent) -> ReconcileResult<Action>;
}
