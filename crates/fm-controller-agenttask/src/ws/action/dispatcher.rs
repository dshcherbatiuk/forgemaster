//! Routes WebSocket commands to the appropriate action handler.

use std::mem::{Discriminant, discriminant};
use std::sync::Arc;

use dashmap::DashMap;
use tracing::info;

use super::{SubmitTaskAction, WsAction};
use crate::ws::command::WsCommand;
use crate::ws::connection_registry::ConnectionRegistry;
use crate::ws::task_creator::TaskCreator;

/// Routes WebSocket commands to registered action handlers.
pub struct ActionDispatcher {
    actions: DashMap<Discriminant<WsCommand>, Arc<dyn WsAction>>,
}

impl ActionDispatcher {
    /// Creates a dispatcher with all action handlers registered.
    pub fn new(task_creator: Arc<TaskCreator>, registry: Arc<ConnectionRegistry>) -> Self {
        let actions: DashMap<Discriminant<WsCommand>, Arc<dyn WsAction>> = DashMap::new();

        actions.insert(
            discriminant(&WsCommand::SubmitTask {
                description: String::new(),
            }),
            Arc::new(SubmitTaskAction::new(task_creator, registry)),
        );

        Self { actions }
    }

    /// Dispatches a raw WebSocket message to the appropriate action handler.
    pub async fn dispatch(&self, message: &str, client_id: &str) {
        let command: WsCommand = match serde_json::from_str(message) {
            Ok(cmd) => cmd,
            Err(err) => {
                info!("⚠️ Invalid command from {client_id}: {err}");
                return;
            }
        };

        if matches!(command, WsCommand::Connect) {
            info!("🤝 Client {client_id} sent connect");
            return;
        }

        let key = discriminant(&command);
        let action = match self.actions.get(&key) {
            Some(a) => Arc::clone(a.value()),
            None => {
                info!("⚠️ No handler for command from {client_id}");
                return;
            }
        };

        // Parse as raw Value for the action handler to extract fields
        let payload: serde_json::Value = match serde_json::from_str(message) {
            Ok(v) => v,
            Err(_) => return, // Already validated above
        };

        if let Err(err) = action.execute(&payload, client_id).await {
            info!("❌ Action failed for {client_id}: {err}");
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn discriminant_keys_match() {
        let key = discriminant(&WsCommand::SubmitTask {
            description: String::new(),
        });
        let command = WsCommand::SubmitTask {
            description: "Build API".to_string(),
        };
        assert_eq!(discriminant(&command), key);
    }

    #[test]
    fn different_variants_have_different_keys() {
        let connect_key = discriminant(&WsCommand::Connect);
        let submit_key = discriminant(&WsCommand::SubmitTask {
            description: String::new(),
        });
        assert_ne!(connect_key, submit_key);
    }
}
