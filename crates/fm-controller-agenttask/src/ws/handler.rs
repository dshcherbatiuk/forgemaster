//! Per-connection WebSocket handler.

use std::sync::Arc;

use axum::extract::ws::{Message, WebSocket};
use futures::{SinkExt, StreamExt};
use tokio::sync::mpsc;
use tracing::info;
use uuid::Uuid;

use super::connection_registry::ConnectionRegistry;
use super::event::WsEvent;
use super::schema_cache::SchemaCache;

/// Handles a single WebSocket connection lifecycle.
pub async fn handle_connection(
    socket: WebSocket,
    registry: Arc<ConnectionRegistry>,
    schema_cache: Arc<SchemaCache>,
) {
    let client_id = Uuid::new_v4().to_string();
    let (mut ws_sink, mut ws_stream) = socket.split();
    let (sender, mut receiver) = mpsc::unbounded_channel::<String>();

    registry.register(&client_id, sender);
    info!("🔌 Client connected: {client_id}");

    // Send Connected event
    let connected_event = WsEvent::Connected {
        client_id: client_id.clone(),
    };
    if let Ok(json) = serde_json::to_string(&connected_event) {
        let _ = ws_sink.send(Message::text(json)).await;
    }

    // Send cached dashboard data so late joiners get current state
    if let Some(data) = schema_cache.get("dashboard") {
        let data_event = WsEvent::Data { data };
        if let Ok(json) = serde_json::to_string(&data_event) {
            let _ = ws_sink.send(Message::text(json)).await;
        }
    }

    // Outbound: channel → WebSocket sink
    let outbound = tokio::spawn(async move {
        while let Some(message) = receiver.recv().await {
            if ws_sink.send(Message::text(message)).await.is_err() {
                break;
            }
        }
    });

    // Inbound: WebSocket stream → log
    let inbound_client_id = client_id.clone();
    let inbound = tokio::spawn(async move {
        while let Some(Ok(message)) = ws_stream.next().await {
            match message {
                Message::Text(text) => {
                    info!("📩 From {inbound_client_id}: {text}");
                }
                Message::Close(_) => break,
                _ => {}
            }
        }
    });

    // Wait for either task to finish (client disconnected)
    tokio::select! {
        _ = outbound => {},
        _ = inbound => {},
    }

    registry.unregister(&client_id);
    info!("🔌 Client disconnected: {client_id}");
}
