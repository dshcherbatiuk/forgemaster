//! MCP client using the official `rmcp` SDK.
//!
//! Connects to an MCP server via Streamable HTTP transport,
//! discovers tools, and executes tool calls.

use anyhow::Result;
use rmcp::{
    ServiceExt,
    model::{
        CallToolRequestParams, CallToolResult, ClientCapabilities, ClientInfo,
        Implementation, Tool,
    },
    service::RunningService,
    transport::StreamableHttpClientTransport,
};
use tracing::{debug, info};

/// Type alias for the connected MCP service.
///
/// The second parameter is `ClientInfo` which acts as the client handler.
type McpService = RunningService<rmcp::RoleClient, ClientInfo>;

/// MCP client that connects to a local MCP server via Streamable HTTP.
///
/// Uses the official `rmcp` SDK for protocol-correct MCP communication.
/// See [ADR-0006](../../../docs/adr/0006-mcp-client-sdk-selection.md).
pub struct McpClient {
    service: McpService,
}

impl McpClient {
    /// Connects to an MCP server at the given endpoint URL.
    ///
    /// Performs the MCP protocol handshake (initialize → initialized).
    pub async fn connect(endpoint_url: &str) -> Result<Self> {
        debug!("🔌 Connecting to MCP server at {endpoint_url}");

        let transport = StreamableHttpClientTransport::from_uri(endpoint_url);

        let client_info = ClientInfo {
            meta: None,
            protocol_version: Default::default(),
            capabilities: ClientCapabilities::default(),
            client_info: Implementation {
                name: "fm-agent-runtime-claude".to_string(),
                title: None,
                version: env!("CARGO_PKG_VERSION").to_string(),
                website_url: None,
                icons: None,
            },
        };

        let service = client_info.serve(transport).await?;

        if let Some(server_info) = service.peer_info() {
            info!(
                "🔌 Connected to MCP server: {}",
                server_info.server_info.name
            );
        }

        Ok(Self { service })
    }

    /// Discovers available tools from the MCP server.
    ///
    /// Handles pagination automatically via `list_all_tools`.
    pub async fn list_tools(&self) -> Result<Vec<Tool>> {
        debug!("📡 MCP tools/list");
        let tools: Vec<Tool> = self.service.list_all_tools().await?;
        debug!("📡 MCP returned {} tool(s)", tools.len());
        Ok(tools)
    }

    /// Executes a tool call on the MCP server.
    pub async fn call_tool(
        &self,
        name: &str,
        arguments: &serde_json::Value,
    ) -> Result<CallToolResult> {
        debug!("📡 MCP tools/call({name})");

        let result = self
            .service
            .call_tool(CallToolRequestParams {
                meta: None,
                name: name.to_string().into(),
                arguments: arguments.as_object().cloned(),
                task: None,
            })
            .await?;

        Ok(result)
    }

    /// Gracefully closes the connection to the MCP server.
    pub async fn close(self) -> Result<()> {
        debug!("🔌 Closing MCP connection");
        self.service.cancel().await?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn client_info_has_correct_name() {
        let info = ClientInfo {
            meta: None,
            protocol_version: Default::default(),
            capabilities: ClientCapabilities::default(),
            client_info: Implementation {
                name: "fm-agent-runtime-claude".to_string(),
                title: None,
                version: env!("CARGO_PKG_VERSION").to_string(),
                website_url: None,
                icons: None,
            },
        };
        assert_eq!(info.client_info.name, "fm-agent-runtime-claude");
    }

    #[test]
    fn call_tool_params_with_arguments() {
        let args = serde_json::json!({"agent_type": "code-gen", "task": "build api"});
        let params = CallToolRequestParams {
            meta: None,
            name: "create_agent".to_string().into(),
            arguments: args.as_object().cloned(),
            task: None,
        };
        assert_eq!(params.name.as_ref(), "create_agent");
        assert!(params.arguments.is_some());
        let arguments = params.arguments.as_ref().expect("arguments");
        assert_eq!(arguments["agent_type"], "code-gen");
    }

    #[test]
    fn call_tool_params_without_arguments() {
        let params = CallToolRequestParams {
            meta: None,
            name: "list_agents".to_string().into(),
            arguments: None,
            task: None,
        };
        assert_eq!(params.name.as_ref(), "list_agents");
        assert!(params.arguments.is_none());
    }
}
