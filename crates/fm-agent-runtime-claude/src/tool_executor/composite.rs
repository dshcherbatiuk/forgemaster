//! Composite tool executor that aggregates tools from multiple backends.
//!
//! Routes tool calls to the correct executor based on tool name registration.
//! Supports both MCP servers and A2A tools.

use anyhow::Result;
use async_trait::async_trait;
use dashmap::DashMap;
use tracing::{debug, info};

use crate::claude_api::tool_definition::ToolDefinition;
use crate::mcp_client::McpToolExecutor;
use super::{ToolCallResult, ToolExecutor};

/// Aggregates tools from multiple executors (MCP, A2A, etc.).
///
/// On `discover_tools()`, queries each executor and merges their tool lists.
/// On `execute_tool()`, routes the call to the executor that registered the tool.
pub struct CompositeToolExecutor {
    /// Individual executors (MCP servers, A2A, etc.).
    executors: Vec<Box<dyn ToolExecutor>>,
    /// Maps tool name → index into `executors`.
    tool_routing: DashMap<String, usize>,
}

impl CompositeToolExecutor {
    /// Creates an empty composite executor.
    #[must_use]
    pub fn empty() -> Self {
        Self {
            executors: Vec::new(),
            tool_routing: DashMap::new(),
        }
    }

    /// Creates a composite executor by connecting to all given MCP server URLs.
    pub async fn connect(server_urls: &[String]) -> Result<Self> {
        let mut composite = Self::empty();

        for url in server_urls {
            let executor = McpToolExecutor::connect(url).await?;
            composite.executors.push(Box::new(executor));
        }

        info!(
            "🔌 Connected to {} MCP server(s)",
            composite.executors.len()
        );

        Ok(composite)
    }

    /// Adds a tool executor to the composite.
    pub fn add(&mut self, executor: Box<dyn ToolExecutor>) {
        self.executors.push(executor);
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
                debug!("📡 Registered tool '{}' → executor #{index}", tool.name);
                self.tool_routing.insert(tool.name.clone(), index);
            }

            all_tools.extend(tools);
        }

        info!(
            "🔧 Discovered {} tool(s) from {} executor(s)",
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
                anyhow::anyhow!("tool '{name}' not found in any connected executor")
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

    #[test]
    fn empty_composite_has_no_executors() {
        let composite = CompositeToolExecutor::empty();
        assert!(composite.executors.is_empty());
        assert!(composite.tool_routing.is_empty());
    }

    #[tokio::test]
    async fn empty_composite_discovers_no_tools() {
        let composite = CompositeToolExecutor::empty();
        let tools = composite.discover_tools().await.expect("discover");
        assert!(tools.is_empty());
    }
}
