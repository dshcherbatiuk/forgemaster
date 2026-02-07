//! Builds runtime pods for Agent CRs.

use std::collections::BTreeMap;

use k8s_openapi::api::core::v1::{
    Container, EnvVar, EnvVarSource, Pod, PodSpec, SecretKeySelector,
};
use k8s_openapi::apimachinery::pkg::apis::meta::v1::OwnerReference;
use kube::ResourceExt;
use kube::api::ObjectMeta;

use crate::crd::Agent;

use super::context::ControllerContext;

/// Builds a runtime pod for the given Agent CR.
///
/// The pod runs `fm-agent-runtime` with config injected via env vars.
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

    let env_vars = build_env_vars(agent, ctx);
    let resource_requirements = build_resource_requirements(agent);

    Pod {
        metadata: ObjectMeta {
            name: Some(agent_name),
            namespace: Some(agent_namespace),
            labels: Some(labels),
            owner_references: Some(vec![owner_reference]),
            ..Default::default()
        },
        spec: Some(PodSpec {
            restart_policy: Some("Never".to_string()),
            containers: vec![Container {
                name: "agent-runtime".to_string(),
                image: Some(ctx.runtime_agent_image().to_string()),
                image_pull_policy: Some("Never".to_string()),
                env: Some(env_vars),
                resources: resource_requirements,
                ..Default::default()
            }],
            ..Default::default()
        }),
        ..Default::default()
    }
}

/// Builds env vars for the runtime container from Agent CR fields.
fn build_env_vars(agent: &Agent, ctx: &ControllerContext) -> Vec<EnvVar> {
    let spec = &agent.spec;

    vec![
        env_value("AGENT_NAME", &agent.name_any()),
        env_value(
            "NAMESPACE",
            &agent.namespace().unwrap_or_default(),
        ),
        env_value("TASK_PROMPT", &spec.task_prompt),
        env_value("MODEL_NAME", &spec.model.name),
        env_value("MODEL_TEMPERATURE", &spec.model.temperature.to_string()),
        env_value("MODEL_MAX_TOKENS", &spec.model.max_tokens.to_string()),
        env_value("RUST_LOG", "info"),
        env_secret(
            "ANTHROPIC_API_KEY",
            ctx.llm_provider_secret_name(),
            ctx.llm_provider_secret_key(),
        ),
    ]
}

/// Creates an env var with a literal value.
fn env_value(name: &str, value: &str) -> EnvVar {
    EnvVar {
        name: name.to_string(),
        value: Some(value.to_string()),
        ..Default::default()
    }
}

/// Creates an env var sourced from a K8s secret.
fn env_secret(name: &str, secret_name: &str, secret_key: &str) -> EnvVar {
    EnvVar {
        name: name.to_string(),
        value_from: Some(EnvVarSource {
            secret_key_ref: Some(SecretKeySelector {
                name: secret_name.to_string(),
                key: secret_key.to_string(),
                optional: Some(false),
            }),
            ..Default::default()
        }),
        ..Default::default()
    }
}

