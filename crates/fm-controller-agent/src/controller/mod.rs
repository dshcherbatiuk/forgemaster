//! Agent controller.

mod context;
mod dispatcher;
mod error;
mod reconciler;
mod runner;

pub use context::{ControllerContext, create_context};
pub use dispatcher::Dispatcher;
pub use error::{ReconcileError, ReconcileResult};
pub use reconciler::{
    FailedStrategy, PendingStrategy, ReconcileStrategy, RunningStrategy, SucceededStrategy,
};
pub use runner::run;
