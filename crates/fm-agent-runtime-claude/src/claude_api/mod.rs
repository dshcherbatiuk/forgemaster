//! Claude Messages API client with streaming SSE support.

pub mod client;
pub mod content_block;
mod error;
pub mod request;
pub mod response;
pub mod retry_policy;
pub mod sse;
pub mod tool_definition;
pub mod response_collector;
pub mod tool_use_block;

pub use client::ClaudeClient;
pub use content_block::RequestContentBlock;
pub use error::ClaudeApiError;
pub use request::{Message, MessageContent, MessageRole, MessagesRequest};
pub use response::{MessagesResponse, StopReason, Usage};
pub use retry_policy::RetryPolicy;
pub use sse::{SseDispatcher, SseEvent};
pub use tool_definition::ToolDefinition;
pub use tool_use_block::ToolUseBlock;
