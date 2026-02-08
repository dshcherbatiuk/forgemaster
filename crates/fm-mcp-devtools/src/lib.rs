//! MCP server providing Docker and Helm tools for agent DevOps workflows.
//!
//! Exposes `docker_build`, `helm_install`, and `helm_status` tools
//! via the MCP protocol (Streamable HTTP transport).

pub mod action;
pub mod handler;
pub mod param;
pub mod workspace;

use handler::DevtoolsHandler;
use rmcp::transport::streamable_http_server::StreamableHttpService;
use rmcp::transport::streamable_http_server::session::local::LocalSessionManager;
use tracing::info;

/// Default port for the MCP server.
const DEFAULT_PORT: u16 = 3000;

/// Starts the MCP server.
///
/// Serves devtools (`docker_build`, `helm_install`, `helm_status`)
/// via Streamable HTTP transport at `/mcp`.
///
/// # Errors
///
/// Returns an error if the server fails to bind or start.
pub async fn start() -> anyhow::Result<()> {
    let port = std::env::var("MCP_SERVER_PORT")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(DEFAULT_PORT);

    let service = StreamableHttpService::new(
        move || Ok(DevtoolsHandler::new()),
        LocalSessionManager::default().into(),
        rmcp::transport::StreamableHttpServerConfig::default(),
    );

    let router = axum::Router::new().nest_service("/mcp", service);
    let listener = tokio::net::TcpListener::bind(("0.0.0.0", port)).await?;

    info!("🌐 fm-mcp-devtools listening at http://0.0.0.0:{port}/mcp");

    axum::serve(listener, router).await?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_port() {
        assert_eq!(DEFAULT_PORT, 3000);
    }
}
