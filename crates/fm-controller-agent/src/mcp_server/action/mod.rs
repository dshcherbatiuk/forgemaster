//! MCP tool action strategies.
//!
//! Each MCP tool delegates to its own action implementing the `McpAction` trait.

mod create_agent;
mod get_agent;
mod list_agents;

use async_trait::async_trait;
use rmcp::model::CallToolResult;

pub use create_agent::CreateAgentAction;
pub use get_agent::GetAgentAction;
pub use list_agents::ListAgentsAction;

/// Trait for MCP tool action strategies.
#[async_trait]
pub trait McpAction: Send + Sync {
    /// Parameters this action accepts.
    type Params: Send;

    /// Executes the action against the Kubernetes API.
    async fn execute(&self, params: Self::Params) -> CallToolResult;
}
