//! Builds A2UI schema components for task status display.

use serde_json::{json, Value};
use smallvec::{smallvec, SmallVec};

use crate::task_state_changed::TaskStateChanged;

/// Root component ID for the task status schema.
pub const TASK_STATUS_ROOT: &str = "task-status-card";

/// Builds A2UI schema for displaying task status.
///
/// Returns (root, components, data) matching the A2UI v0.8 format.
pub fn build_task_status_schema(
    event: &TaskStateChanged,
) -> (String, SmallVec<[Value; 16]>, Value) {
    let components = smallvec![
        // Override hero-section to include task-status-card in layout
        json!({
            "id": "hero-section",
            "component": { "Column": {
                "children": { "explicitList": [
                    "hero-tagline", "hero-subtitle", "task-card", "task-status-card"
                ] }
            } }
        }),
        json!({
            "id": "task-status-card",
            "component": { "Card": { "child": "task-status-col" } }
        }),
        json!({
            "id": "task-status-col",
            "component": { "Column": {
                "children": { "explicitList": [
                    "task-status-title",
                    "task-status-name-row",
                    "task-status-desc-row",
                    "task-status-phase-row",
                    "task-status-iteration-row",
                    "task-status-error-row",
                    "task-status-tests-row"
                ] }
            } }
        }),
        json!({
            "id": "task-status-title",
            "component": { "Text": {
                "text": { "literalString": "Task Status" },
                "usageHint": "h2"
            } }
        }),
        // Name row
        json!({
            "id": "task-status-name-row",
            "component": { "Row": {
                "distribution": "spaceBetween",
                "children": { "explicitList": ["task-status-name-label", "task-status-name-value"] }
            } }
        }),
        json!({
            "id": "task-status-name-label",
            "component": { "Text": {
                "text": { "literalString": "Name:" },
                "usageHint": "body"
            } }
        }),
        json!({
            "id": "task-status-name-value",
            "component": { "Text": {
                "text": { "path": "/task/name" },
                "usageHint": "body"
            } }
        }),
        // Description row
        json!({
            "id": "task-status-desc-row",
            "component": { "Row": {
                "distribution": "spaceBetween",
                "children": { "explicitList": ["task-status-desc-label", "task-status-desc-value"] }
            } }
        }),
        json!({
            "id": "task-status-desc-label",
            "component": { "Text": {
                "text": { "literalString": "Description:" },
                "usageHint": "body"
            } }
        }),
        json!({
            "id": "task-status-desc-value",
            "component": { "Text": {
                "text": { "path": "/task/description" },
                "usageHint": "body"
            } }
        }),
        // Phase row
        json!({
            "id": "task-status-phase-row",
            "component": { "Row": {
                "distribution": "spaceBetween",
                "children": { "explicitList": ["task-status-phase-label", "task-status-phase-value"] }
            } }
        }),
        json!({
            "id": "task-status-phase-label",
            "component": { "Text": {
                "text": { "literalString": "Phase:" },
                "usageHint": "body"
            } }
        }),
        json!({
            "id": "task-status-phase-value",
            "component": { "Text": {
                "text": { "path": "/task/phase" },
                "usageHint": "body"
            } }
        }),
        // Iteration row
        json!({
            "id": "task-status-iteration-row",
            "component": { "Row": {
                "distribution": "spaceBetween",
                "children": { "explicitList": ["task-status-iteration-label", "task-status-iteration-value"] }
            } }
        }),
        json!({
            "id": "task-status-iteration-label",
            "component": { "Text": {
                "text": { "literalString": "Iteration:" },
                "usageHint": "body"
            } }
        }),
        json!({
            "id": "task-status-iteration-value",
            "component": { "Text": {
                "text": { "path": "/task/iteration" },
                "usageHint": "body"
            } }
        }),
        // Error row
        json!({
            "id": "task-status-error-row",
            "component": { "Row": {
                "distribution": "spaceBetween",
                "children": { "explicitList": ["task-status-error-label", "task-status-error-value"] }
            } }
        }),
        json!({
            "id": "task-status-error-label",
            "component": { "Text": {
                "text": { "literalString": "Error Signal:" },
                "usageHint": "body"
            } }
        }),
        json!({
            "id": "task-status-error-value",
            "component": { "Text": {
                "text": { "path": "/task/error" },
                "usageHint": "body"
            } }
        }),
        // Tests row
        json!({
            "id": "task-status-tests-row",
            "component": { "Row": {
                "distribution": "spaceBetween",
                "children": { "explicitList": ["task-status-tests-label", "task-status-tests-value"] }
            } }
        }),
        json!({
            "id": "task-status-tests-label",
            "component": { "Text": {
                "text": { "literalString": "Tests:" },
                "usageHint": "body"
            } }
        }),
        json!({
            "id": "task-status-tests-value",
            "component": { "Text": {
                "text": { "path": "/task/testsDisplay" },
                "usageHint": "body"
            } }
        }),
    ];

    let data = json!({
        "task": {
            "name": event.task_name,
            "description": event.description,
            "phase": event.phase,
            "iteration": event.iteration,
            "error": event.error,
            "testsTotal": event.tests_total,
            "testsPassed": event.tests_passed,
            "testsDisplay": format!("{}/{}", event.tests_passed, event.tests_total),
        }
    });

    (TASK_STATUS_ROOT.to_string(), components, data)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::crd::AgentTaskPhase;

    fn sample_event() -> TaskStateChanged {
        TaskStateChanged {
            task_name: "task-abc12345".to_string(),
            namespace: "forgemaster-system".to_string(),
            description: "Build a REST API".to_string(),
            phase: AgentTaskPhase::Running,
            iteration: 2,
            error: 0.4,
            tests_total: 5,
            tests_passed: 3,
        }
    }

    #[test]
    fn root_is_task_status_card() {
        let (root, _, _) = build_task_status_schema(&sample_event());
        assert_eq!(root, "task-status-card");
    }

    #[test]
    fn components_contain_expected_ids() {
        let (_, components, _) = build_task_status_schema(&sample_event());
        let ids: Vec<&str> = components
            .iter()
            .filter_map(|c| c.get("id").and_then(|v| v.as_str()))
            .collect();

        assert!(ids.contains(&"hero-section"));
        assert!(ids.contains(&"task-status-card"));
        assert!(ids.contains(&"task-status-title"));
        assert!(ids.contains(&"task-status-name-value"));
        assert!(ids.contains(&"task-status-desc-value"));
        assert!(ids.contains(&"task-status-phase-value"));
        assert!(ids.contains(&"task-status-iteration-value"));
        assert!(ids.contains(&"task-status-error-value"));
        assert!(ids.contains(&"task-status-tests-value"));
    }

    #[test]
    fn hero_section_override_includes_task_status_card() {
        let (_, components, _) = build_task_status_schema(&sample_event());
        let hero_section = components
            .iter()
            .find(|c| c["id"] == "hero-section")
            .unwrap();
        let children = &hero_section["component"]["Column"]["children"]["explicitList"];
        assert!(children.as_array().unwrap().contains(&json!("task-status-card")));
    }

    #[test]
    fn data_contains_task_fields() {
        let (_, _, data) = build_task_status_schema(&sample_event());
        assert_eq!(data["task"]["name"], "task-abc12345");
        assert_eq!(data["task"]["description"], "Build a REST API");
        assert_eq!(data["task"]["phase"], "Running");
        assert_eq!(data["task"]["iteration"], 2);
        assert_eq!(data["task"]["error"], 0.4);
        assert_eq!(data["task"]["testsTotal"], 5);
        assert_eq!(data["task"]["testsPassed"], 3);
        assert_eq!(data["task"]["testsDisplay"], "3/5");
    }

    #[test]
    fn components_use_data_bindings() {
        let (_, components, _) = build_task_status_schema(&sample_event());
        let name_value = components
            .iter()
            .find(|c| c["id"] == "task-status-name-value")
            .unwrap();
        assert_eq!(name_value["component"]["Text"]["text"]["path"], "/task/name");
    }

    #[test]
    fn components_count() {
        let (_, components, _) = build_task_status_schema(&sample_event());
        assert_eq!(components.len(), 22);
    }
}
