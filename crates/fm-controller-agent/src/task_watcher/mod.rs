//! Watches AgentTask CRs and creates Orchestrator Agent CRs.

pub mod orchestrator_factory;
mod watcher;

pub use watcher::run;
