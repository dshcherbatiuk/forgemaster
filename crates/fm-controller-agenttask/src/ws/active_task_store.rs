//! In-memory store tracking active tasks with a configurable concurrency limit.

use dashmap::DashMap;
use smallvec::SmallVec;

use crate::task_state_changed::TaskStateChanged;

/// Default maximum number of concurrent tasks.
pub const DEFAULT_MAX_ACTIVE_TASKS: usize = 2;

/// Tracks active task states keyed by task name.
///
/// Uses `DashMap` for lock-free concurrent reads/writes.
/// The concurrency limit is configurable via [`ActiveTaskStore::with_limit`].
pub struct ActiveTaskStore {
    tasks: DashMap<String, TaskStateChanged>,
    max_tasks: usize,
}

impl ActiveTaskStore {
    /// Creates an empty store with the default limit ([`DEFAULT_MAX_ACTIVE_TASKS`]).
    pub fn new() -> Self {
        Self::with_limit(DEFAULT_MAX_ACTIVE_TASKS)
    }

    /// Creates an empty store with a custom concurrency limit.
    pub fn with_limit(max_tasks: usize) -> Self {
        Self {
            tasks: DashMap::new(),
            max_tasks,
        }
    }

    /// Inserts or updates a task by name.
    pub fn insert(&self, event: TaskStateChanged) {
        self.tasks.insert(event.task_name.clone(), event);
    }

    /// Removes a task by name. Returns `true` if it existed.
    pub fn remove(&self, task_name: &str) -> bool {
        self.tasks.remove(task_name).is_some()
    }

    /// Returns `true` when the store has reached its concurrency limit.
    pub fn is_full(&self) -> bool {
        self.tasks.len() >= self.max_tasks
    }

    /// The configured concurrency limit.
    pub fn max_tasks(&self) -> usize {
        self.max_tasks
    }

    /// Number of active tasks.
    pub fn count(&self) -> usize {
        self.tasks.len()
    }

    /// Returns all active tasks sorted by name for deterministic ordering.
    pub fn ordered_tasks(&self) -> SmallVec<[TaskStateChanged; 2]> {
        let mut tasks: SmallVec<[TaskStateChanged; 2]> = self
            .tasks
            .iter()
            .map(|entry| entry.value().clone())
            .collect();
        tasks.sort_by(|a, b| a.task_name.cmp(&b.task_name));
        tasks
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::agent_info::AgentInfoList;
    use crate::crd::AgentTaskPhase;

    fn sample_event(name: &str) -> TaskStateChanged {
        TaskStateChanged {
            task_name: name.to_string(),
            namespace: "forgemaster-system".to_string(),
            description: format!("Task {name}"),
            created_at: None,
            phase: AgentTaskPhase::Running,
            iteration: 0,
            error: 1.0,
            tests_total: 0,
            tests_passed: 0,
            agents: AgentInfoList::new(),
        }
    }

    #[test]
    fn empty_store_has_zero_count() {
        let store = ActiveTaskStore::new();
        assert_eq!(store.count(), 0);
    }

    #[test]
    fn default_limit_is_two() {
        let store = ActiveTaskStore::new();
        assert_eq!(store.max_tasks(), DEFAULT_MAX_ACTIVE_TASKS);
    }

    #[test]
    fn custom_limit() {
        let store = ActiveTaskStore::with_limit(5);
        assert_eq!(store.max_tasks(), 5);
        for i in 0..5 {
            store.insert(sample_event(&format!("task-{i}")));
        }
        assert!(store.is_full());
    }

    #[test]
    fn insert_one_task_count_is_one() {
        let store = ActiveTaskStore::new();
        store.insert(sample_event("task-aaa"));
        assert_eq!(store.count(), 1);
    }

    #[test]
    fn insert_two_tasks_count_is_two() {
        let store = ActiveTaskStore::new();
        store.insert(sample_event("task-aaa"));
        store.insert(sample_event("task-bbb"));
        assert_eq!(store.count(), 2);
    }

    #[test]
    fn is_full_after_two_inserts() {
        let store = ActiveTaskStore::new();
        store.insert(sample_event("task-aaa"));
        store.insert(sample_event("task-bbb"));
        assert!(store.is_full());
    }

    #[test]
    fn is_not_full_after_one_insert() {
        let store = ActiveTaskStore::new();
        store.insert(sample_event("task-aaa"));
        assert!(!store.is_full());
    }

    #[test]
    fn remove_existing_returns_true() {
        let store = ActiveTaskStore::new();
        store.insert(sample_event("task-aaa"));
        assert!(store.remove("task-aaa"));
    }

    #[test]
    fn remove_nonexistent_returns_false() {
        let store = ActiveTaskStore::new();
        assert!(!store.remove("task-missing"));
    }

    #[test]
    fn remove_decreases_count() {
        let store = ActiveTaskStore::new();
        store.insert(sample_event("task-aaa"));
        store.insert(sample_event("task-bbb"));
        store.remove("task-aaa");
        assert_eq!(store.count(), 1);
    }

    #[test]
    fn ordered_tasks_sorted_by_name() {
        let store = ActiveTaskStore::new();
        store.insert(sample_event("task-zzz"));
        store.insert(sample_event("task-aaa"));
        let tasks = store.ordered_tasks();
        assert_eq!(tasks[0].task_name, "task-aaa");
        assert_eq!(tasks[1].task_name, "task-zzz");
    }

    #[test]
    fn insert_overwrites_same_task_name() {
        let store = ActiveTaskStore::new();
        let mut event = sample_event("task-aaa");
        event.iteration = 1;
        store.insert(event);

        let mut updated = sample_event("task-aaa");
        updated.iteration = 5;
        store.insert(updated);

        assert_eq!(store.count(), 1);
        let tasks = store.ordered_tasks();
        assert_eq!(tasks[0].iteration, 5);
    }

    #[test]
    fn ordered_tasks_empty_when_no_tasks() {
        let store = ActiveTaskStore::new();
        let tasks = store.ordered_tasks();
        assert!(tasks.is_empty());
    }

    #[test]
    fn is_not_full_after_remove_from_full() {
        let store = ActiveTaskStore::new();
        store.insert(sample_event("task-aaa"));
        store.insert(sample_event("task-bbb"));
        assert!(store.is_full());
        store.remove("task-aaa");
        assert!(!store.is_full());
    }
}
