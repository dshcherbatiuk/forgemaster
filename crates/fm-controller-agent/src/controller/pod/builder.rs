//! Assembles runtime pods for Agent CRs.
//!
//! Delegates env var building, resource requirements, and workspace volume
//! to their respective modules within the `pod` module.

use k8s_openapi::api::core::v1::{Container, Pod, PodSpec};
use k8s_openapi::apimachinery::pkg::apis::meta::v1::OwnerReference;
use kube::ResourceExt;
use kube::api::ObjectMeta;

use crate::controller::context::ControllerContext;
use crate::crd::Agent;

/// Builds a runtime pod for the given Agent CR.
///
/// The pod runs `fm-agent-runtime-claude` with config injected via env vars.
/// It is a run-once pod (`restartPolicy: Never`) owned by the Agent CR.
pub fn build(agent: &Agent, ctx: &ControllerContext) -> Pod {
    let agent_name = agent.name_any();
    let agent_namespace = agent.namespace().unwrap_or_default();
    let agent_uid = agent.metadata.uid.clone().unwrap_or_default();

    let mut labels = agent.metadata.labels.clone().unwrap_or_default();
    labels.insert(
        "app.kubernetes.io/component".to_string(),
        "agent-runtime".to_string(),
    );

    let owner_reference = OwnerReference {
        api_version: "forgemaster.io/v1alpha1".to_string(),
        kind: "Agent".to_string(),
        name: agent_name.clone(),
        uid: agent_uid,
        controller: Some(true),
        block_owner_deletion: Some(true),
    };

    let env_vars = super::env_vars::build(agent, ctx);
    let resource_requirements = super::resource_requirements::build(agent);
    let (volumes, volume_mounts) = super::workspace_volume::build(
        ctx.workspace_base_path(),
        ctx.workspace_container_path(),
        &agent_namespace,
    );

    Pod {
        metadata: ObjectMeta {
            name: Some(agent_name),
            namespace: Some(agent_namespace),
            labels: Some(labels),
            owner_references: Some(vec![owner_reference]),
            ..Default::default()
        },
        spec: Some(PodSpec {
            service_account_name: Some(
                crate::controller::rbac_propagator::RUNTIME_SERVICE_ACCOUNT.to_string(),
            ),
            restart_policy: Some("Never".to_string()),
            volumes: if volumes.is_empty() { None } else { Some(volumes) },
            containers: vec![Container {
                name: "agent-runtime".to_string(),
                image: Some(ctx.runtime_agent_image().to_string()),
                image_pull_policy: Some("Never".to_string()),
                env: Some(env_vars),
                resources: resource_requirements,
                volume_mounts: if volume_mounts.is_empty() { None } else { Some(volume_mounts) },
                ..Default::default()
            }],
            ..Default::default()
        }),
        ..Default::default()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::BTreeMap;
    use crate::controller::pod::env_vars;
    use crate::controller::rbac_propagator;
    use crate::crd::{AgentCrd, ModelConfig};

    fn test_agent() -> Agent {
        Agent {
            metadata: ObjectMeta {
                name: Some("orchestrator-task-abc".to_string()),
                namespace: Some("task-abc".to_string()),
                uid: Some("uid-123".to_string()),
                labels: Some(BTreeMap::from([
                    ("forgemaster.io/task".to_string(), "task-abc".to_string()),
                ])),
                ..Default::default()
            },
            spec: AgentCrd {
                agent_type: "orchestrator".to_string(),
                model: ModelConfig::builder()
                    .name("claude-sonnet-4-20250514".to_string())
                    .temperature(0.5)
                    .max_tokens(8192)
                    .build(),
                task_prompt: "You are an orchestrator.\n\nTask: Build API".to_string(),
                mcp_servers: vec![],
                resources: None,
            },
            status: None,
        }
    }

    fn build_test_pod(agent: &Agent) -> Pod {
        let env_vars = vec![
            env_vars::literal("AGENT_NAME", &agent.name_any()),
            env_vars::literal("NAMESPACE", &agent.namespace().unwrap_or_default()),
        ];

        Pod {
            metadata: ObjectMeta {
                name: Some(agent.name_any()),
                namespace: agent.namespace().map(|ns| ns.to_string()),
                ..Default::default()
            },
            spec: Some(PodSpec {
                service_account_name: Some(
                    rbac_propagator::RUNTIME_SERVICE_ACCOUNT.to_string(),
                ),
                restart_policy: Some("Never".to_string()),
                containers: vec![Container {
                    name: "agent-runtime".to_string(),
                    image: Some("fm-agent-runtime-claude:latest".to_string()),
                    image_pull_policy: Some("Never".to_string()),
                    env: Some(env_vars),
                    ..Default::default()
                }],
                ..Default::default()
            }),
            ..Default::default()
        }
    }

    #[test]
    fn pod_name_matches_agent() {
        let pod = build_test_pod(&test_agent());
        assert_eq!(pod.metadata.name, Some("orchestrator-task-abc".to_string()));
    }

    #[test]
    fn pod_namespace_matches_agent() {
        let pod = build_test_pod(&test_agent());
        assert_eq!(pod.metadata.namespace, Some("task-abc".to_string()));
    }

    #[test]
    fn pod_restart_policy_is_never() {
        let pod = build_test_pod(&test_agent());
        assert_eq!(pod.spec.unwrap().restart_policy, Some("Never".to_string()));
    }

    #[test]
    fn pod_image_pull_policy_is_never() {
        let pod = build_test_pod(&test_agent());
        let container = &pod.spec.unwrap().containers[0];
        assert_eq!(container.image_pull_policy, Some("Never".to_string()));
    }

    #[test]
    fn pod_container_image() {
        let pod = build_test_pod(&test_agent());
        let container = &pod.spec.unwrap().containers[0];
        assert_eq!(container.image, Some("fm-agent-runtime-claude:latest".to_string()));
    }

    #[test]
    fn pod_service_account_name() {
        let pod = build_test_pod(&test_agent());
        assert_eq!(
            pod.spec.unwrap().service_account_name,
            Some("fm-agent-runtime-claude".to_string())
        );
    }

    #[test]
    fn owner_reference_points_to_agent() {
        let agent = test_agent();
        let owner_ref = OwnerReference {
            api_version: "forgemaster.io/v1alpha1".to_string(),
            kind: "Agent".to_string(),
            name: agent.name_any(),
            uid: agent.metadata.uid.clone().unwrap_or_default(),
            controller: Some(true),
            block_owner_deletion: Some(true),
        };

        assert_eq!(owner_ref.kind, "Agent");
        assert_eq!(owner_ref.name, "orchestrator-task-abc");
        assert_eq!(owner_ref.uid, "uid-123");
        assert!(owner_ref.controller.unwrap());
    }

    #[test]
    fn labels_include_component() {
        let agent = test_agent();
        let mut labels = agent.metadata.labels.clone().unwrap_or_default();
        labels.insert(
            "app.kubernetes.io/component".to_string(),
            "agent-runtime".to_string(),
        );

        assert_eq!(labels.get("app.kubernetes.io/component").unwrap(), "agent-runtime");
        assert_eq!(labels.get("forgemaster.io/task").unwrap(), "task-abc");
    }
}
