//! Builds A2UI schema components for multi-task status display with tab bar.

use chrono::Utc;
use serde_json::{Value, json};
use smallvec::SmallVec;

use crate::agent_info::AgentInfoList;
use crate::task_state_changed::TaskStateChanged;

use super::active_task_store::DEFAULT_MAX_ACTIVE_TASKS;

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

/// Root component ID for the multi-task schema.
pub const MULTI_TASK_ROOT: &str = "hero-section";

/// Builds A2UI schema for displaying up to N tasks with a tab bar.
///
/// Returns (root, components, data) matching the A2UI v0.8 format.
/// The input form is hidden when the task limit is reached.
pub fn build_multi_task_schema(
    active_tasks: &[TaskStateChanged],
) -> (String, SmallVec<[Value; 96]>, Value) {
    build_multi_task_schema_with_limit(active_tasks, DEFAULT_MAX_ACTIVE_TASKS)
}

/// Builds the multi-task schema with an explicit task limit (for testing).
pub fn build_multi_task_schema_with_limit(
    active_tasks: &[TaskStateChanged],
    max_tasks: usize,
) -> (String, SmallVec<[Value; 96]>, Value) {
    let mut components: SmallVec<[Value; 96]> = SmallVec::new();

    // Build hero-section children
    let mut hero_children: Vec<Value> = vec![json!("hero-tagline"), json!("hero-subtitle")];

    // Show task input form only when below the limit
    if active_tasks.len() < max_tasks {
        hero_children.push(json!("task-card"));
    }

    // Tab bar when there are active tasks
    if !active_tasks.is_empty() {
        hero_children.push(json!("task-tab-bar"));
    }

    // One status card per task
    for i in 0..active_tasks.len() {
        hero_children.push(json!(format!("task-{i}-status-card")));
    }

    components.push(json!({
        "id": "hero-section",
        "component": { "Column": {
            "children": { "explicitList": hero_children }
        } }
    }));

    // Tab bar
    if !active_tasks.is_empty() {
        build_tab_bar(&mut components, active_tasks);
    }

    // Per-task cards + data
    let mut items: Vec<Value> = Vec::with_capacity(active_tasks.len());
    for (i, event) in active_tasks.iter().enumerate() {
        let task_data = build_single_task_components(i, event, &mut components);
        items.push(task_data);
    }

    let data = json!({
        "hero": {
            "tagline": "AI-Powered Code Generation",
            "subtitle": "Describe your task. Let agents build it. Tests drive the loop."
        },
        "tasks": {
            "count": active_tasks.len(),
            "items": items,
        }
    });

    (MULTI_TASK_ROOT.to_string(), components, data)
}

/// Builds tab bar — a Row with one text button per task.
fn build_tab_bar(components: &mut SmallVec<[Value; 96]>, tasks: &[TaskStateChanged]) {
    let tab_children: Vec<Value> = (0..tasks.len())
        .map(|i| json!(format!("task-tab-{i}")))
        .collect();

    components.push(json!({
        "id": "task-tab-bar",
        "component": { "Row": {
            "distribution": "start",
            "children": { "explicitList": tab_children }
        } }
    }));

    for i in 0..tasks.len() {
        components.push(json!({
            "id": format!("task-tab-{i}"),
            "component": { "Text": {
                "text": { "path": format!("/tasks/items/{i}/name") },
                "usageHint": "body"
            } }
        }));
    }
}

/// Builds components for a single task status card at the given index.
///
/// All IDs are namespaced: `task-{index}-status-*`.
/// Data paths: `/tasks/items/{index}/*`.
/// Returns the data value for this task.
fn build_single_task_components(
    index: usize,
    event: &TaskStateChanged,
    components: &mut SmallVec<[Value; 96]>,
) -> Value {
    let prefix = format!("task-{index}");
    let data_prefix = format!("/tasks/items/{index}");

    let card_id = format!("{prefix}-status-card");
    let col_id = format!("{prefix}-status-col");
    let title_id = format!("{prefix}-status-title");
    let agents_section_id = format!("{prefix}-status-agents-section");

    components.push(json!({
        "id": card_id,
        "component": { "Card": { "child": col_id } }
    }));

    components.push(json!({
        "id": title_id,
        "component": { "Text": {
            "text": { "literalString": "Task Status" },
            "usageHint": "h2"
        } }
    }));

    push_label_value_row(components, &prefix, "name", "Name:", &data_prefix);
    push_label_value_row(components, &prefix, "desc", "Description:", &data_prefix);
    push_label_value_row(components, &prefix, "age", "Age:", &data_prefix);
    push_label_value_row(components, &prefix, "phase", "Phase:", &data_prefix);
    push_label_value_row(components, &prefix, "iteration", "Iteration:", &data_prefix);
    push_label_value_row(components, &prefix, "error", "Error Signal:", &data_prefix);
    push_label_value_row(components, &prefix, "tests", "Tests:", &data_prefix);

    build_agents_section(components, &prefix, &data_prefix, &event.agents);

    let col_children = vec![
        json!(&title_id),
        json!(format!("{prefix}-status-name-row")),
        json!(format!("{prefix}-status-desc-row")),
        json!(format!("{prefix}-status-age-row")),
        json!(format!("{prefix}-status-phase-row")),
        json!(format!("{prefix}-status-iteration-row")),
        json!(format!("{prefix}-status-error-row")),
        json!(format!("{prefix}-status-tests-row")),
        json!(&agents_section_id),
    ];

    components.push(json!({
        "id": col_id,
        "component": { "Column": {
            "children": { "explicitList": col_children }
        } }
    }));

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

    json!({
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
    })
}

