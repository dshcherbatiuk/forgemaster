//! Builds Orchestrator Agent CR from an AgentTask.

use std::collections::BTreeMap;

use fm_controller_agenttask::crd::AgentTask;
use k8s_openapi::apimachinery::pkg::apis::meta::v1::OwnerReference;
use kube::api::ObjectMeta;
use kube::ResourceExt;

use crate::crd::{Agent, AgentCrd, ModelConfig};

const ORCHESTRATOR_SYSTEM_PROMPT: &str = "\
You are an Orchestrator and Architect agent for ForgeMaster. Your role is to:
- Analyze the task description and design the solution architecture
- Decompose the task into subtasks and decide the implementation approach
- Decide which executor agents are needed (code-generator, test-generator, test-runner, reviewer)
- Create executor Agent CRs and MCPServer CRs via K8s API
- Coordinate agent execution and collect results
- Report progress and handle failures";

/// Builds an Orchestrator Agent CR for the given AgentTask.
pub fn build(task: &AgentTask) -> Agent {
    let task_name = task.name_any();
    let task_namespace = task.namespace().unwrap_or_default();
    let task_uid = task.metadata.uid.clone().unwrap_or_default();

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

    let owner_reference = OwnerReference {
        api_version: "forgemaster.io/v1alpha1".to_string(),
        kind: "AgentTask".to_string(),
        name: task_name.clone(),
        uid: task_uid,
        controller: Some(true),
        block_owner_deletion: Some(true),
    };

    Agent {
        metadata: ObjectMeta {
            name: Some(agent_name),
            namespace: Some(task_namespace),
            labels: Some(labels),
            owner_references: Some(vec![owner_reference]),
            ..Default::default()
        },
        spec: AgentCrd {
            agent_type: "orchestrator".to_string(),
            model: ModelConfig::builder()
                .name("claude-sonnet-4-20250514".to_string())
                .build(),
            system_prompt: format!(
                "{ORCHESTRATOR_SYSTEM_PROMPT}\n\nTask: {}\nDescription: {}",
                task_name, task.spec.description
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
                namespace: Some("task-abc123".to_string()),
                uid: Some("uid-xyz".to_string()),
                ..Default::default()
            },
            spec: AgentTaskCrd {
                description: "Build an e-commerce API".to_string(),
                timeout: "30m".to_string(),
                clarifications: vec![],
                resource_quota: None,
            },
            status: Some(AgentTaskStatus::builder().phase(AgentTaskPhase::Running).build()),
        }
    }

    #[test]
    fn agent_name_contains_task_name() {
        let agent = build(&test_task());
        assert_eq!(
            agent.metadata.name,
            Some("orchestrator-task-abc123".to_string())
        );
    }

    #[test]
    fn agent_namespace_matches_task() {
        let agent = build(&test_task());
        assert_eq!(
            agent.metadata.namespace,
            Some("task-abc123".to_string())
        );
    }

    #[test]
    fn agent_type_is_orchestrator() {
        let agent = build(&test_task());
        assert_eq!(agent.spec.agent_type, "orchestrator");
    }

    #[test]
    fn agent_has_task_label() {
        let agent = build(&test_task());
        let labels = agent.metadata.labels.as_ref().unwrap();
        assert_eq!(labels.get("forgemaster.io/task").unwrap(), "task-abc123");
    }

    #[test]
    fn agent_has_type_label() {
        let agent = build(&test_task());
        let labels = agent.metadata.labels.as_ref().unwrap();
        assert_eq!(labels.get("forgemaster.io/type").unwrap(), "orchestrator");
    }

    #[test]
    fn agent_has_owner_reference() {
        let agent = build(&test_task());
        let owners = agent.metadata.owner_references.as_ref().unwrap();
        assert_eq!(owners.len(), 1);
        assert_eq!(owners[0].kind, "AgentTask");
        assert_eq!(owners[0].name, "task-abc123");
        assert_eq!(owners[0].uid, "uid-xyz");
    }

    #[test]
    fn system_prompt_contains_architect_role() {
        let agent = build(&test_task());
        assert!(agent.spec.system_prompt.contains("Architect"));
    }

    #[test]
    fn system_prompt_contains_task_description() {
        let agent = build(&test_task());
        assert!(agent.spec.system_prompt.contains("Build an e-commerce API"));
    }

    #[test]
    fn model_defaults_to_sonnet() {
        let agent = build(&test_task());
        assert_eq!(agent.spec.model.name, "claude-sonnet-4-20250514");
    }
}
