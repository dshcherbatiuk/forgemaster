//! Builds A2UI schema components for task status display.

use chrono::Utc;
use serde_json::{Value, json};
use smallvec::SmallVec;

use crate::agent_info::AgentInfoList;
use crate::task_state_changed::TaskStateChanged;

/// Formats a duration as a human-readable age string (e.g. "2m 30s", "1h 5m").
fn format_age(seconds: i64) -> String {
    if seconds < 0 {
        return "0s".to_string();
    }
    let hours = seconds / 3600;
    let minutes = (seconds % 3600) / 60;
    let secs = seconds % 60;

    if hours > 0 {
        format!("{hours}h {minutes}m")
    } else if minutes > 0 {
        format!("{minutes}m {secs}s")
    } else {
        format!("{secs}s")
    }
}

/// Root component ID for the task status schema.
pub const TASK_STATUS_ROOT: &str = "task-status-card";

/// Builds A2UI schema for displaying task status.
///
/// Returns (root, components, data) matching the A2UI v0.8 format.
pub fn build_task_status_schema(
    event: &TaskStateChanged,
) -> (String, SmallVec<[Value; 48]>, Value) {
    let mut components: SmallVec<[Value; 48]> = SmallVec::new();

    // Override hero-section to include task-status-card in layout
    components.push(json!({
        "id": "hero-section",
        "component": { "Column": {
            "children": { "explicitList": [
                "hero-tagline", "hero-subtitle", "task-card", "task-status-card"
            ] }
        } }
    }));

    components.push(json!({
        "id": "task-status-card",
        "component": { "Card": { "child": "task-status-col" } }
    }));

    // Build the column children list
    let col_children = vec![
        json!("task-status-title"),
        json!("task-status-name-row"),
        json!("task-status-desc-row"),
        json!("task-status-age-row"),
        json!("task-status-phase-row"),
        json!("task-status-iteration-row"),
        json!("task-status-error-row"),
        json!("task-status-tests-row"),
        json!("task-status-agents-section"),
    ];

    components.push(json!({
        "id": "task-status-title",
        "component": { "Text": {
            "text": { "literalString": "Task Status" },
            "usageHint": "h2"
        } }
    }));

    // Name row
    push_label_value_row(&mut components, "name", "Name:");
    // Description row
    push_label_value_row(&mut components, "desc", "Description:");
    // Age row
    push_label_value_row(&mut components, "age", "Age:");
    // Phase row
    push_label_value_row(&mut components, "phase", "Phase:");
    // Iteration row
    push_label_value_row(&mut components, "iteration", "Iteration:");
    // Error row
    push_label_value_row(&mut components, "error", "Error Signal:");
    // Tests row
    push_label_value_row(&mut components, "tests", "Tests:");

    // Agents section
    build_agents_section(&mut components, &event.agents);

    // Now set the column component with the final children list
    components.insert(
        2,
        json!({
            "id": "task-status-col",
            "component": { "Column": {
                "children": { "explicitList": col_children }
            } }
        }),
    );

    let age = event.created_at.map_or_else(
        || "unknown".to_string(),
        |t| format_age((Utc::now() - t).num_seconds()),
    );

    let agents_data: Vec<Value> = event
        .agents
        .iter()
        .map(|a| {
            json!({
                "name": a.name,
                "type": a.agent_type,
                "phase": a.phase,
            })
        })
        .collect();

    let data = json!({
        "task": {
            "name": event.task_name,
            "description": event.description,
            "age": age,
            "phase": event.phase,
            "iteration": event.iteration,
            "error": event.error,
            "testsTotal": event.tests_total,
            "testsPassed": event.tests_passed,
            "testsDisplay": format!("{}/{}", event.tests_passed, event.tests_total),
            "agents": agents_data,
        }
    });

    (TASK_STATUS_ROOT.to_string(), components, data)
}

/// Pushes a label-value row with its two text components.
fn push_label_value_row(components: &mut SmallVec<[Value; 48]>, field: &str, label: &str) {
    let row_id = format!("task-status-{field}-row");
    let label_id = format!("task-status-{field}-label");
    let value_id = format!("task-status-{field}-value");
    let data_path = match field {
        "tests" => "/task/testsDisplay".to_string(),
        other => format!("/task/{other}"),
    };

    components.push(json!({
        "id": row_id,
        "component": { "Row": {
            "distribution": "spaceBetween",
            "children": { "explicitList": [label_id, value_id] }
        } }
    }));
    components.push(json!({
        "id": label_id,
        "component": { "Text": {
            "text": { "literalString": label },
            "usageHint": "body"
        } }
    }));
    components.push(json!({
        "id": value_id,
        "component": { "Text": {
            "text": { "path": data_path },
            "usageHint": "body"
        } }
    }));
}

