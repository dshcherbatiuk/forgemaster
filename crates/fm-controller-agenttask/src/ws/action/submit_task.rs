//! Handles the SubmitTask command by creating an AgentTask CRD.

use std::sync::Arc;

use anyhow::{bail, Result};
use async_trait::async_trait;
use tracing::info;

use super::WsAction;
use crate::ws::connection_registry::ConnectionRegistry;
use crate::ws::event::WsEvent;
use crate::ws::task_creator::TaskCreator;

/// Creates an AgentTask CRD when the user submits a task.
pub struct SubmitTaskAction {
    /// Creates AgentTask CRDs in Kubernetes.
    task_creator: Arc<TaskCreator>,
    /// Registry for sending responses back to the client.
    registry: Arc<ConnectionRegistry>,
}

impl SubmitTaskAction {
    /// Creates a new submit task action handler.
    pub fn new(task_creator: Arc<TaskCreator>, registry: Arc<ConnectionRegistry>) -> Self {
        Self {
            task_creator,
            registry,
        }
    }
}

#[async_trait]
impl WsAction for SubmitTaskAction {
    async fn execute(&self, payload: &serde_json::Value, client_id: &str) -> Result<()> {
        let description = payload
            .get("description")
            .and_then(|v| v.as_str())
            .unwrap_or("");

        if description.trim().is_empty() {
            bail!("empty task description");
        }

        let task_name = self
            .task_creator
            .create_from_description(description)
            .await?;

        let response = WsEvent::Data {
            data: serde_json::json!({
                "task": {
                    "name": task_name,
                    "description": description,
                    "phase": "Pending"
                }
            }),
        };

        if let Ok(json) = serde_json::to_string(&response) {
            let _ = self.registry.send_to(client_id, &json);
        }

        info!("📝 Task {task_name} created for client {client_id}");
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn submit_task_action_fields() {
        // Verify struct construction compiles with the right types.
        // TaskCreator requires a K8s client, so we only test ConnectionRegistry here.
        let registry = Arc::new(ConnectionRegistry::new());
        assert_eq!(registry.connected_count(), 0);
    }
}
