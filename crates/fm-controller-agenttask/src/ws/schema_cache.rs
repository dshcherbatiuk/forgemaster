//! Cache for the latest dashboard data state.
//!
//! When agents update task state (progress, test results, TCP signal, etc.),
//! the new state is stored here. This serves two purposes:
//!
//! 1. **Late joiners** — when a new UI client connects, it immediately receives
//!    the latest state from the cache instead of waiting for the next update.
//! 2. **Single source of truth** — the controller owns the current state and
//!    broadcasts changes to all connected clients from one place.
//!
//! Each entry is keyed by a task identifier (or "dashboard" for the global view).
//! When agents push updated data, the controller stores it here and broadcasts
//! the change to all connected UI clients.

use dashmap::DashMap;
use serde_json::Value;

/// Stores the latest data state per key.
pub struct SchemaCache {
    entries: DashMap<String, Value>,
}

impl SchemaCache {
    /// Creates an empty cache.
    pub fn new() -> Self {
        Self {
            entries: DashMap::new(),
        }
    }

    /// Get a clone of the cached value.
    pub fn get(&self, key: &str) -> Option<Value> {
        self.entries.get(key).map(|entry| entry.value().clone())
    }

    /// Store or replace a value.
    pub fn set(&self, key: &str, value: Value) {
        self.entries.insert(key.to_string(), value);
    }

    /// Remove a cached entry.
    pub fn remove(&self, key: &str) {
        self.entries.remove(key);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn get_returns_none_for_missing_key() {
        let cache = SchemaCache::new();
        assert!(cache.get("missing").is_none());
    }

    #[test]
    fn set_and_get_roundtrip() {
        let cache = SchemaCache::new();
        let data = json!({"status": "Ready"});
        cache.set("dashboard", data.clone());
        assert_eq!(cache.get("dashboard").unwrap(), data);
    }

    #[test]
    fn set_overwrites_previous_value() {
        let cache = SchemaCache::new();
        cache.set("dashboard", json!({"status": "Ready"}));
        cache.set("dashboard", json!({"status": "Running"}));
        assert_eq!(cache.get("dashboard").unwrap()["status"], "Running");
    }

    #[test]
    fn remove_invalidates_entry() {
        let cache = SchemaCache::new();
        cache.set("dashboard", json!({"status": "Ready"}));
        cache.remove("dashboard");
        assert!(cache.get("dashboard").is_none());
    }
}
