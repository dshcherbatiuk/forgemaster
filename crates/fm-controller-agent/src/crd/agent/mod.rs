//! Agent CRD core types.

mod crd;
mod phase;
mod status;

pub use crd::{Agent, AgentCrd};
pub use phase::AgentPhase;
pub use status::AgentStatus;
