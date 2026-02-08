//! Agent controller.

mod context;
mod dispatcher;
mod error;
mod event_recorder;
pub mod pod;
pub mod rbac_propagator;
mod reconciler;
mod runner;
pub mod secret_propagator;
pub mod service_creator;

pub use context::{ControllerContext, McpServerRefs, create_context};
pub use dispatcher::Dispatcher;
pub use error::{ReconcileError, ReconcileResult};
pub use reconciler::{
    FailedStrategy, PendingStrategy, ReconcileStrategy, RunningStrategy, SucceededStrategy,
};
pub use runner::run;
