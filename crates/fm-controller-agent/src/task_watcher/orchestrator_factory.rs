//! Builds Orchestrator Agent CR from an AgentTask.

use std::collections::BTreeMap;

use fm_controller_agenttask::crd::AgentTask;
use kube::ResourceExt;
use kube::api::ObjectMeta;

use crate::crd::{Agent, AgentCrd, McpServerRef, ModelConfig};

const ORCHESTRATOR_ROLE: &str =
    include_str!("../../resources/prompts/orchestrator-role.md");

/// Builds an Orchestrator Agent CR for the given AgentTask.
pub fn build(task: &AgentTask, model_name: &str, mcp_servers: &[McpServerRef]) -> Agent {
    let task_name = task.name_any();
    // Agent goes into the task-specific namespace (same name as the task)
    let task_namespace = task_name.clone();
    let agent_name = format!("orchestrator-{}", &task_name);

    let mut labels = BTreeMap::new();
    labels.insert("forgemaster.io/task".to_string(), task_name.clone());
    labels.insert(
        "forgemaster.io/type".to_string(),
        "orchestrator".to_string(),
    );
    labels.insert(
        "app.kubernetes.io/managed-by".to_string(),
        "fm-controller-agent".to_string(),
    );

    // No ownerReference — AgentTask lives in forgemaster-system,
    // Agent lives in the task namespace. Cross-namespace ownerRefs
    // are not supported by K8s. Cleanup is handled by namespace
    // deletion (AgentTask finalizer deletes the entire task namespace).

    let task_prompt = format!(
        "{}\n\n---\n\nTask ID: {}\nNamespace: {}\n\nTask Description:\n{}",
        ORCHESTRATOR_ROLE, task_name, task_namespace, task.spec.description
    );

    Agent {
        metadata: ObjectMeta {
            name: Some(agent_name),
            namespace: Some(task_namespace),
            labels: Some(labels),
            ..Default::default()
        },
        spec: AgentCrd {
            agent_type: "orchestrator".to_string(),
            model: ModelConfig::builder()
                .name(model_name.to_string())
                .build(),
            task_prompt,
            mcp_servers: mcp_servers.to_vec(),
            resources: None,
        },
        status: None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use fm_controller_agenttask::crd::{AgentTaskCrd, AgentTaskPhase, AgentTaskStatus};

    fn test_task() -> AgentTask {
        AgentTask {
            metadata: ObjectMeta {
                name: Some("task-abc123".to_string()),
                namespace: Some("forgemaster-system".to_string()),
                ..Default::default()
            },
            spec: AgentTaskCrd {
                description: "Build an e-commerce API".to_string(),
                timeout: "30m".to_string(),
                clarifications: vec![],
                resource_quota: None,
            },
            status: Some(
                AgentTaskStatus::builder()
                    .phase(AgentTaskPhase::Running)
                    .build(),
            ),
        }
    }

    fn build_test_agent() -> Agent {
        build(&test_task(), "claude-sonnet-4-20250514", &[])
    }

    #[test]
    fn agent_name_contains_task_name() {
        let agent = build_test_agent();
        assert_eq!(
            agent.metadata.name,
            Some("orchestrator-task-abc123".to_string())
        );
    }

    #[test]
    fn agent_namespace_matches_task() {
        let agent = build_test_agent();
        assert_eq!(agent.metadata.namespace, Some("task-abc123".to_string()));
    }

    #[test]
    fn agent_type_is_orchestrator() {
        let agent = build_test_agent();
        assert_eq!(agent.spec.agent_type, "orchestrator");
    }

    #[test]
    fn agent_has_task_label() {
        let agent = build_test_agent();
        let labels = agent.metadata.labels.as_ref().unwrap();
        assert_eq!(labels.get("forgemaster.io/task").unwrap(), "task-abc123");
    }

    #[test]
    fn agent_has_type_label() {
        let agent = build_test_agent();
        let labels = agent.metadata.labels.as_ref().unwrap();
        assert_eq!(labels.get("forgemaster.io/type").unwrap(), "orchestrator");
    }

    #[test]
    fn agent_has_no_owner_reference() {
        let agent = build_test_agent();
        // Cross-namespace ownerRefs not supported by K8s.
        // Cleanup via namespace deletion instead.
        assert!(agent.metadata.owner_references.is_none());
    }

    #[test]
    fn task_prompt_contains_architect_role() {
        let agent = build_test_agent();
        assert!(agent.spec.task_prompt.contains("Architect"));
    }

    #[test]
    fn task_prompt_contains_task_description() {
        let agent = build_test_agent();
        assert!(agent.spec.task_prompt.contains("Build an e-commerce API"));
        assert!(agent.spec.task_prompt.contains("task-abc123"));
    }

    #[test]
    fn task_prompt_contains_namespace() {
        let agent = build_test_agent();
        assert!(agent.spec.task_prompt.contains("Namespace: task-abc123"));
    }

    #[test]
    fn model_defaults_to_sonnet() {
        let agent = build_test_agent();
        assert_eq!(agent.spec.model.name, "claude-sonnet-4-20250514");
    }

    #[test]
    fn mcp_servers_empty_when_none_configured() {
        let agent = build_test_agent();
        assert!(agent.spec.mcp_servers.is_empty());
    }

    #[test]
    fn mcp_servers_from_config() {
        let servers = vec![
            McpServerRef::builder()
                .name("fm-controller-agent".to_string())
                .build(),
            McpServerRef::builder()
                .name("github-mcp".to_string())
                .port(9090)
                .build(),
        ];
        let agent = build(&test_task(), "claude-sonnet-4-20250514", &servers);
        assert_eq!(agent.spec.mcp_servers.len(), 2);
        assert_eq!(agent.spec.mcp_servers[0].name, "fm-controller-agent");
        assert_eq!(agent.spec.mcp_servers[1].name, "github-mcp");
        assert_eq!(agent.spec.mcp_servers[1].port, 9090);
    }
}
