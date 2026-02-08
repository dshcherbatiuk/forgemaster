//! MCP client using the official `rmcp` SDK.
//!
//! Connects to an MCP server via Streamable HTTP transport,
//! discovers tools, and executes tool calls.
//! Automatically reconnects on transport errors (e.g. SSE stream EOF).

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
use tokio::sync::Mutex;
use tracing::{debug, info, warn};

/// Type alias for the connected MCP service.
///
/// The second parameter is `ClientInfo` which acts as the client handler.
type McpService = RunningService<rmcp::RoleClient, ClientInfo>;

/// MCP client that connects to a local MCP server via Streamable HTTP.
///
/// Uses the official `rmcp` SDK for protocol-correct MCP communication.
/// Reconnects automatically when the transport dies (e.g. supergateway
/// closes the SSE stream after `initialize`).
///
/// See [ADR-0006](../../../docs/adr/0006-mcp-client-sdk-selection.md).
pub struct McpClient {
    endpoint_url: String,
    service: Mutex<McpService>,
}

impl McpClient {
    /// Maximum number of connection attempts.
    const MAX_RETRIES: u32 = 5;

    /// Initial delay between retries (doubles each attempt).
    const INITIAL_RETRY_DELAY: std::time::Duration = std::time::Duration::from_secs(2);

    /// Connects to an MCP server at the given endpoint URL with retry.
    ///
    /// Retries up to [`Self::MAX_RETRIES`] times with exponential backoff.
    pub async fn connect(endpoint_url: &str) -> Result<Self> {
        let service = Self::connect_with_retry(endpoint_url).await?;
        Ok(Self {
            endpoint_url: endpoint_url.to_string(),
            service: Mutex::new(service),
        })
    }

    /// Discovers available tools from the MCP server.
    ///
    /// Handles pagination automatically via `list_all_tools`.
    /// Reconnects once on transport error.
    pub async fn list_tools(&self) -> Result<Vec<Tool>> {
        debug!("📡 MCP tools/list({})", self.endpoint_url);

        let service = self.service.lock().await;
        match service.list_all_tools().await {
            Ok(tools) => {
                debug!("📡 MCP returned {} tool(s)", tools.len());
                Ok(tools)
            }
            Err(err) => {
                drop(service);
                warn!(
                    "📡 MCP tools/list failed ({}), reconnecting: {err:#}",
                    self.endpoint_url
                );
                self.reconnect().await?;
                let service = self.service.lock().await;
                let tools = service.list_all_tools().await?;
                debug!("📡 MCP returned {} tool(s) after reconnect", tools.len());
                Ok(tools)
            }
        }
    }

    /// Executes a tool call on the MCP server.
    ///
    /// Reconnects once on transport error.
    pub async fn call_tool(
        &self,
        name: &str,
        arguments: &serde_json::Value,
    ) -> Result<CallToolResult> {
        debug!("📡 MCP tools/call({name})");

        let params = Self::build_call_params(name, arguments);
        let service = self.service.lock().await;
        match service.call_tool(params).await {
            Ok(result) => Ok(result),
            Err(err) => {
                drop(service);
                warn!(
                    "📡 MCP tools/call({name}) failed ({}), reconnecting: {err:#}",
                    self.endpoint_url
                );
                self.reconnect().await?;
                let retry_params = Self::build_call_params(name, arguments);
                let service = self.service.lock().await;
                Ok(service.call_tool(retry_params).await?)
            }
        }
    }

    /// Gracefully closes the connection to the MCP server.
    pub async fn close(self) -> Result<()> {
        debug!("🔌 Closing MCP connection to {}", self.endpoint_url);
        let service = self.service.into_inner();
        service.cancel().await?;
        Ok(())
    }

    /// Replaces the dead service with a fresh connection.
    async fn reconnect(&self) -> Result<()> {
        info!("🔌 Reconnecting to MCP server at {}", self.endpoint_url);
        let new_service = Self::connect_with_retry(&self.endpoint_url).await?;
        let mut service = self.service.lock().await;
        *service = new_service;
        Ok(())
    }

    /// Connects with exponential backoff retry.
    ///
    /// Each attempt creates the service AND verifies it works by listing tools.
    /// This catches the case where `initialize` succeeds but the SSE stream
    /// dies immediately after (supergateway closing the stream).
    async fn connect_with_retry(endpoint_url: &str) -> Result<McpService> {
        let mut delay = Self::INITIAL_RETRY_DELAY;

        for attempt in 1..=Self::MAX_RETRIES {
            match Self::create_and_verify(endpoint_url).await {
                Ok(service) => return Ok(service),
                Err(err) if attempt < Self::MAX_RETRIES => {
                    warn!(
                        "🔌 MCP connection attempt {attempt}/{} to {endpoint_url} failed: {err:#}, retrying in {delay:?}",
                        Self::MAX_RETRIES
                    );
                    tokio::time::sleep(delay).await;
                    delay *= 2;
                }
                Err(err) => {
                    return Err(err.context(format!(
                        "MCP connection to {endpoint_url} failed after {} attempts",
                        Self::MAX_RETRIES
                    )));
                }
            }
        }

        unreachable!()
    }

    /// Creates a service and verifies it works by listing tools.
    async fn create_and_verify(endpoint_url: &str) -> Result<McpService> {
        let service = Self::create_service(endpoint_url).await?;

        // Verify the transport is alive — the SSE stream may have closed
        // right after initialize, leaving a dead service.
        service
            .list_all_tools()
            .await
            .map_err(|e| anyhow::anyhow!("MCP health check (tools/list) failed: {e:#}"))?;

        info!("🔌 MCP connection verified for {endpoint_url}");
        Ok(service)
    }

    /// Single connection attempt — creates transport, performs MCP initialize handshake.
    async fn create_service(endpoint_url: &str) -> Result<McpService> {
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

        Ok(service)
    }

    fn build_call_params(name: &str, arguments: &serde_json::Value) -> CallToolRequestParams {
        CallToolRequestParams {
            meta: None,
            name: name.to_string().into(),
            arguments: arguments.as_object().cloned(),
            task: None,
        }
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
        let params = McpClient::build_call_params("create_agent", &args);
        assert_eq!(params.name.as_ref(), "create_agent");
        assert!(params.arguments.is_some());
        let arguments = params.arguments.as_ref().expect("arguments");
        assert_eq!(arguments["agent_type"], "code-gen");
    }

    #[test]
    fn call_tool_params_without_arguments() {
        let args = serde_json::json!(null);
        let params = McpClient::build_call_params("list_agents", &args);
        assert_eq!(params.name.as_ref(), "list_agents");
        assert!(params.arguments.is_none());
    }

    #[test]
    fn build_call_params_extracts_object() {
        let args = serde_json::json!({"key": "value"});
        let params = McpClient::build_call_params("test_tool", &args);
        assert!(params.arguments.is_some());
    }

    #[test]
    fn build_call_params_null_is_none() {
        let args = serde_json::Value::Null;
        let params = McpClient::build_call_params("test_tool", &args);
        assert!(params.arguments.is_none());
    }
}
