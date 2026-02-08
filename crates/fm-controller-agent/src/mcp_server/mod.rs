//! MCP server for agent lifecycle management.
//!
//! Exposes tools via the MCP protocol (Streamable HTTP transport)
//! so that orchestrator agents can create, list, and inspect agents.

pub mod action;
mod handler;
pub mod param;

pub use handler::AgentMcpHandler;
pub use param::{CreateAgentParams, GetAgentParams, ListAgentsParams};

use crate::controller::McpServerRefs;

use kube::Client;
use rmcp::transport::streamable_http_server::session::local::LocalSessionManager;
use rmcp::transport::streamable_http_server::StreamableHttpService;
use tracing::info;

/// Default port for the MCP server.
const DEFAULT_MCP_PORT: u16 = 3000;

/// Starts the MCP server on the given port.
///
/// Serves agent lifecycle tools (`create_agent`, `list_agents`, `get_agent_status`)
/// via Streamable HTTP transport at `/mcp`.
pub async fn start(client: Client, default_model: &str, default_max_tokens: i32, default_mcp_servers: &[crate::crd::McpServerRef]) -> anyhow::Result<()> {
    let port = std::env::var("MCP_SERVER_PORT")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(DEFAULT_MCP_PORT);

    let default_model = default_model.to_string();
    let default_mcp_servers: McpServerRefs = default_mcp_servers.into();
    let service = StreamableHttpService::new(
        move || Ok(AgentMcpHandler::new(client.clone(), default_model.clone(), default_max_tokens, default_mcp_servers.clone())),
        LocalSessionManager::default().into(),
        Default::default(),
    );

    let router = axum::Router::new().nest_service("/mcp", service);
    let listener = tokio::net::TcpListener::bind(("0.0.0.0", port)).await?;

    info!("🌐 MCP server listening at http://0.0.0.0:{port}/mcp");

    axum::serve(listener, router).await?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_mcp_port() {
        assert_eq!(DEFAULT_MCP_PORT, 3000);
    }
}
