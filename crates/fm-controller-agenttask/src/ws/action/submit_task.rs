//! Handles the SubmitTask command by creating an AgentTask CRD.

use std::sync::Arc;

use anyhow::{Result, bail};
use async_trait::async_trait;
use tracing::{info, warn};

use super::WsAction;
use crate::ws::active_task_store::ActiveTaskStore;
use crate::ws::connection_registry::ConnectionRegistry;
use crate::ws::event::WsEvent;
use crate::ws::task_creator::TaskCreator;

/// Creates an AgentTask CRD when the user submits a task.
pub struct SubmitTaskAction {
    /// Creates AgentTask CRDs in Kubernetes.
    task_creator: Arc<TaskCreator>,
    /// Registry for sending responses back to the client.
    registry: Arc<ConnectionRegistry>,
    /// Tracks active tasks to enforce the concurrency limit.
    active_tasks: Arc<ActiveTaskStore>,
}

impl SubmitTaskAction {
    /// Creates a new submit task action handler.
    pub fn new(
        task_creator: Arc<TaskCreator>,
        registry: Arc<ConnectionRegistry>,
        active_tasks: Arc<ActiveTaskStore>,
    ) -> Self {
        Self {
            task_creator,
            registry,
            active_tasks,
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

        if self.active_tasks.is_full() {
            let max = self.active_tasks.max_tasks();
            warn!(
                "⚠️ Task limit reached ({max}), rejecting submission from {client_id}"
            );

            let error_response = WsEvent::Data {
                data: serde_json::json!({
                    "error": {
                        "message": format!("Maximum of {max} concurrent tasks reached")
                    }
                }),
            };
            if let Ok(json) = serde_json::to_string(&error_response) {
                let _ = self.registry.send_to(client_id, &json);
            }

            bail!("maximum of {max} concurrent tasks reached");
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
        let registry = Arc::new(ConnectionRegistry::new());
        let active_tasks = Arc::new(ActiveTaskStore::new());
        assert_eq!(registry.connected_count(), 0);
        assert!(!active_tasks.is_full());
    }

    #[test]
    fn is_full_uses_configurable_limit() {
        let store = ActiveTaskStore::with_limit(1);
        assert!(!store.is_full());
        // Cannot insert without a real TaskStateChanged, but we can verify the limit
        assert_eq!(store.max_tasks(), 1);
    }
}