/// Pushes a label-value row (3 components: row, label text, value text).
fn push_label_value_row(
    components: &mut SmallVec<[Value; 96]>,
    prefix: &str,
    field: &str,
    label: &str,
    data_prefix: &str,
) {
    let row_id = format!("{prefix}-status-{field}-row");
    let label_id = format!("{prefix}-status-{field}-label");
    let value_id = format!("{prefix}-status-{field}-value");
    let data_path = match field {
        "tests" => format!("{data_prefix}/testsDisplay"),
        other => format!("{data_prefix}/{other}"),
    };

    components.push(json!({
        "id": row_id,
        "component": { "Row": {
            "distribution": "spaceBetween",
            "children": { "explicitList": [&label_id, &value_id] }
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

/// Builds agent section components (title + agent rows or empty placeholder).
fn build_agents_section(
    components: &mut SmallVec<[Value; 96]>,
    prefix: &str,
    data_prefix: &str,
    agents: &AgentInfoList,
) {
    let title_id = format!("{prefix}-status-agents-title");
    let section_id = format!("{prefix}-status-agents-section");
    let mut section_children: Vec<Value> = vec![json!(&title_id)];

    components.push(json!({
        "id": &title_id,
        "component": { "Text": {
            "text": { "literalString": "Agents" },
            "usageHint": "h3"
        } }
    }));

    if agents.is_empty() {
        let empty_id = format!("{prefix}-status-agents-empty");
        section_children.push(json!(&empty_id));
        components.push(json!({
            "id": &empty_id,
            "component": { "Text": {
                "text": { "literalString": "No agents yet" },
                "usageHint": "caption"
            } }
        }));
    } else {
        for (i, _agent) in agents.iter().enumerate() {
            let row_id = format!("{prefix}-status-agent-{i}-row");
            let type_id = format!("{prefix}-status-agent-{i}-type");
            let phase_id = format!("{prefix}-status-agent-{i}-phase");

            section_children.push(json!(&row_id));

            components.push(json!({
                "id": &row_id,
                "component": { "Row": {
                    "distribution": "spaceBetween",
                    "children": { "explicitList": [&type_id, &phase_id] }
                } }
            }));
            components.push(json!({
                "id": &type_id,
                "component": { "Text": {
                    "text": { "path": format!("{data_prefix}/agents/{i}/type") },
                    "usageHint": "body"
                } }
            }));
            components.push(json!({
                "id": &phase_id,
                "component": { "Text": {
                    "text": { "path": format!("{data_prefix}/agents/{i}/phase") },
                    "usageHint": "body"
                } }
            }));
        }
    }

    components.push(json!({
        "id": &section_id,
        "component": { "Column": {
            "children": { "explicitList": section_children }
        } }
    }));
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::agent_info::{AgentInfo, AgentInfoList};
    use crate::crd::AgentTaskPhase;
    use smallvec::smallvec;

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

    fn second_event() -> TaskStateChanged {
        TaskStateChanged {
            task_name: "task-def67890".to_string(),
            namespace: "forgemaster-system".to_string(),
            description: "Create auth service".to_string(),
            created_at: Some(Utc::now() - chrono::Duration::seconds(60)),
            phase: AgentTaskPhase::Pending,
            iteration: 0,
            error: 1.0,
            tests_total: 0,
            tests_passed: 0,
            agents: AgentInfoList::new(),
        }
    }

    fn event_with_agents() -> TaskStateChanged {
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

    fn get_ids(components: &SmallVec<[Value; 96]>) -> Vec<String> {
        components
            .iter()
            .filter_map(|c| c.get("id").and_then(|v| v.as_str()).map(String::from))
            .collect()
    }

    // --- Root ---

    #[test]
    fn root_is_hero_section() {
        let (root, _, _) = build_multi_task_schema(&[sample_event()]);
        assert_eq!(root, MULTI_TASK_ROOT);
    }

    // --- Empty state ---

    #[test]
    fn empty_tasks_shows_only_input_form() {
        let (_, components, _) = build_multi_task_schema(&[]);
        let hero = components.iter().find(|c| c["id"] == "hero-section").unwrap();
        let children = hero["component"]["Column"]["children"]["explicitList"]
            .as_array()
            .unwrap();
        assert!(children.contains(&json!("task-card")));
        assert!(!children.iter().any(|c| c.as_str().map_or(false, |s| s.contains("status"))));
    }

    #[test]
    fn empty_tasks_data_has_zero_count() {
        let (_, _, data) = build_multi_task_schema(&[]);
        assert_eq!(data["tasks"]["count"], 0);
        assert!(data["tasks"]["items"].as_array().unwrap().is_empty());
    }

    #[test]
    fn empty_tasks_no_tab_bar() {
        let (_, components, _) = build_multi_task_schema(&[]);
        let ids = get_ids(&components);
        assert!(!ids.contains(&"task-tab-bar".to_string()));
    }

    // --- Single task ---

    #[test]
    fn one_task_includes_input_form() {
        let (_, components, _) = build_multi_task_schema(&[sample_event()]);
        let hero = components.iter().find(|c| c["id"] == "hero-section").unwrap();
        let children = hero["component"]["Column"]["children"]["explicitList"]
            .as_array()
            .unwrap();
        assert!(children.contains(&json!("task-card")));
    }

    #[test]
    fn one_task_includes_tab_bar() {
        let (_, components, _) = build_multi_task_schema(&[sample_event()]);
        let ids = get_ids(&components);
        assert!(ids.contains(&"task-tab-bar".to_string()));
        assert!(ids.contains(&"task-tab-0".to_string()));
    }

    #[test]
    fn one_task_has_status_card() {
        let (_, components, _) = build_multi_task_schema(&[sample_event()]);
        let ids = get_ids(&components);
        assert!(ids.contains(&"task-0-status-card".to_string()));
    }

    #[test]
    fn one_task_ids_namespaced() {
        let (_, components, _) = build_multi_task_schema(&[sample_event()]);
        let ids = get_ids(&components);
        assert!(ids.contains(&"task-0-status-name-value".to_string()));
        assert!(ids.contains(&"task-0-status-phase-value".to_string()));
        assert!(ids.contains(&"task-0-status-agents-section".to_string()));
    }

    #[test]
    fn one_task_data_paths() {
        let (_, components, _) = build_multi_task_schema(&[sample_event()]);
        let name_val = components
            .iter()
            .find(|c| c["id"] == "task-0-status-name-value")
            .unwrap();
        assert_eq!(name_val["component"]["Text"]["text"]["path"], "/tasks/items/0/name");
    }

    #[test]
    fn one_task_data_count() {
        let (_, _, data) = build_multi_task_schema(&[sample_event()]);
        assert_eq!(data["tasks"]["count"], 1);
    }

    #[test]
    fn one_task_data_fields() {
        let (_, _, data) = build_multi_task_schema(&[sample_event()]);
        let task = &data["tasks"]["items"][0];
        assert_eq!(task["name"], "task-abc12345");
        assert_eq!(task["description"], "Build a REST API");
        assert_eq!(task["phase"], "Running");
        assert_eq!(task["iteration"], 2);
        assert_eq!(task["testsDisplay"], "3/5");
    }

    // --- Two tasks ---

    #[test]
    fn two_tasks_hides_input_form() {
        let (_, components, _) = build_multi_task_schema(&[sample_event(), second_event()]);
        let hero = components.iter().find(|c| c["id"] == "hero-section").unwrap();
        let children = hero["component"]["Column"]["children"]["explicitList"]
            .as_array()
            .unwrap();
        assert!(!children.contains(&json!("task-card")));
    }

    #[test]
    fn two_tasks_has_two_tabs() {
        let (_, components, _) = build_multi_task_schema(&[sample_event(), second_event()]);
        let ids = get_ids(&components);
        assert!(ids.contains(&"task-tab-0".to_string()));
        assert!(ids.contains(&"task-tab-1".to_string()));
    }

    #[test]
    fn two_tasks_has_two_status_cards() {
        let (_, components, _) = build_multi_task_schema(&[sample_event(), second_event()]);
        let ids = get_ids(&components);
        assert!(ids.contains(&"task-0-status-card".to_string()));
        assert!(ids.contains(&"task-1-status-card".to_string()));
    }

    #[test]
    fn two_tasks_data_count() {
        let (_, _, data) = build_multi_task_schema(&[sample_event(), second_event()]);
        assert_eq!(data["tasks"]["count"], 2);
        assert_eq!(data["tasks"]["items"].as_array().unwrap().len(), 2);
    }

    #[test]
    fn two_tasks_second_data() {
        let (_, _, data) = build_multi_task_schema(&[sample_event(), second_event()]);
        let second = &data["tasks"]["items"][1];
        assert_eq!(second["name"], "task-def67890");
        assert_eq!(second["phase"], "Pending");
    }

    #[test]
    fn two_tasks_second_data_paths() {
        let (_, components, _) = build_multi_task_schema(&[sample_event(), second_event()]);
        let name_val = components
            .iter()
            .find(|c| c["id"] == "task-1-status-name-value")
            .unwrap();
        assert_eq!(name_val["component"]["Text"]["text"]["path"], "/tasks/items/1/name");
    }

    // --- Tab bar ---

    #[test]
    fn tab_bar_binds_to_task_name() {
        let (_, components, _) = build_multi_task_schema(&[sample_event()]);
        let tab = components.iter().find(|c| c["id"] == "task-tab-0").unwrap();
        assert_eq!(tab["component"]["Text"]["text"]["path"], "/tasks/items/0/name");
    }

    // --- Agents ---

    #[test]
    fn agents_namespaced() {
        let (_, components, _) = build_multi_task_schema(&[event_with_agents()]);
        let ids = get_ids(&components);
        assert!(ids.contains(&"task-0-status-agent-0-row".to_string()));
        assert!(ids.contains(&"task-0-status-agent-0-type".to_string()));
        assert!(ids.contains(&"task-0-status-agent-1-phase".to_string()));
    }

    #[test]
    fn agents_data_paths() {
        let (_, components, _) = build_multi_task_schema(&[event_with_agents()]);
        let type_comp = components
            .iter()
            .find(|c| c["id"] == "task-0-status-agent-0-type")
            .unwrap();
        assert_eq!(
            type_comp["component"]["Text"]["text"]["path"],
            "/tasks/items/0/agents/0/type"
        );
    }

    #[test]
    fn agents_data_in_items() {
        let (_, _, data) = build_multi_task_schema(&[event_with_agents()]);
        let agents = data["tasks"]["items"][0]["agents"].as_array().unwrap();
        assert_eq!(agents.len(), 2);
        assert_eq!(agents[0]["type"], "orchestrator");
    }

    #[test]
    fn empty_agents_placeholder() {
        let (_, components, _) = build_multi_task_schema(&[sample_event()]);
        let ids = get_ids(&components);
        assert!(ids.contains(&"task-0-status-agents-empty".to_string()));
    }

    // --- Format age ---

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
        let (_, _, data) = build_multi_task_schema(&[event]);
        assert_eq!(data["tasks"]["items"][0]["age"], "unknown");
    }

    // --- Hero data ---

    #[test]
    fn data_contains_hero() {
        let (_, _, data) = build_multi_task_schema(&[sample_event()]);
        assert_eq!(data["hero"]["tagline"], "AI-Powered Code Generation");
    }

    // --- Component counts ---

    #[test]
    fn count_one_task_no_agents() {
        // hero(1) + tab-bar(1) + tab-0(1) = 3
        // + card(1) + title(1) + 7*3(21) + agents-title(1) + agents-empty(1) + agents-section(1) + col(1) = 27
        // = 30
        let (_, components, _) = build_multi_task_schema(&[sample_event()]);
        assert_eq!(components.len(), 30);
    }

    #[test]
    fn count_two_tasks_no_agents() {
        // hero(1) + tab-bar(1) + tab-0(1) + tab-1(1) = 4
        // + 2 * 27 = 54
        // = 58
        let (_, components, _) = build_multi_task_schema(&[sample_event(), second_event()]);
        assert_eq!(components.len(), 58);
    }

    #[test]
    fn count_one_task_with_agents() {
        // hero(1) + tab-bar(1) + tab-0(1) = 3
        // + card(1) + title(1) + 7*3(21) + agents-title(1) + 2*3(6) + agents-section(1) + col(1) = 32
        // = 35
        let (_, components, _) = build_multi_task_schema(&[event_with_agents()]);
        assert_eq!(components.len(), 35);
    }

    // --- Configurable limit ---

    #[test]
    fn limit_one_hides_form_at_one_task() {
        let (_, components, _) = build_multi_task_schema_with_limit(&[sample_event()], 1);
        let hero = components.iter().find(|c| c["id"] == "hero-section").unwrap();
        let children = hero["component"]["Column"]["children"]["explicitList"]
            .as_array()
            .unwrap();
        assert!(!children.contains(&json!("task-card")));
    }

    #[test]
    fn limit_three_shows_form_at_two_tasks() {
        let (_, components, _) =
            build_multi_task_schema_with_limit(&[sample_event(), second_event()], 3);
        let hero = components.iter().find(|c| c["id"] == "hero-section").unwrap();
        let children = hero["component"]["Column"]["children"]["explicitList"]
            .as_array()
            .unwrap();
        assert!(children.contains(&json!("task-card")));
    }
}
