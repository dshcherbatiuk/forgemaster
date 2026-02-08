//! `ToolExecutor` implementation backed by an MCP server via `rmcp`.

use anyhow::Result;
use async_trait::async_trait;
use tracing::debug;

use super::client::McpClient;
use super::tool_converter;
use crate::claude_api::tool_definition::ToolDefinition;
use crate::tool_executor::{ToolCallResult, ToolExecutor};

/// Executes tools by forwarding calls to an MCP server.
///
/// Wraps `McpClient` and implements `ToolExecutor` to bridge
/// the conversation loop with the MCP server.
pub struct McpToolExecutor {
    client: McpClient,
}

impl McpToolExecutor {
    /// Creates an executor connected to the given MCP server URL.
    pub async fn connect(endpoint_url: &str) -> Result<Self> {
        let client = McpClient::connect(endpoint_url).await?;
        Ok(Self { client })
    }
}

#[async_trait]
impl ToolExecutor for McpToolExecutor {
    async fn discover_tools(&self) -> Result<Vec<ToolDefinition>> {
        let mcp_tools = self.client.list_tools().await?;
        Ok(tool_converter::to_claude_tools(&mcp_tools))
    }

    async fn execute_tool(&self, name: &str, input: &serde_json::Value) -> Result<ToolCallResult> {
        let result = self.client.call_tool(name, input).await?;

        let content = result
            .content
            .iter()
            .filter_map(|c| c.as_text().map(|t| t.text.clone()))
            .collect::<Vec<_>>()
            .join("\n");

        let is_error = result.is_error.unwrap_or(false);

        debug!(
            "📡 MCP tool {name} → {} chars, is_error={is_error}",
            content.len()
        );

        Ok(ToolCallResult { content, is_error })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tool_call_result_from_text() {
        let result = ToolCallResult {
            content: "Agent created: code-gen-123".to_string(),
            is_error: false,
        };
        assert_eq!(result.content, "Agent created: code-gen-123");
        assert!(!result.is_error);
    }

    #[test]
    fn tool_call_result_error() {
        let result = ToolCallResult {
            content: "Agent not found".to_string(),
            is_error: true,
        };
        assert!(result.is_error);
    }
}
