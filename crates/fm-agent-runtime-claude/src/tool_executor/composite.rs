//! Composite tool executor that aggregates tools from multiple MCP servers.
//!
//! Routes tool calls to the correct server based on tool name registration.
//!
//! ## Future: Dynamic MCP Server Registry
//!
//! Currently, MCP server URLs are provided statically via env vars at startup.
//! In the future, this should evolve into a registry pattern with dynamic
//! MCP server discovery — allowing servers to be added/removed at runtime
//! (e.g., via K8s watch on MCPServer CRDs). This is out of scope for now.

use anyhow::Result;
use async_trait::async_trait;
use dashmap::DashMap;
use tracing::{debug, info};

use crate::claude_api::tool_definition::ToolDefinition;
use crate::mcp_client::McpToolExecutor;
use super::{ToolCallResult, ToolExecutor};

/// Aggregates tools from multiple MCP servers.
///
/// On `discover_tools()`, connects to each server and merges their tool lists.
/// On `execute_tool()`, routes the call to the server that provides the tool.
pub struct CompositeToolExecutor {
    /// Individual executors, one per MCP server.
    executors: Vec<McpToolExecutor>,
    /// Maps tool name → index into `executors`.
    tool_routing: DashMap<String, usize>,
}

impl CompositeToolExecutor {
    /// Creates a composite executor by connecting to all given MCP server URLs.
    pub async fn connect(server_urls: &[String]) -> Result<Self> {
        let mut executors = Vec::with_capacity(server_urls.len());

        for url in server_urls {
            let executor = McpToolExecutor::connect(url).await?;
            executors.push(executor);
        }

        info!(
            "🔌 Connected to {} MCP server(s)",
            executors.len()
        );

        Ok(Self {
            executors,
            tool_routing: DashMap::new(),
        })
    }
}

#[async_trait]
impl ToolExecutor for CompositeToolExecutor {
    async fn discover_tools(&self) -> Result<Vec<ToolDefinition>> {
        self.tool_routing.clear();
        let mut all_tools = Vec::new();

        for (index, executor) in self.executors.iter().enumerate() {
            let tools = executor.discover_tools().await?;

            for tool in &tools {
                debug!("📡 Registered tool '{}' → server #{index}", tool.name);
                self.tool_routing.insert(tool.name.clone(), index);
            }

            all_tools.extend(tools);
        }

        info!(
            "🔧 Discovered {} tool(s) from {} server(s)",
            all_tools.len(),
            self.executors.len()
        );

        Ok(all_tools)
    }

    async fn execute_tool(&self, name: &str, input: &serde_json::Value) -> Result<ToolCallResult> {
        let index = self
            .tool_routing
            .get(name)
            .map(|entry| *entry.value())
            .ok_or_else(|| {
                anyhow::anyhow!("tool '{name}' not found in any connected MCP server")
            })?;

        self.executors[index].execute_tool(name, input).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tool_routing_maps_correctly() {
        let routing: DashMap<String, usize> = DashMap::new();
        routing.insert("create_pr".to_string(), 0);
        routing.insert("read_file".to_string(), 1);
        routing.insert("list_issues".to_string(), 0);

        assert_eq!(*routing.get("create_pr").expect("exists").value(), 0);
        assert_eq!(*routing.get("read_file").expect("exists").value(), 1);
        assert_eq!(*routing.get("list_issues").expect("exists").value(), 0);
    }

    #[test]
    fn tool_routing_unknown_tool_returns_none() {
        let routing: DashMap<String, usize> = DashMap::new();
        routing.insert("create_pr".to_string(), 0);

        assert!(routing.get("unknown_tool").is_none());
    }

    #[test]
    fn tool_routing_clear_removes_all() {
        let routing: DashMap<String, usize> = DashMap::new();
        routing.insert("create_pr".to_string(), 0);
        routing.insert("read_file".to_string(), 1);

        routing.clear();
        assert!(routing.is_empty());
    }
}
