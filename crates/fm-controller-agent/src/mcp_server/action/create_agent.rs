//! Strategy for the `create_agent` MCP tool.

use std::collections::BTreeMap;

use async_trait::async_trait;
use kube::Client;
use kube::ResourceExt;
use kube::api::{Api, PostParams};
use rmcp::model::*;
use tracing::info;

use crate::controller::McpServerRefs;
use crate::crd::{Agent, AgentCrd, McpServerRef, ModelConfig};
use crate::mcp_server::CreateAgentParams;

use super::McpAction;

/// Creates a new Agent CR in Kubernetes.
pub struct CreateAgentAction {
    client: Client,
    default_model: String,
    default_max_tokens: i32,
    default_mcp_servers: McpServerRefs,
}

impl CreateAgentAction {
    /// Creates a new action with the given Kubernetes client, default model, max tokens, and default MCP servers.
    pub fn new(client: Client, default_model: String, default_max_tokens: i32, default_mcp_servers: McpServerRefs) -> Self {
        Self {
            client,
            default_model,
            default_max_tokens,
            default_mcp_servers,
        }
    }
}

#[async_trait]
impl McpAction for CreateAgentAction {
    type Params = CreateAgentParams;

    async fn execute(&self, params: Self::Params) -> CallToolResult {
        info!(
            "🔧 MCP: create_agent name={} namespace={} type={} model={} mcp_servers={:?}\n📝 task_prompt:\n{}",
            params.name, params.namespace, params.agent_type,
            self.default_model, params.mcp_servers, params.task_prompt
        );

        let mcp_servers = merge_mcp_servers(
            &self.default_mcp_servers,
            params.mcp_servers.as_deref(),
        );

        let labels = build_agent_labels(&params.namespace, &params.agent_type);

        let mut agent = Agent::new(
            &params.name,
            AgentCrd {
                agent_type: params.agent_type,
                model: ModelConfig::builder()
                    .name(self.default_model.clone())
                    .max_tokens(self.default_max_tokens)
                    .build(),
                task_prompt: params.task_prompt,
                mcp_servers,
                resources: None,
            },
        );
        agent.metadata.namespace = Some(params.namespace.clone());
        agent.metadata.labels = Some(labels);

        let api: Api<Agent> = Api::namespaced(self.client.clone(), &params.namespace);

        match api.create(&PostParams::default(), &agent).await {
            Ok(created) => {
                let response = serde_json::json!({
                    "name": created.name_any(),
                    "namespace": params.namespace,
                    "status": "created"
                });
                CallToolResult::success(vec![Content::text(response.to_string())])
            }
            Err(kube::Error::Api(ref api_err)) if api_err.code == 409 => {
                info!(
                    "🔁 Agent '{}' already exists in namespace '{}', reusing",
                    params.name, params.namespace
                );
                let response = serde_json::json!({
                    "name": params.name,
                    "namespace": params.namespace,
                    "status": "already_exists"
                });
                CallToolResult::success(vec![Content::text(response.to_string())])
            }
            Err(err) => CallToolResult::error(vec![Content::text(format!(
                "Failed to create agent: {err}"
            ))]),
        }
    }
}

/// Builds standard labels for an Agent CR.
fn build_agent_labels(task_name: &str, agent_type: &str) -> BTreeMap<String, String> {
    let mut labels = BTreeMap::new();
    labels.insert("forgemaster.io/task".to_string(), task_name.to_string());
    labels.insert("forgemaster.io/type".to_string(), agent_type.to_string());
    labels.insert(
        "app.kubernetes.io/managed-by".to_string(),
        "fm-controller-agent".to_string(),
    );
    labels
}

/// Merges default MCP servers with optional custom servers.
///
/// Defaults are always included. Custom servers are appended, skipping
/// any that share a name with a default server (no duplicates).
fn merge_mcp_servers(defaults: &[McpServerRef], custom: Option<&str>) -> Vec<McpServerRef> {
    let mut merged = defaults.to_vec();
    if let Some(input) = custom {
        let custom_servers = parse_mcp_servers(input);
        for server in custom_servers {
            if !merged.iter().any(|s| s.name == server.name) {
                merged.push(server);
            }
        }
    }
    merged
}

