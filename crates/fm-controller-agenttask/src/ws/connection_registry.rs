//! Registry of active WebSocket connections.

use dashmap::DashMap;
use tokio::sync::mpsc;
use tracing::warn;

/// Sender half for pushing serialized messages to a single client.
pub type ClientSender = mpsc::UnboundedSender<String>;

/// Tracks all active WebSocket client connections.
pub struct ConnectionRegistry {
    connections: DashMap<String, ClientSender>,
}

impl ConnectionRegistry {
    /// Creates an empty registry.
    pub fn new() -> Self {
        Self {
            connections: DashMap::new(),
        }
    }

    /// Register a new client connection.
    pub fn register(&self, client_id: &str, sender: ClientSender) {
        self.connections.insert(client_id.to_string(), sender);
    }

    /// Remove a client connection.
    pub fn unregister(&self, client_id: &str) {
        self.connections.remove(client_id);
    }

    /// Send a message to a specific client. Returns error if client not found.
    pub fn send_to(&self, client_id: &str, message: &str) -> anyhow::Result<()> {
        let sender = self
            .connections
            .get(client_id)
            .ok_or_else(|| anyhow::anyhow!("Client not found: {client_id}"))?;
        sender
            .send(message.to_string())
            .map_err(|e| anyhow::anyhow!("Failed to send to {client_id}: {e}"))
    }

    /// Broadcast a message to all connected clients.
    pub fn broadcast(&self, message: &str) {
        for entry in self.connections.iter() {
            if let Err(e) = entry.value().send(message.to_string()) {
                warn!("⚠️ Failed to send to client {}: {e}", entry.key());
            }
        }
    }

    /// Number of connected clients.
    pub fn connected_count(&self) -> usize {
        self.connections.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn register_increases_count() {
        let registry = ConnectionRegistry::new();
        let (sender, _receiver) = mpsc::unbounded_channel();
        registry.register("client-1", sender);
        assert_eq!(registry.connected_count(), 1);
    }

    #[test]
    fn unregister_decreases_count() {
        let registry = ConnectionRegistry::new();
        let (sender, _receiver) = mpsc::unbounded_channel();
        registry.register("client-1", sender);
        registry.unregister("client-1");
        assert_eq!(registry.connected_count(), 0);
    }

    #[test]
    fn send_to_unknown_client_fails() {
        let registry = ConnectionRegistry::new();
        let result = registry.send_to("nonexistent", "hello");
        assert!(result.is_err());
    }

    #[test]
    fn send_to_known_client_delivers_message() {
        let registry = ConnectionRegistry::new();
        let (sender, mut receiver) = mpsc::unbounded_channel();
        registry.register("client-1", sender);
        registry.send_to("client-1", "hello").unwrap();
        let received = receiver.try_recv().unwrap();
        assert_eq!(received, "hello");
    }

    #[test]
    fn broadcast_sends_to_all_clients() {
        let registry = ConnectionRegistry::new();
        let (sender_a, mut receiver_a) = mpsc::unbounded_channel();
        let (sender_b, mut receiver_b) = mpsc::unbounded_channel();
        registry.register("a", sender_a);
        registry.register("b", sender_b);

        registry.broadcast("ping");

        assert_eq!(receiver_a.try_recv().unwrap(), "ping");
        assert_eq!(receiver_b.try_recv().unwrap(), "ping");
    }
}