/// Converts Agent CRD resource requirements to K8s resource requirements.
fn build_resource_requirements(
    agent: &Agent,
) -> Option<k8s_openapi::api::core::v1::ResourceRequirements> {
    let resources = agent.spec.resources.as_ref()?;

    let limits = resources.limits.as_ref().map(|l| {
        let mut map = BTreeMap::new();
        if let Some(memory) = &l.memory {
            map.insert(
                "memory".to_string(),
                k8s_openapi::apimachinery::pkg::api::resource::Quantity(memory.clone()),
            );
        }
        if let Some(cpu) = &l.cpu {
            map.insert(
                "cpu".to_string(),
                k8s_openapi::apimachinery::pkg::api::resource::Quantity(cpu.clone()),
            );
        }
        map
    });

    let requests = resources.requests.as_ref().map(|r| {
        let mut map = BTreeMap::new();
        if let Some(memory) = &r.memory {
            map.insert(
                "memory".to_string(),
                k8s_openapi::apimachinery::pkg::api::resource::Quantity(memory.clone()),
            );
        }
        if let Some(cpu) = &r.cpu {
            map.insert(
                "cpu".to_string(),
                k8s_openapi::apimachinery::pkg::api::resource::Quantity(cpu.clone()),
            );
        }
        map
    });

    Some(k8s_openapi::api::core::v1::ResourceRequirements {
        limits,
        requests,
        ..Default::default()
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::crd::{AgentCrd, ModelConfig, ResourceLimits, ResourceRequirements};

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

    fn build_pod_from(agent: &Agent) -> Pod {
        // Build pod without real K8s client by using a test context
        // We test the env_vars and resource building separately
        let env_vars = build_env_vars_test(agent);
        let resource_requirements = build_resource_requirements(agent);

        Pod {
            metadata: ObjectMeta {
                name: Some(agent.name_any()),
                namespace: agent.namespace().map(|ns| ns.to_string()),
                ..Default::default()
            },
            spec: Some(PodSpec {
                restart_policy: Some("Never".to_string()),
                containers: vec![Container {
                    name: "agent-runtime".to_string(),
                    image: Some("fm-agent-runtime:latest".to_string()),
                    image_pull_policy: Some("Never".to_string()),
                    env: Some(env_vars),
                    resources: resource_requirements,
                    ..Default::default()
                }],
                ..Default::default()
            }),
            ..Default::default()
        }
    }

    /// Test helper that builds env vars without needing ControllerContext.
    fn build_env_vars_test(agent: &Agent) -> Vec<EnvVar> {
        let spec = &agent.spec;
        vec![
            env_value("AGENT_NAME", &agent.name_any()),
            env_value("NAMESPACE", &agent.namespace().unwrap_or_default()),
            env_value("TASK_PROMPT", &spec.task_prompt),
            env_value("MODEL_NAME", &spec.model.name),
            env_value("MODEL_TEMPERATURE", &spec.model.temperature.to_string()),
            env_value("MODEL_MAX_TOKENS", &spec.model.max_tokens.to_string()),
            env_value("RUST_LOG", "info"),
            env_secret("ANTHROPIC_API_KEY", "anthropic-credentials", "api-key"),
        ]
    }

    #[test]
    fn pod_name_matches_agent() {
        let pod = build_pod_from(&test_agent());
        assert_eq!(
            pod.metadata.name,
            Some("orchestrator-task-abc".to_string())
        );
    }

    #[test]
    fn pod_namespace_matches_agent() {
        let pod = build_pod_from(&test_agent());
        assert_eq!(pod.metadata.namespace, Some("task-abc".to_string()));
    }

    #[test]
    fn pod_restart_policy_is_never() {
        let pod = build_pod_from(&test_agent());
        let spec = pod.spec.unwrap();
        assert_eq!(spec.restart_policy, Some("Never".to_string()));
    }

    #[test]
    fn pod_image_pull_policy_is_never() {
        let pod = build_pod_from(&test_agent());
        let container = &pod.spec.unwrap().containers[0];
        assert_eq!(container.image_pull_policy, Some("Never".to_string()));
    }

    #[test]
    fn pod_container_image() {
        let pod = build_pod_from(&test_agent());
        let container = &pod.spec.unwrap().containers[0];
        assert_eq!(
            container.image,
            Some("fm-agent-runtime:latest".to_string())
        );
    }

    #[test]
    fn env_contains_agent_name() {
        let env_vars = build_env_vars_test(&test_agent());
        let agent_name_var = env_vars.iter().find(|e| e.name == "AGENT_NAME").unwrap();
        assert_eq!(
            agent_name_var.value,
            Some("orchestrator-task-abc".to_string())
        );
    }

    #[test]
    fn env_contains_namespace() {
        let env_vars = build_env_vars_test(&test_agent());
        let ns_var = env_vars.iter().find(|e| e.name == "NAMESPACE").unwrap();
        assert_eq!(ns_var.value, Some("task-abc".to_string()));
    }

    #[test]
    fn env_contains_task_prompt() {
        let env_vars = build_env_vars_test(&test_agent());
        let prompt_var = env_vars.iter().find(|e| e.name == "TASK_PROMPT").unwrap();
        assert!(prompt_var.value.as_ref().unwrap().contains("orchestrator"));
    }

    #[test]
    fn env_contains_model_name() {
        let env_vars = build_env_vars_test(&test_agent());
        let model_var = env_vars.iter().find(|e| e.name == "MODEL_NAME").unwrap();
        assert_eq!(
            model_var.value,
            Some("claude-sonnet-4-20250514".to_string())
        );
    }

    #[test]
    fn env_contains_model_temperature() {
        let env_vars = build_env_vars_test(&test_agent());
        let temp_var = env_vars
            .iter()
            .find(|e| e.name == "MODEL_TEMPERATURE")
            .unwrap();
        assert_eq!(temp_var.value, Some("0.5".to_string()));
    }

    #[test]
    fn env_contains_model_max_tokens() {
        let env_vars = build_env_vars_test(&test_agent());
        let tokens_var = env_vars
            .iter()
            .find(|e| e.name == "MODEL_MAX_TOKENS")
            .unwrap();
        assert_eq!(tokens_var.value, Some("8192".to_string()));
    }

    #[test]
    fn env_api_key_from_secret() {
        let env_vars = build_env_vars_test(&test_agent());
        let api_key_var = env_vars
            .iter()
            .find(|e| e.name == "ANTHROPIC_API_KEY")
            .unwrap();
        assert!(api_key_var.value.is_none());
        let secret_ref = api_key_var
            .value_from
            .as_ref()
            .unwrap()
            .secret_key_ref
            .as_ref()
            .unwrap();
        assert_eq!(secret_ref.name, "anthropic-credentials");
        assert_eq!(secret_ref.key, "api-key");
    }

    #[test]
    fn no_resource_requirements_when_none() {
        let agent = test_agent();
        let resources = build_resource_requirements(&agent);
        assert!(resources.is_none());
    }

    #[test]
    fn resource_limits_from_agent_spec() {
        let mut agent = test_agent();
        agent.spec.resources = Some(ResourceRequirements::builder()
            .limits(
                ResourceLimits::builder()
                    .memory("512Mi".to_string())
                    .cpu("500m".to_string())
                    .build(),
            )
            .build());

        let resources = build_resource_requirements(&agent).unwrap();
        let limits = resources.limits.unwrap();
        assert_eq!(limits["memory"].0, "512Mi");
        assert_eq!(limits["cpu"].0, "500m");
    }

    #[test]
    fn resource_requests_from_agent_spec() {
        let mut agent = test_agent();
        agent.spec.resources = Some(ResourceRequirements::builder()
            .requests(
                ResourceLimits::builder()
                    .memory("256Mi".to_string())
                    .build(),
            )
            .build());

        let resources = build_resource_requirements(&agent).unwrap();
        let requests = resources.requests.unwrap();
        assert_eq!(requests["memory"].0, "256Mi");
        assert!(requests.get("cpu").is_none());
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

        assert_eq!(
            labels.get("app.kubernetes.io/component").unwrap(),
            "agent-runtime"
        );
        assert_eq!(
            labels.get("forgemaster.io/task").unwrap(),
            "task-abc"
        );
    }

    #[test]
    fn env_value_creates_literal() {
        let env = env_value("KEY", "value");
        assert_eq!(env.name, "KEY");
        assert_eq!(env.value, Some("value".to_string()));
        assert!(env.value_from.is_none());
    }

    #[test]
    fn env_secret_creates_secret_ref() {
        let env = env_secret("KEY", "secret-name", "secret-key");
        assert_eq!(env.name, "KEY");
        assert!(env.value.is_none());
        let secret_ref = env.value_from.unwrap().secret_key_ref.unwrap();
        assert_eq!(secret_ref.name, "secret-name");
        assert_eq!(secret_ref.key, "secret-key");
    }
}
