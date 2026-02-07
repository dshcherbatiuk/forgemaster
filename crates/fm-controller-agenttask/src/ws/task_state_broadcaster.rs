//! Bridges controller state changes to WebSocket clients.

use std::sync::Arc;

use tokio::sync::broadcast;
use tracing::{info, warn};

use crate::task_event::TaskEvent;
use crate::task_state_changed::TaskStateChanged;

use super::active_task_store::ActiveTaskStore;
use super::connection_registry::ConnectionRegistry;
use super::event::WsEvent;
use super::schema_cache::SchemaCache;
use super::task_status_schema::build_multi_task_schema;

/// Listens for task state changes and pushes combined schema to connected WS clients.
pub struct TaskStateBroadcaster {
    receiver: broadcast::Receiver<TaskEvent>,
    schema_cache: Arc<SchemaCache>,
    registry: Arc<ConnectionRegistry>,
    active_tasks: Arc<ActiveTaskStore>,
}

impl TaskStateBroadcaster {
    /// Creates a new broadcaster.
    pub fn new(
        receiver: broadcast::Receiver<TaskEvent>,
        schema_cache: Arc<SchemaCache>,
        registry: Arc<ConnectionRegistry>,
        active_tasks: Arc<ActiveTaskStore>,
    ) -> Self {
        Self {
            receiver,
            schema_cache,
            registry,
            active_tasks,
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
        self.active_tasks.insert(event.clone());
        self.rebuild_and_broadcast();

        info!(
            "📤 Broadcast multi-task schema: {} → {:?} ({} active)",
            event.task_name,
            event.phase,
            self.active_tasks.count()
        );
    }

    fn handle_deletion(&self, task_name: &str) {
        self.active_tasks.remove(task_name);

        if self.active_tasks.count() == 0 {
            self.schema_cache.remove("schema");
            self.schema_cache.remove("dashboard");

            let ws_event = WsEvent::TaskDeleted {
                task_name: task_name.to_string(),
            };
            if let Ok(json) = serde_json::to_string(&ws_event) {
                self.registry.broadcast(&json);
            }
            info!("📤 Broadcast task deleted (last task): {}", task_name);
        } else {
            self.rebuild_and_broadcast();
            info!(
                "📤 Broadcast rebuilt schema after deleting {} ({} remaining)",
                task_name,
                self.active_tasks.count()
            );
        }
    }

    /// Rebuilds the combined schema from all active tasks and broadcasts it.
    fn rebuild_and_broadcast(&self) {
        let all_tasks = self.active_tasks.ordered_tasks();
        let (root, components, data) = build_multi_task_schema(&all_tasks);

        self.schema_cache.set(
            "schema",
            serde_json::json!({
                "root": root,
                "components": components,
                "data": data,
            }),
        );

        self.schema_cache.set("dashboard", data.clone());

        let ws_event = WsEvent::Schema {
            root,
            components,
            data,
        };

        if let Ok(json) = serde_json::to_string(&ws_event) {
            self.registry.broadcast(&json);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::agent_info::AgentInfoList;
    use crate::crd::AgentTaskPhase;
    use tokio::sync::mpsc;

    fn sample_event(name: &str) -> TaskStateChanged {
        TaskStateChanged {
            task_name: name.to_string(),
            namespace: "forgemaster-system".to_string(),
            description: format!("Task {name}"),
            created_at: Some(chrono::Utc::now()),
            phase: AgentTaskPhase::Running,
            iteration: 1,
            error: 0.6,
            tests_total: 5,
            tests_passed: 2,
            agents: AgentInfoList::new(),
        }
    }

    fn create_broadcaster_with_client() -> (TaskStateBroadcaster, mpsc::UnboundedReceiver<String>) {
        let (sender, _) = broadcast::channel::<TaskEvent>(16);
        let schema_cache = Arc::new(SchemaCache::new());
        let registry = Arc::new(ConnectionRegistry::new());
        let active_tasks = Arc::new(ActiveTaskStore::new());

        let (client_tx, client_rx) = mpsc::unbounded_channel();
        registry.register("test-client", client_tx);

        let broadcaster =
            TaskStateBroadcaster::new(sender.subscribe(), schema_cache, registry, active_tasks);

        (broadcaster, client_rx)
    }

    #[test]
    fn handle_event_caches_schema() {
        let (broadcaster, _rx) = create_broadcaster_with_client();
        broadcaster.handle_event(&sample_event("task-abc"));

        let cached = broadcaster.schema_cache.get("schema").unwrap();
        assert_eq!(cached["root"], "hero-section");
        assert!(cached["components"].is_array());
        assert_eq!(cached["data"]["tasks"]["items"][0]["name"], "task-abc");
    }

    #[test]
    fn handle_event_updates_dashboard() {
        let (broadcaster, _rx) = create_broadcaster_with_client();
        broadcaster.handle_event(&sample_event("task-abc"));

        let cached = broadcaster.schema_cache.get("dashboard").unwrap();
        assert_eq!(cached["tasks"]["count"], 1);
        assert_eq!(cached["tasks"]["items"][0]["name"], "task-abc");
    }

    #[test]
    fn handle_event_broadcasts_schema() {
        let (broadcaster, mut rx) = create_broadcaster_with_client();
        broadcaster.handle_event(&sample_event("task-abc"));

        let msg = rx.try_recv().unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&msg).unwrap();
        assert_eq!(parsed["type"], "schema");
        assert_eq!(parsed["root"], "hero-section");
        assert_eq!(parsed["data"]["tasks"]["items"][0]["phase"], "Running");
    }

    #[test]
    fn handle_two_events_caches_both() {
        let (broadcaster, _rx) = create_broadcaster_with_client();
        broadcaster.handle_event(&sample_event("task-aaa"));
        broadcaster.handle_event(&sample_event("task-bbb"));

        let cached = broadcaster.schema_cache.get("dashboard").unwrap();
        assert_eq!(cached["tasks"]["count"], 2);
    }

    #[test]
    fn handle_deletion_last_task_clears_caches() {
        let (broadcaster, _rx) = create_broadcaster_with_client();
        broadcaster.handle_event(&sample_event("task-abc"));
        assert!(broadcaster.schema_cache.get("schema").is_some());

        broadcaster.handle_deletion("task-abc");
        assert!(broadcaster.schema_cache.get("schema").is_none());
        assert!(broadcaster.schema_cache.get("dashboard").is_none());
    }

    #[test]
    fn handle_deletion_last_task_broadcasts_task_deleted() {
        let (broadcaster, mut rx) = create_broadcaster_with_client();
        broadcaster.handle_event(&sample_event("task-abc"));
        let _ = rx.try_recv(); // consume schema event

        broadcaster.handle_deletion("task-abc");
        let msg = rx.try_recv().unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&msg).unwrap();
        assert_eq!(parsed["type"], "task_deleted");
        assert_eq!(parsed["task_name"], "task-abc");
    }

    #[test]
    fn handle_deletion_with_remaining_rebuilds_schema() {
        let (broadcaster, mut rx) = create_broadcaster_with_client();
        broadcaster.handle_event(&sample_event("task-aaa"));
        broadcaster.handle_event(&sample_event("task-bbb"));
        let _ = rx.try_recv(); // consume first
        let _ = rx.try_recv(); // consume second

        broadcaster.handle_deletion("task-aaa");
        let msg = rx.try_recv().unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&msg).unwrap();
        assert_eq!(parsed["type"], "schema");
        assert_eq!(parsed["data"]["tasks"]["count"], 1);
        assert_eq!(parsed["data"]["tasks"]["items"][0]["name"], "task-bbb");
    }

    #[test]
    fn handle_deletion_with_remaining_keeps_caches() {
        let (broadcaster, _rx) = create_broadcaster_with_client();
        broadcaster.handle_event(&sample_event("task-aaa"));
        broadcaster.handle_event(&sample_event("task-bbb"));

        broadcaster.handle_deletion("task-aaa");
        assert!(broadcaster.schema_cache.get("schema").is_some());
        assert!(broadcaster.schema_cache.get("dashboard").is_some());
    }
}
