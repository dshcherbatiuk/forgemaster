//! Agent controller.

mod context;
mod dispatcher;
mod error;
mod reconciler;
mod runner;

pub use context::{create_context, ControllerContext};
pub use dispatcher::Dispatcher;
pub use error::{ReconcileError, ReconcileResult};
pub use reconciler::{
    FailedStrategy, PendingStrategy, ReconcileStrategy, RunningStrategy, SucceededStrategy,
};
pub use runner::run;
