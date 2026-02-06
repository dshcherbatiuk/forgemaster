//! WebSocket server that runs alongside the K8s controller.

use std::sync::Arc;

use anyhow::Result;
use kube::Client;
use serde_json::json;
use tokio::net::TcpListener;
use tower_http::cors::CorsLayer;
use tracing::info;

use super::action::ActionDispatcher;
use super::connection_registry::ConnectionRegistry;
use super::router::{WsState, ws_router};
use super::schema_cache::SchemaCache;
use super::task_creator::TaskCreator;

/// WebSocket server exposing the `/ws` endpoint.
pub struct WsServer {
    /// Port to bind the WebSocket listener.
    port: u16,
    /// Tracks connected clients.
    registry: Arc<ConnectionRegistry>,
    /// Cached dashboard data pushed to late joiners.
    schema_cache: Arc<SchemaCache>,
    /// Dispatches commands to action handlers.
    dispatcher: Arc<ActionDispatcher>,
}

impl WsServer {
    /// Creates a new server on the given port with K8s client for task creation.
    pub fn new(port: u16, client: Client, namespace: String) -> Self {
        let schema_cache = Arc::new(SchemaCache::new());
        let registry = Arc::new(ConnectionRegistry::new());
        let task_creator = Arc::new(TaskCreator::new(client, namespace));
        let dispatcher = Arc::new(ActionDispatcher::new(
            task_creator,
            Arc::clone(&registry),
        ));

        // Seed initial dashboard state matching the UI's defaultData
        schema_cache.set(
            "dashboard",
            json!({
                "hero": {
                    "tagline": "AI-Powered Code Generation",
                    "subtitle": "Describe your task. Let agents build it. Tests drive the loop."
                },
                "task": {
                    "description": ""
                }
            }),
        );

        Self {
            port,
            registry,
            schema_cache,
            dispatcher,
        }
    }

    /// Start the server. Blocks until shutdown.
    pub async fn run(self) -> Result<()> {
        let state = WsState {
            registry: self.registry,
            schema_cache: self.schema_cache,
            dispatcher: self.dispatcher,
        };

        let app = ws_router(state).layer(CorsLayer::permissive());

        let listener = TcpListener::bind(format!("0.0.0.0:{}", self.port)).await?;
        info!("🌐 WebSocket server listening on port {}", self.port);

        axum::serve(listener, app).await?;
        Ok(())
    }
}
