//! Claude Messages API client with streaming SSE support.

pub mod client;
mod error;
pub mod request;
pub mod response;
pub mod retry_policy;
pub mod sse;

pub use client::ClaudeClient;
pub use error::ClaudeApiError;
pub use request::{Message, MessagesRequest, Role};
pub use response::{MessagesResponse, StopReason, Usage};
pub use retry_policy::RetryPolicy;
pub use sse::{SseDispatcher, SseEvent};
