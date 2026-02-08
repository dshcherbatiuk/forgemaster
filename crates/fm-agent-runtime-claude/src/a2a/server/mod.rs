//! A2A server — embedded in each agent runtime pod.
//!
//! Serves the Agent Card at `/.well-known/agent-card.json`
//! and accepts incoming messages from peer agents via JSON-RPC.

pub mod agent_card_builder;
pub mod message_handler;

use a2a_rs_server::A2aServer;
use anyhow::Result;
use tracing::info;

use message_handler::AgentMessageHandler;

/// Starts the A2A server on the given port.
///
/// The server listens for incoming A2A messages from peer agents
/// and serves the agent card at `/.well-known/agent-card.json`.
///
/// This function blocks until the server is stopped.
///
/// # Errors
///
/// Returns an error if the server fails to bind to the address or encounters a runtime error.
pub async fn start(handler: AgentMessageHandler, port: u16) -> Result<()> {
    let addr = format!("0.0.0.0:{port}");
    info!("🌐 A2A server starting on {addr}");

    A2aServer::new(handler).bind(&addr)?.run().await
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn handler_created_for_server() {
        let handler =
            AgentMessageHandler::new("test-gen".to_string(), "test-generator".to_string());
        let card =
            a2a_rs_server::MessageHandler::agent_card(&handler, "http://localhost:9090");
        assert_eq!(card.name, "test-gen");
    }

    #[test]
    fn default_port_format() {
        let addr = format!("0.0.0.0:{}", 9090_u16);
        assert_eq!(addr, "0.0.0.0:9090");
    }
}
