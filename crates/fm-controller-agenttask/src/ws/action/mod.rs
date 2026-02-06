//! WebSocket action handlers.
//!
//! Each `WsCommand` variant that triggers side effects has a corresponding
//! action handler implementing the `WsAction` trait. The `ActionDispatcher`
//! routes commands to the right handler via a DashMap keyed by command type.

mod dispatcher;
mod submit_task;

use anyhow::Result;
use async_trait::async_trait;

pub use dispatcher::ActionDispatcher;
pub use submit_task::SubmitTaskAction;

/// Trait for handling WebSocket commands that produce side effects.
#[async_trait]
pub trait WsAction: Send + Sync {
    /// Executes the action with the given command payload.
    async fn execute(&self, payload: &serde_json::Value, client_id: &str) -> Result<()>;
}
