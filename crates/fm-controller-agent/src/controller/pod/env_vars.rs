//! Builds environment variables for agent runtime pods.

use k8s_openapi::api::core::v1::{EnvVar, EnvVarSource, SecretKeySelector};
use kube::ResourceExt;

use crate::controller::context::ControllerContext;
use crate::crd::Agent;

/// Builds env vars for the runtime container from Agent CR fields.
pub fn build(agent: &Agent, ctx: &ControllerContext) -> Vec<EnvVar> {
    let spec = &agent.spec;
    let agent_namespace = agent.namespace().unwrap_or_default();

    let mut env_vars = vec![
        literal("AGENT_NAME", &agent.name_any()),
        literal("NAMESPACE", &agent_namespace),
        literal("TASK_PROMPT", &spec.task_prompt),
        literal("MODEL_NAME", &spec.model.name),
        literal("MODEL_TEMPERATURE", &spec.model.temperature.to_string()),
        literal("MODEL_MAX_TOKENS", &spec.model.max_tokens.to_string()),
        literal("RUST_LOG", "info"),
        from_secret(
            "ANTHROPIC_API_KEY",
            ctx.llm_provider_secret_name(),
            ctx.llm_provider_secret_key(),
        ),
    ];

    if !spec.mcp_servers.is_empty() {
        let mcp_urls = build_mcp_server_urls(&spec.mcp_servers, ctx.namespace());
        env_vars.push(literal("MCP_SERVER_URLS", &mcp_urls));
    }

    if ctx.workspace_base_path().is_some() {
        env_vars.push(literal("WORKSPACE_DIR", ctx.workspace_container_path()));
    }

    // A2A communication env vars
    env_vars.push(literal(
        "A2A_PORT",
        &crate::controller::service_creator::A2A_PORT.to_string(),
    ));
    env_vars.push(literal("AGENT_TYPE", &spec.agent_type));

    env_vars
}

/// Creates an env var with a literal value.
pub fn literal(name: &str, value: &str) -> EnvVar {
    EnvVar {
        name: name.to_string(),
        value: Some(value.to_string()),
        ..Default::default()
    }
}

