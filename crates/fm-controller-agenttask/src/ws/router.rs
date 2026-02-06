//! WebSocket route definition.

use std::sync::Arc;

use axum::extract::{State, WebSocketUpgrade};
use axum::response::IntoResponse;
use axum::routing::get;
use axum::Router;

use super::connection_registry::ConnectionRegistry;
use super::handler::handle_connection;
use super::schema_cache::SchemaCache;

/// Shared state for the WebSocket route.
#[derive(Clone)]
pub struct WsState {
    /// Tracks connected WebSocket clients.
    pub registry: Arc<ConnectionRegistry>,
    /// Cached dashboard data for late joiners.
    pub schema_cache: Arc<SchemaCache>,
}

/// Creates the router with the `/ws` endpoint.
pub fn ws_router(state: WsState) -> Router {
    Router::new().route("/ws", get(ws_upgrade)).with_state(state)
}

async fn ws_upgrade(
    ws: WebSocketUpgrade,
    State(state): State<WsState>,
) -> impl IntoResponse {
    ws.on_upgrade(move |socket| {
        handle_connection(socket, state.registry, state.schema_cache)
    })
}
