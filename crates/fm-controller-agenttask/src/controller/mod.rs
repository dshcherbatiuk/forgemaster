//! AgentTask controller.

mod context;
mod dispatcher;
mod error;
mod namespace_lifecycle;
mod reconciler;
mod runner;

pub use context::{ControllerContext, create_context};
pub use dispatcher::Dispatcher;
pub use error::{ReconcileError, ReconcileResult};
pub use namespace_lifecycle::NamespaceLifecycle;
pub use reconciler::{
    ClarifyingStrategy, FailedStrategy, PendingStrategy, ReconcileStrategy, RunningStrategy,
    SucceededStrategy,
};
pub use runner::run;