/// Parses comma-separated `name:port` entries into `McpServerRef` list.
fn parse_mcp_servers(input: &str) -> Vec<McpServerRef> {
    input
        .split(',')
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(|entry| match entry.rsplit_once(':') {
            Some((name, port_str)) => {
                let port = port_str.parse().unwrap_or(3000);
                McpServerRef::builder()
                    .name(name.to_string())
                    .port(port)
                    .build()
            }
            None => McpServerRef::builder()
                .name(entry.to_string())
                .build(),
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_mcp_servers_single() {
        let servers = parse_mcp_servers("github-mcp:3000");
        assert_eq!(servers.len(), 1);
        assert_eq!(servers[0].name, "github-mcp");
        assert_eq!(servers[0].port, 3000);
    }

    #[test]
    fn parse_mcp_servers_multiple() {
        let servers = parse_mcp_servers("a-mcp:3000,b-mcp:4000");
        assert_eq!(servers.len(), 2);
        assert_eq!(servers[0].name, "a-mcp");
        assert_eq!(servers[1].name, "b-mcp");
        assert_eq!(servers[1].port, 4000);
    }

    #[test]
    fn parse_mcp_servers_no_port() {
        let servers = parse_mcp_servers("github-mcp");
        assert_eq!(servers.len(), 1);
        assert_eq!(servers[0].name, "github-mcp");
        assert_eq!(servers[0].port, 3000);
    }

    #[test]
    fn parse_mcp_servers_empty() {
        let servers = parse_mcp_servers("");
        assert!(servers.is_empty());
    }

    #[test]
    fn merge_no_custom_returns_defaults() {
        let defaults = vec![
            McpServerRef::builder().name("controller-mcp".to_string()).build(),
        ];
        let merged = merge_mcp_servers(&defaults, None);
        assert_eq!(merged.len(), 1);
        assert_eq!(merged[0].name, "controller-mcp");
    }

    #[test]
    fn merge_appends_custom_servers() {
        let defaults = vec![
            McpServerRef::builder().name("controller-mcp".to_string()).build(),
        ];
        let merged = merge_mcp_servers(&defaults, Some("fs-mcp,devtools-mcp"));
        assert_eq!(merged.len(), 3);
        assert_eq!(merged[0].name, "controller-mcp");
        assert_eq!(merged[1].name, "fs-mcp");
        assert_eq!(merged[2].name, "devtools-mcp");
    }

    #[test]
    fn merge_skips_duplicates() {
        let defaults = vec![
            McpServerRef::builder().name("controller-mcp".to_string()).build(),
        ];
        let merged = merge_mcp_servers(&defaults, Some("controller-mcp,fs-mcp"));
        assert_eq!(merged.len(), 2);
        assert_eq!(merged[0].name, "controller-mcp");
        assert_eq!(merged[1].name, "fs-mcp");
    }

    #[test]
    fn merge_empty_defaults_uses_custom_only() {
        let merged = merge_mcp_servers(&[], Some("fs-mcp"));
        assert_eq!(merged.len(), 1);
        assert_eq!(merged[0].name, "fs-mcp");
    }

    #[test]
    fn labels_contain_task_name() {
        let labels = build_agent_labels("task-abc", "code-generator");
        assert_eq!(labels.get("forgemaster.io/task").unwrap(), "task-abc");
    }

    #[test]
    fn labels_contain_agent_type() {
        let labels = build_agent_labels("task-abc", "code-generator");
        assert_eq!(labels.get("forgemaster.io/type").unwrap(), "code-generator");
    }

    #[test]
    fn labels_contain_managed_by() {
        let labels = build_agent_labels("task-abc", "reviewer");
        assert_eq!(
            labels.get("app.kubernetes.io/managed-by").unwrap(),
            "fm-controller-agent"
        );
    }
}
