//! Agent CRD core types.

mod crd;
mod phase;
mod spec;
mod status;

pub use crd::{Agent, AgentCrd};
pub use phase::AgentPhase;
pub use spec::AgentSpec;
pub use status::AgentStatus;
