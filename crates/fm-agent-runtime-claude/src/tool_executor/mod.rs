//! Tool execution abstraction.
//!
//! Defines the `ToolExecutor` trait and implementations:
//! - `NoOpToolExecutor` — for agents without MCP servers
//! - `CompositeToolExecutor` — aggregates tools from multiple MCP servers
//! - `McpToolExecutor` — bridges to a single MCP server (in `mcp_client` module)

mod composite;
mod noop;

pub use composite::CompositeToolExecutor;
pub use noop::NoOpToolExecutor;

use anyhow::Result;
use async_trait::async_trait;

use crate::claude_api::tool_definition::ToolDefinition;

/// Result of a tool call execution.
#[derive(Debug, Clone)]
pub struct ToolCallResult {
    /// Text content returned by the tool.
    pub content: String,
    /// Whether the tool execution resulted in an error.
    pub is_error: bool,
}

/// Discovers and executes tools.
///
/// Implementations bridge to specific backends: MCP servers, mock tools, etc.
#[async_trait]
pub trait ToolExecutor: Send + Sync {
    /// Discovers available tool definitions from the backend.
    async fn discover_tools(&self) -> Result<Vec<ToolDefinition>>;

    /// Executes a tool call and returns the result.
    async fn execute_tool(&self, name: &str, input: &serde_json::Value) -> Result<ToolCallResult>;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tool_call_result_is_cloneable() {
        let result = ToolCallResult {
            content: "Agent created".to_string(),
            is_error: false,
        };
        let cloned = result.clone();
        assert_eq!(cloned.content, "Agent created");
        assert!(!cloned.is_error);
    }

    #[test]
    fn tool_call_result_error() {
        let result = ToolCallResult {
            content: "Connection refused".to_string(),
            is_error: true,
        };
        assert!(result.is_error);
    }
}
