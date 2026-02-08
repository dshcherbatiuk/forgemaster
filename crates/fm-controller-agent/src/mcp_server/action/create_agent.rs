//! Strategy for the `create_agent` MCP tool.

use std::collections::BTreeMap;

use async_trait::async_trait;
use kube::Client;
use kube::ResourceExt;
use kube::api::{Api, PostParams};
use rmcp::model::*;
use tracing::info;

use crate::crd::{Agent, AgentCrd, McpServerRef, ModelConfig};
use crate::mcp_server::CreateAgentParams;

use super::McpAction;

/// Creates a new Agent CR in Kubernetes.
pub struct CreateAgentAction {
    client: Client,
    default_model: String,
}

impl CreateAgentAction {
    /// Creates a new action with the given Kubernetes client and default model fallback.
    pub fn new(client: Client, default_model: String) -> Self {
        Self {
            client,
            default_model,
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

        let mcp_servers = params
            .mcp_servers
            .map(|s| parse_mcp_servers(&s))
            .unwrap_or_default();

        let labels = build_agent_labels(&params.namespace, &params.agent_type);

        let mut agent = Agent::new(
            &params.name,
            AgentCrd {
                agent_type: params.agent_type,
                model: ModelConfig::builder().name(self.default_model.clone()).build(),
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
