//! Bridges controller state changes to WebSocket clients.

use std::sync::Arc;

use tokio::sync::broadcast;
use tracing::{info, warn};

use crate::task_event::TaskEvent;
use crate::task_state_changed::TaskStateChanged;

use super::connection_registry::ConnectionRegistry;
use super::event::WsEvent;
use super::schema_cache::SchemaCache;
use super::task_status_schema::build_task_status_schema;

/// Listens for task state changes and pushes them to connected WS clients.
pub struct TaskStateBroadcaster {
    receiver: broadcast::Receiver<TaskEvent>,
    schema_cache: Arc<SchemaCache>,
    registry: Arc<ConnectionRegistry>,
}

impl TaskStateBroadcaster {
    /// Creates a new broadcaster.
    pub fn new(
        receiver: broadcast::Receiver<TaskEvent>,
        schema_cache: Arc<SchemaCache>,
        registry: Arc<ConnectionRegistry>,
    ) -> Self {
        Self {
            receiver,
            schema_cache,
            registry,
        }
    }

    /// Runs the listener loop. Blocks until the channel closes.
    pub async fn run(mut self) {
        info!("📡 TaskStateBroadcaster started");
        loop {
            match self.receiver.recv().await {
                Ok(TaskEvent::StateChanged(event)) => self.handle_event(&event),
                Ok(TaskEvent::Deleted { task_name }) => self.handle_deletion(&task_name),
                Err(broadcast::error::RecvError::Lagged(count)) => {
                    warn!("⚠️ Task state broadcaster lagged by {} events", count);
                }
                Err(broadcast::error::RecvError::Closed) => {
                    info!("📡 Task state channel closed, stopping broadcaster");
                    break;
                }
            }
        }
    }

    fn handle_event(&self, event: &TaskStateChanged) {
        let (root, components, data) = build_task_status_schema(event);

        // Cache schema for late joiners
        self.schema_cache.set(
            "schema",
            serde_json::json!({
                "root": root,
                "components": components,
                "data": data,
            }),
        );

        // Also update dashboard data cache (for data-only late joiner path)
        let task_value = &data["task"];
        if let Some(mut existing) = self.schema_cache.get("dashboard") {
            if let Some(obj) = existing.as_object_mut() {
                obj.insert("task".to_string(), task_value.clone());
            }
            self.schema_cache.set("dashboard", existing);
        }

        let ws_event = WsEvent::Schema {
            root,
            components,
            data,
        };

        if let Ok(json) = serde_json::to_string(&ws_event) {
            self.registry.broadcast(&json);
            info!(
                "📤 Broadcast task schema: {} → {:?}",
                event.task_name, event.phase
            );
        }
    }

    fn handle_deletion(&self, task_name: &str) {
        self.schema_cache.remove("schema");
        self.schema_cache.remove("dashboard");

        let ws_event = WsEvent::TaskDeleted {
            task_name: task_name.to_string(),
        };

        if let Ok(json) = serde_json::to_string(&ws_event) {
            self.registry.broadcast(&json);
            info!("📤 Broadcast task deleted: {}", task_name);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::crd::AgentTaskPhase;
    use serde_json::json;
    use tokio::sync::mpsc;

    fn sample_event() -> TaskStateChanged {
        TaskStateChanged {
            task_name: "task-abc12345".to_string(),
            namespace: "forgemaster-system".to_string(),
            description: "Build a REST API".to_string(),
            created_at: Some(chrono::Utc::now()),
            phase: AgentTaskPhase::Running,
            iteration: 1,
            error: 0.6,
            tests_total: 5,
            tests_passed: 2,
        }
    }

    fn create_broadcaster_with_client() -> (TaskStateBroadcaster, mpsc::UnboundedReceiver<String>) {
        let (sender, _) = broadcast::channel::<TaskEvent>(16);
        let schema_cache = Arc::new(SchemaCache::new());
        let registry = Arc::new(ConnectionRegistry::new());

        let (client_tx, client_rx) = mpsc::unbounded_channel();
        registry.register("test-client", client_tx);

        let broadcaster = TaskStateBroadcaster::new(sender.subscribe(), schema_cache, registry);

        (broadcaster, client_rx)
    }

    #[test]
    fn handle_event_caches_schema() {
        let (broadcaster, _rx) = create_broadcaster_with_client();
        broadcaster.handle_event(&sample_event());

        let cached = broadcaster.schema_cache.get("schema").unwrap();
        assert_eq!(cached["root"], "task-status-card");
        assert!(cached["components"].is_array());
        assert_eq!(cached["data"]["task"]["name"], "task-abc12345");
    }

    #[test]
    fn handle_event_updates_dashboard_cache() {
        let (broadcaster, _rx) = create_broadcaster_with_client();
        broadcaster.schema_cache.set(
            "dashboard",
            json!({
                "hero": { "tagline": "AI-Powered" },
                "task": { "description": "" }
            }),
        );

        broadcaster.handle_event(&sample_event());

        let cached = broadcaster.schema_cache.get("dashboard").unwrap();
        assert_eq!(cached["hero"]["tagline"], "AI-Powered");
        assert_eq!(cached["task"]["name"], "task-abc12345");
    }

    #[test]
    fn handle_event_broadcasts_schema_event() {
        let (broadcaster, mut rx) = create_broadcaster_with_client();
        broadcaster.handle_event(&sample_event());

        let msg = rx.try_recv().unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&msg).unwrap();
        assert_eq!(parsed["type"], "schema");
        assert_eq!(parsed["root"], "task-status-card");
        assert!(parsed["components"].is_array());
        assert_eq!(parsed["data"]["task"]["phase"], "Running");
    }

    #[test]
    fn handle_event_broadcasts_task_data() {
        let (broadcaster, mut rx) = create_broadcaster_with_client();
        broadcaster.handle_event(&sample_event());

        let msg = rx.try_recv().unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&msg).unwrap();
        assert_eq!(parsed["data"]["task"]["name"], "task-abc12345");
        assert_eq!(parsed["data"]["task"]["iteration"], 1);
        assert_eq!(parsed["data"]["task"]["error"], 0.6);
    }

    #[test]
    fn handle_deletion_clears_schema_cache() {
        let (broadcaster, _rx) = create_broadcaster_with_client();
        broadcaster.handle_event(&sample_event());
        assert!(broadcaster.schema_cache.get("schema").is_some());

        broadcaster.handle_deletion("task-abc12345");
        assert!(broadcaster.schema_cache.get("schema").is_none());
    }

    #[test]
    fn handle_deletion_clears_dashboard_cache() {
        let (broadcaster, _rx) = create_broadcaster_with_client();
        broadcaster
            .schema_cache
            .set("dashboard", json!({"task": {"name": "task-abc12345"}}));

        broadcaster.handle_deletion("task-abc12345");
        assert!(broadcaster.schema_cache.get("dashboard").is_none());
    }

    #[test]
    fn handle_deletion_broadcasts_task_deleted_event() {
        let (broadcaster, mut rx) = create_broadcaster_with_client();
        broadcaster.handle_deletion("task-abc12345");

        let msg = rx.try_recv().unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&msg).unwrap();
        assert_eq!(parsed["type"], "task_deleted");
        assert_eq!(parsed["task_name"], "task-abc12345");
    }
}
