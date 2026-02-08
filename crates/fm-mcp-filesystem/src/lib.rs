//! Native Rust MCP server providing filesystem tools for agent workflows.
//!
//! Replaces the Node.js supergateway + @modelcontextprotocol/server-filesystem stack
//! with a native Rust implementation using the rmcp SDK.
//!
//! Exposes 7 filesystem tools via MCP Streamable HTTP transport at `/mcp`:
//! `read_file`, `write_file`, `edit_file`, `create_directory`,
//! `list_directory`, `directory_tree`, `search_files`.

pub mod action;
pub mod handler;
pub mod param;
pub mod workspace;

use axum::http::StatusCode;
use axum::response::IntoResponse;
use handler::FilesystemHandler;
use rmcp::transport::StreamableHttpServerConfig;
use rmcp::transport::streamable_http_server::StreamableHttpService;
use rmcp::transport::streamable_http_server::session::local::LocalSessionManager;
use tracing::info;

/// Default port for the MCP server.
const DEFAULT_PORT: u16 = 3000;

/// Starts the MCP server with `/mcp` and `/healthz` endpoints.
///
/// # Errors
///
/// Returns an error if the server fails to bind or start.
pub async fn start() -> anyhow::Result<()> {
    let port = std::env::var("MCP_SERVER_PORT")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(DEFAULT_PORT);

    let mcp_service = StreamableHttpService::new(
        move || Ok(FilesystemHandler::new()),
        LocalSessionManager::default().into(),
        StreamableHttpServerConfig::default(),
    );

    let router = axum::Router::new()
        .route("/healthz", axum::routing::get(healthz))
        .nest_service("/mcp", mcp_service);

    let listener = tokio::net::TcpListener::bind(("0.0.0.0", port)).await?;

    info!("🌐 fm-mcp-filesystem listening at http://0.0.0.0:{port}/mcp");

    axum::serve(listener, router).await?;

    Ok(())
}

/// Health check endpoint. Verifies workspace directory is accessible.
async fn healthz() -> impl IntoResponse {
    let root = workspace::workspace_root();
    match tokio::fs::metadata(&root).await {
        Ok(meta) if meta.is_dir() => (StatusCode::OK, format!("ok: {root}")),
        Ok(_) => (
            StatusCode::SERVICE_UNAVAILABLE,
            format!("workspace is not a directory: {root}"),
        ),
        Err(err) => (
            StatusCode::SERVICE_UNAVAILABLE,
            format!("workspace not accessible: {root}: {err}"),
        ),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_port() {
        assert_eq!(DEFAULT_PORT, 3000);
    }
}