/// Creates an env var sourced from a K8s secret.
pub fn from_secret(name: &str, secret_name: &str, secret_key: &str) -> EnvVar {
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

/// Builds a comma-separated list of MCP server URLs from server refs.
fn build_mcp_server_urls(servers: &[crate::crd::McpServerRef], namespace: &str) -> String {
    servers
        .iter()
        .map(|s| s.url(namespace))
        .collect::<Vec<_>>()
        .join(",")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::crd::{AgentCrd, McpServerRef, ModelConfig};
    use kube::api::ObjectMeta;
    use std::collections::BTreeMap;

    const TEST_CONTROLLER_NAMESPACE: &str = "forgemaster-system";

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

    /// Test helper that builds env vars without needing ControllerContext.
    fn build_test_env_vars(agent: &Agent) -> Vec<EnvVar> {
        let spec = &agent.spec;
        let agent_namespace = agent.namespace().unwrap_or_default();

        let mut env_vars = vec![
            literal("AGENT_NAME", &agent.name_any()),
            literal("NAMESPACE", &agent_namespace),
            literal("TASK_PROMPT", &spec.task_prompt),
            literal("MODEL_NAME", &spec.model.name),
            literal("MODEL_TEMPERATURE", &spec.model.temperature.to_string()),
            literal("MODEL_MAX_TOKENS", &spec.model.max_tokens.to_string()),
            literal("RUST_LOG", "info"),
            from_secret("ANTHROPIC_API_KEY", "anthropic-credentials", "api-key"),
        ];

        if !spec.mcp_servers.is_empty() {
            let mcp_urls = build_mcp_server_urls(&spec.mcp_servers, TEST_CONTROLLER_NAMESPACE);
            env_vars.push(literal("MCP_SERVER_URLS", &mcp_urls));
        }

        // A2A communication env vars
        env_vars.push(literal(
            "A2A_PORT",
            &crate::controller::service_creator::A2A_PORT.to_string(),
        ));
        env_vars.push(literal("AGENT_TYPE", &spec.agent_type));

        env_vars
    }

    #[test]
    fn env_contains_agent_name() {
        let env_vars = build_test_env_vars(&test_agent());
        let var = env_vars.iter().find(|e| e.name == "AGENT_NAME").unwrap();
        assert_eq!(var.value, Some("orchestrator-task-abc".to_string()));
    }

    #[test]
    fn env_contains_namespace() {
        let env_vars = build_test_env_vars(&test_agent());
        let var = env_vars.iter().find(|e| e.name == "NAMESPACE").unwrap();
        assert_eq!(var.value, Some("task-abc".to_string()));
    }

    #[test]
    fn env_contains_task_prompt() {
        let env_vars = build_test_env_vars(&test_agent());
        let var = env_vars.iter().find(|e| e.name == "TASK_PROMPT").unwrap();
        assert!(var.value.as_ref().unwrap().contains("orchestrator"));
    }

    #[test]
    fn env_contains_model_name() {
        let env_vars = build_test_env_vars(&test_agent());
        let var = env_vars.iter().find(|e| e.name == "MODEL_NAME").unwrap();
        assert_eq!(var.value, Some("claude-sonnet-4-20250514".to_string()));
    }

    #[test]
    fn env_contains_model_temperature() {
        let env_vars = build_test_env_vars(&test_agent());
        let var = env_vars.iter().find(|e| e.name == "MODEL_TEMPERATURE").unwrap();
        assert_eq!(var.value, Some("0.5".to_string()));
    }

    #[test]
    fn env_contains_model_max_tokens() {
        let env_vars = build_test_env_vars(&test_agent());
        let var = env_vars.iter().find(|e| e.name == "MODEL_MAX_TOKENS").unwrap();
        assert_eq!(var.value, Some("8192".to_string()));
    }

    #[test]
    fn env_api_key_from_secret() {
        let env_vars = build_test_env_vars(&test_agent());
        let var = env_vars.iter().find(|e| e.name == "ANTHROPIC_API_KEY").unwrap();
        assert!(var.value.is_none());
        let secret_ref = var.value_from.as_ref().unwrap().secret_key_ref.as_ref().unwrap();
        assert_eq!(secret_ref.name, "anthropic-credentials");
        assert_eq!(secret_ref.key, "api-key");
    }

    #[test]
    fn literal_creates_env_var() {
        let env = literal("KEY", "value");
        assert_eq!(env.name, "KEY");
        assert_eq!(env.value, Some("value".to_string()));
        assert!(env.value_from.is_none());
    }

    #[test]
    fn from_secret_creates_secret_ref() {
        let env = from_secret("KEY", "secret-name", "secret-key");
        assert_eq!(env.name, "KEY");
        assert!(env.value.is_none());
        let secret_ref = env.value_from.unwrap().secret_key_ref.unwrap();
        assert_eq!(secret_ref.name, "secret-name");
        assert_eq!(secret_ref.key, "secret-key");
    }

    #[test]
    fn no_mcp_server_urls_when_empty() {
        let env_vars = build_test_env_vars(&test_agent());
        assert!(env_vars.iter().all(|e| e.name != "MCP_SERVER_URLS"));
    }

    #[test]
    fn mcp_server_urls_single_server() {
        let mut agent = test_agent();
        agent.spec.mcp_servers = vec![McpServerRef::builder()
            .name("github-mcp".to_string())
            .build()];

        let env_vars = build_test_env_vars(&agent);
        let var = env_vars.iter().find(|e| e.name == "MCP_SERVER_URLS").unwrap();
        assert_eq!(
            var.value,
            Some("http://github-mcp.forgemaster-system.svc.cluster.local:3000/mcp".to_string())
        );
    }

    #[test]
    fn mcp_server_urls_multiple_servers() {
        let mut agent = test_agent();
        agent.spec.mcp_servers = vec![
            McpServerRef::builder().name("github-mcp".to_string()).build(),
            McpServerRef::builder().name("fs-mcp".to_string()).port(9090).build(),
        ];

        let env_vars = build_test_env_vars(&agent);
        let var = env_vars.iter().find(|e| e.name == "MCP_SERVER_URLS").unwrap();
        assert_eq!(
            var.value,
            Some(
                "http://github-mcp.forgemaster-system.svc.cluster.local:3000/mcp,\
                 http://fs-mcp.forgemaster-system.svc.cluster.local:9090/mcp"
                    .to_string()
            )
        );
    }

    #[test]
    fn build_mcp_server_urls_joins_with_comma() {
        let servers = vec![
            McpServerRef::builder().name("a-mcp".to_string()).build(),
            McpServerRef::builder().name("b-mcp".to_string()).build(),
        ];
        let urls = build_mcp_server_urls(&servers, "ns");
        assert_eq!(
            urls,
            "http://a-mcp.ns.svc.cluster.local:3000/mcp,http://b-mcp.ns.svc.cluster.local:3000/mcp"
        );
    }

    #[test]
    fn env_contains_a2a_port() {
        let env_vars = build_test_env_vars(&test_agent());
        let var = env_vars.iter().find(|e| e.name == "A2A_PORT").expect("A2A_PORT");
        assert_eq!(var.value, Some("9090".to_string()));
    }

    #[test]
    fn env_contains_agent_type() {
        let env_vars = build_test_env_vars(&test_agent());
        let var = env_vars.iter().find(|e| e.name == "AGENT_TYPE").expect("AGENT_TYPE");
        assert_eq!(var.value, Some("orchestrator".to_string()));
    }
}
