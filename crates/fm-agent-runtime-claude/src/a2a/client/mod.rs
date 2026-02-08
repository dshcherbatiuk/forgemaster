//! A2A client — tools for inter-agent communication.
//!
//! Exposes A2A operations as tools in the conversation loop,
//! allowing agents to send messages, discover peer capabilities,
//! and subscribe to task updates via SSE streaming.

pub mod executor;
pub mod sse_stream;

pub use executor::A2aToolExecutor;
