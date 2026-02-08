//! No-op tool executor for agents without MCP server configuration.

use anyhow::{Result, bail};
use async_trait::async_trait;

use crate::claude_api::tool_definition::ToolDefinition;
use super::{ToolCallResult, ToolExecutor};

/// Returns no tools and fails fast on any tool execution attempt.
///
/// Used when the agent has no MCP servers configured.
pub struct NoOpToolExecutor;

#[async_trait]
impl ToolExecutor for NoOpToolExecutor {
    async fn discover_tools(&self) -> Result<Vec<ToolDefinition>> {
        Ok(Vec::new())
    }

    async fn execute_tool(&self, name: &str, _input: &serde_json::Value) -> Result<ToolCallResult> {
        bail!("no tool executor configured, cannot execute tool: {name}")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn discover_tools_returns_empty() {
        let executor = NoOpToolExecutor;
        let tools = executor.discover_tools().await.unwrap();
        assert!(tools.is_empty());
    }

    #[tokio::test]
    async fn execute_tool_fails() {
        let executor = NoOpToolExecutor;
        let result = executor
            .execute_tool("create_agent", &serde_json::json!({}))
            .await;
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("create_agent"));
    }
}
