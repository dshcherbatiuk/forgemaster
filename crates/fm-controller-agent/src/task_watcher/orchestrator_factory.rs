//! Builds Orchestrator Agent CR from an AgentTask.

use std::collections::BTreeMap;

use fm_controller_agenttask::crd::AgentTask;
use kube::ResourceExt;
use kube::api::ObjectMeta;

use crate::crd::{Agent, AgentCrd, ModelConfig};

const ORCHESTRATOR_ROLE: &str = "\
You are an Orchestrator and Architect agent for ForgeMaster.

Phase 1 — Requirements Analysis:
- Parse the task description to identify the domain, scope, and boundaries
- Extract functional and non-functional requirements
- Define acceptance criteria for the overall task
- Identify the technology stack, constraints, and dependencies

Phase 2 — Architecture & Decomposition:
- Design the solution architecture based on the requirements
- Decompose into subtasks, each with clear inputs, outputs, and acceptance criteria
- Decide which executor agents are needed (code-generator, test-generator, test-runner, reviewer)
- Provide each agent with domain-specific context and requirements so they can produce accurate results

Phase 3 — Orchestration:
- Create executor Agent CRs and MCPServer CRs via K8s API
- Coordinate agent execution and collect results
- Report progress and handle failures";

/// Builds an Orchestrator Agent CR for the given AgentTask.
pub fn build(task: &AgentTask, model_name: &str) -> Agent {
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
            task_prompt: format!(
                "{}\n\n---\n\nTask ID: {}\n\nTask Description:\n{}",
                ORCHESTRATOR_ROLE, task_name, task.spec.description
            ),
            mcp_servers: vec![],
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

    #[test]
    fn agent_name_contains_task_name() {
        let agent = build(&test_task(), "claude-sonnet-4-20250514");
        assert_eq!(
            agent.metadata.name,
            Some("orchestrator-task-abc123".to_string())
        );
    }

    #[test]
    fn agent_namespace_matches_task() {
        let agent = build(&test_task(), "claude-sonnet-4-20250514");
        assert_eq!(agent.metadata.namespace, Some("task-abc123".to_string()));
    }

    #[test]
    fn agent_type_is_orchestrator() {
        let agent = build(&test_task(), "claude-sonnet-4-20250514");
        assert_eq!(agent.spec.agent_type, "orchestrator");
    }

    #[test]
    fn agent_has_task_label() {
        let agent = build(&test_task(), "claude-sonnet-4-20250514");
        let labels = agent.metadata.labels.as_ref().unwrap();
        assert_eq!(labels.get("forgemaster.io/task").unwrap(), "task-abc123");
    }

    #[test]
    fn agent_has_type_label() {
        let agent = build(&test_task(), "claude-sonnet-4-20250514");
        let labels = agent.metadata.labels.as_ref().unwrap();
        assert_eq!(labels.get("forgemaster.io/type").unwrap(), "orchestrator");
    }

    #[test]
    fn agent_has_no_owner_reference() {
        let agent = build(&test_task(), "claude-sonnet-4-20250514");
        // Cross-namespace ownerRefs not supported by K8s.
        // Cleanup via namespace deletion instead.
        assert!(agent.metadata.owner_references.is_none());
    }

    #[test]
    fn task_prompt_contains_architect_role() {
        let agent = build(&test_task(), "claude-sonnet-4-20250514");
        assert!(agent.spec.task_prompt.contains("Architect"));
    }

    #[test]
    fn task_prompt_contains_task_description() {
        let agent = build(&test_task(), "claude-sonnet-4-20250514");
        assert!(agent.spec.task_prompt.contains("Build an e-commerce API"));
        assert!(agent.spec.task_prompt.contains("task-abc123"));
    }

    #[test]
    fn model_defaults_to_sonnet() {
        let agent = build(&test_task(), "claude-sonnet-4-20250514");
        assert_eq!(agent.spec.model.name, "claude-sonnet-4-20250514");
    }
}
