//! AgentTask CRD types.

mod phase;
mod spec;
mod status;
mod task;

pub use phase::AgentTaskPhase;
pub use spec::AgentTaskSpec;
pub use status::AgentTaskStatus;
pub use task::{AgentTask, AgentTaskCrd};