/// Builds agent section components (title, rows, or empty placeholder).
fn build_agents_section(components: &mut SmallVec<[Value; 48]>, agents: &AgentInfoList) {
    let mut section_children: Vec<Value> = vec![json!("task-status-agents-title")];

    components.push(json!({
        "id": "task-status-agents-title",
        "component": { "Text": {
            "text": { "literalString": "Agents" },
            "usageHint": "h3"
        } }
    }));

    if agents.is_empty() {
        section_children.push(json!("task-status-agents-empty"));
        components.push(json!({
            "id": "task-status-agents-empty",
            "component": { "Text": {
                "text": { "literalString": "No agents yet" },
                "usageHint": "caption"
            } }
        }));
    } else {
        for (i, _agent) in agents.iter().enumerate() {
            let row_id = format!("task-status-agent-{i}-row");
            let name_id = format!("task-status-agent-{i}-type");
            let phase_id = format!("task-status-agent-{i}-phase");

            section_children.push(json!(row_id));

            components.push(json!({
                "id": row_id,
                "component": { "Row": {
                    "distribution": "spaceBetween",
                    "children": { "explicitList": [name_id, phase_id] }
                } }
            }));
            components.push(json!({
                "id": name_id,
                "component": { "Text": {
                    "text": { "path": format!("/task/agents/{i}/type") },
                    "usageHint": "body"
                } }
            }));
            components.push(json!({
                "id": phase_id,
                "component": { "Text": {
                    "text": { "path": format!("/task/agents/{i}/phase") },
                    "usageHint": "body"
                } }
            }));
        }
    }

    components.push(json!({
        "id": "task-status-agents-section",
        "component": { "Column": {
            "children": { "explicitList": section_children }
        } }
    }));
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::agent_info::AgentInfoList;
    use crate::crd::AgentTaskPhase;

    fn sample_event() -> TaskStateChanged {
        TaskStateChanged {
            task_name: "task-abc12345".to_string(),
            namespace: "forgemaster-system".to_string(),
            description: "Build a REST API".to_string(),
            created_at: Some(Utc::now() - chrono::Duration::seconds(150)),
            phase: AgentTaskPhase::Running,
            iteration: 2,
            error: 0.4,
            tests_total: 5,
            tests_passed: 3,
            agents: AgentInfoList::new(),
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
        assert!(ids.contains(&"task-status-age-value"));
        assert!(ids.contains(&"task-status-phase-value"));
        assert!(ids.contains(&"task-status-iteration-value"));
        assert!(ids.contains(&"task-status-error-value"));
        assert!(ids.contains(&"task-status-tests-value"));
        assert!(ids.contains(&"task-status-agents-section"));
        assert!(ids.contains(&"task-status-agents-title"));
    }

    #[test]
    fn hero_section_override_includes_task_status_card() {
        let (_, components, _) = build_task_status_schema(&sample_event());
        let hero_section = components
            .iter()
            .find(|c| c["id"] == "hero-section")
            .unwrap();
        let children = &hero_section["component"]["Column"]["children"]["explicitList"];
        assert!(
            children
                .as_array()
                .unwrap()
                .contains(&json!("task-status-card"))
        );
    }

    #[test]
    fn data_contains_task_fields() {
        let (_, _, data) = build_task_status_schema(&sample_event());
        assert_eq!(data["task"]["name"], "task-abc12345");
        assert_eq!(data["task"]["description"], "Build a REST API");
        assert_eq!(data["task"]["age"], "2m 30s");
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
        assert_eq!(
            name_value["component"]["Text"]["text"]["path"],
            "/task/name"
        );
    }

    #[test]
    fn components_count_no_agents() {
        // hero-section(1) + card(1) + col(1) + title(1) + 7 rows * 3 components(21)
        // + agents-title(1) + agents-empty(1) + agents-section(1) = 28
        let (_, components, _) = build_task_status_schema(&sample_event());
        assert_eq!(components.len(), 28);
    }

    #[test]
    fn format_age_seconds_only() {
        assert_eq!(format_age(45), "45s");
    }

    #[test]
    fn format_age_minutes_and_seconds() {
        assert_eq!(format_age(150), "2m 30s");
    }

    #[test]
    fn format_age_hours_and_minutes() {
        assert_eq!(format_age(3725), "1h 2m");
    }

    #[test]
    fn format_age_negative_returns_zero() {
        assert_eq!(format_age(-5), "0s");
    }

    #[test]
    fn age_unknown_when_no_created_at() {
        let mut event = sample_event();
        event.created_at = None;
        let (_, _, data) = build_task_status_schema(&event);
        assert_eq!(data["task"]["age"], "unknown");
    }

    #[test]
    fn data_contains_empty_agents_array() {
        let (_, _, data) = build_task_status_schema(&sample_event());
        assert!(data["task"]["agents"].is_array());
        assert_eq!(data["task"]["agents"].as_array().unwrap().len(), 0);
    }

    #[test]
    fn empty_agents_shows_no_agents_text() {
        let (_, components, _) = build_task_status_schema(&sample_event());
        let ids: Vec<&str> = components
            .iter()
            .filter_map(|c| c.get("id").and_then(|v| v.as_str()))
            .collect();
        assert!(ids.contains(&"task-status-agents-empty"));
    }

    fn sample_event_with_agents() -> TaskStateChanged {
        use crate::agent_info::AgentInfo;
        use smallvec::smallvec;

        TaskStateChanged {
            task_name: "task-abc12345".to_string(),
            namespace: "forgemaster-system".to_string(),
            description: "Build a REST API".to_string(),
            created_at: Some(Utc::now() - chrono::Duration::seconds(150)),
            phase: AgentTaskPhase::Running,
            iteration: 2,
            error: 0.4,
            tests_total: 5,
            tests_passed: 3,
            agents: smallvec![
                AgentInfo {
                    name: "orchestrator-task-abc12345".to_string(),
                    agent_type: "orchestrator".to_string(),
                    phase: "Running".to_string(),
                },
                AgentInfo {
                    name: "code-gen-task-abc12345".to_string(),
                    agent_type: "code-generator".to_string(),
                    phase: "Pending".to_string(),
                },
            ],
        }
    }

    #[test]
    fn agents_generate_row_components() {
        let (_, components, _) = build_task_status_schema(&sample_event_with_agents());
        let ids: Vec<&str> = components
            .iter()
            .filter_map(|c| c.get("id").and_then(|v| v.as_str()))
            .collect();

        assert!(ids.contains(&"task-status-agent-0-row"));
        assert!(ids.contains(&"task-status-agent-0-type"));
        assert!(ids.contains(&"task-status-agent-0-phase"));
        assert!(ids.contains(&"task-status-agent-1-row"));
        assert!(ids.contains(&"task-status-agent-1-type"));
        assert!(ids.contains(&"task-status-agent-1-phase"));
        // No empty message when agents present
        assert!(!ids.contains(&"task-status-agents-empty"));
    }

    #[test]
    fn agents_data_populated() {
        let (_, _, data) = build_task_status_schema(&sample_event_with_agents());
        let agents = data["task"]["agents"].as_array().unwrap();
        assert_eq!(agents.len(), 2);
        assert_eq!(agents[0]["type"], "orchestrator");
        assert_eq!(agents[0]["phase"], "Running");
        assert_eq!(agents[1]["type"], "code-generator");
        assert_eq!(agents[1]["phase"], "Pending");
    }

    #[test]
    fn agent_components_use_data_bindings() {
        let (_, components, _) = build_task_status_schema(&sample_event_with_agents());
        let type_component = components
            .iter()
            .find(|c| c["id"] == "task-status-agent-0-type")
            .unwrap();
        assert_eq!(
            type_component["component"]["Text"]["text"]["path"],
            "/task/agents/0/type"
        );

        let phase_component = components
            .iter()
            .find(|c| c["id"] == "task-status-agent-1-phase")
            .unwrap();
        assert_eq!(
            phase_component["component"]["Text"]["text"]["path"],
            "/task/agents/1/phase"
        );
    }

    #[test]
    fn components_count_with_agents() {
        // 28 base (without agent rows) - 2 (no empty text + its reference)
        // + 2 agents * 3 components = 6 → 28 - 1 (agents-empty) + 6 = 33
        let (_, components, _) = build_task_status_schema(&sample_event_with_agents());
        assert_eq!(components.len(), 33);
    }

    #[test]
    fn agents_section_in_col_children() {
        let (_, components, _) = build_task_status_schema(&sample_event());
        let col = components
            .iter()
            .find(|c| c["id"] == "task-status-col")
            .unwrap();
        let children = col["component"]["Column"]["children"]["explicitList"]
            .as_array()
            .unwrap();
        assert!(children.contains(&json!("task-status-agents-section")));
    }
}
