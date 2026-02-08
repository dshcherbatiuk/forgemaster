//! A2A client — tools for inter-agent communication.
//!
//! Exposes A2A operations as tools in the conversation loop,
//! allowing agents to send messages and discover peer capabilities.

pub mod executor;

pub use executor::A2aToolExecutor;
