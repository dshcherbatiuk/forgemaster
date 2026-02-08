//! A2A server — embedded in each agent runtime pod.
//!
//! Serves the Agent Card at `/.well-known/agent-card.json`
//! and accepts incoming messages from peer agents via JSON-RPC.
//! Supports SSE streaming for real-time task status updates.

pub mod a2a_message_converter;
pub mod agent_card_builder;
pub mod conversation_deps;
pub mod message_handler;

use std::sync::{Arc, OnceLock};

use a2a_rs_server::A2aServer;
use anyhow::Result;
use tracing::info;

use conversation_deps::ConversationDeps;
use message_handler::{AgentMessageHandler, EventSender};

/// Starts the A2A server on the given port.
///
/// Constructs the handler internally and wires the broadcast event sender
/// from the server to the handler via `OnceLock`, enabling SSE streaming.
///
/// When `conversation_deps` is provided, incoming A2A messages are processed
/// through the Claude conversation loop. Without it, the handler returns
/// a stub acknowledgment.
///
/// This function blocks until the server is stopped.
///
/// # Errors
///
/// Returns an error if the server fails to bind to the address or encounters a runtime error.
pub async fn start(
    agent_name: &str,
    agent_type: &str,
    namespace: &str,
    port: u16,
    conversation_deps: Option<ConversationDeps>,
) -> Result<()> {
    let addr = format!("0.0.0.0:{port}");
    info!("🌐 A2A server starting on {addr} (streaming enabled)");

    let event_sender: EventSender = Arc::new(OnceLock::new());
    let handler = AgentMessageHandler::new(
        agent_name.to_string(),
        agent_type.to_string(),
        namespace.to_string(),
        port,
        event_sender.clone(),
        conversation_deps,
    );

    let server = A2aServer::new(handler);
    event_sender.set(server.get_event_sender()).ok();

    server.bind(&addr)?.run().await
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn event_sender_starts_empty() {
        let sender: EventSender = Arc::new(OnceLock::new());
        assert!(sender.get().is_none());
    }

    #[test]
    fn handler_created_for_server() {
        let handler = AgentMessageHandler::new(
            "test-gen".to_string(),
            "test-generator".to_string(),
            "task-abc".to_string(),
            9090,
            Arc::new(OnceLock::new()),
            None,
        );
        let card =
            a2a_rs_server::MessageHandler::agent_card(&handler, "http://0.0.0.0:9090");
        assert_eq!(card.name, "test-gen");
        // Should use real Service DNS URL, not the bind address
        assert_eq!(
            card.supported_interfaces[0].url,
            "http://test-gen.task-abc.svc.cluster.local:9090/v1/rpc"
        );
    }

    #[test]
    fn default_port_format() {
        let addr = format!("0.0.0.0:{}", 9090_u16);
        assert_eq!(addr, "0.0.0.0:9090");
    }
}
