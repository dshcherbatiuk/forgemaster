//! Strategy for the `list_agents` MCP tool.

use async_trait::async_trait;
use kube::Client;
use kube::ResourceExt;
use kube::api::{Api, ListParams};
use rmcp::model::*;
use tracing::info;

use crate::crd::Agent;
use crate::mcp_server::ListAgentsParams;

use super::McpAction;

/// Lists Agent CRs, optionally filtered by namespace.
pub struct ListAgentsAction {
    client: Client,
}

impl ListAgentsAction {
    /// Creates a new action with the given Kubernetes client.
    pub fn new(client: Client) -> Self {
        Self { client }
    }
}

#[async_trait]
impl McpAction for ListAgentsAction {
    type Params = ListAgentsParams;

    async fn execute(&self, params: Self::Params) -> CallToolResult {
        info!("📋 MCP: list_agents namespace={:?}", params.namespace);

        let agents = match &params.namespace {
            Some(ns) => {
                let api: Api<Agent> = Api::namespaced(self.client.clone(), ns);
                api.list(&ListParams::default()).await
            }
            None => {
                let api: Api<Agent> = Api::all(self.client.clone());
                api.list(&ListParams::default()).await
            }
        };

        match agents {
            Ok(agent_list) => {
                let summaries: Vec<serde_json::Value> = agent_list
                    .items
                    .iter()
                    .map(agent_summary)
                    .collect();

                let response = serde_json::json!({
                    "count": summaries.len(),
                    "agents": summaries
                });
                CallToolResult::success(vec![Content::text(response.to_string())])
            }
            Err(err) => CallToolResult::error(vec![Content::text(format!(
                "Failed to list agents: {err}"
            ))]),
        }
    }
}

/// Builds a summary JSON object for a single agent.
fn agent_summary(agent: &Agent) -> serde_json::Value {
    let phase = agent
        .status
        .as_ref()
        .map(|s| s.phase.to_string())
        .unwrap_or_else(|| "Unknown".to_string());

    serde_json::json!({
        "name": agent.name_any(),
        "namespace": agent.namespace().unwrap_or_default(),
        "type": agent.spec.agent_type,
        "phase": phase,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::crd::{AgentCrd, AgentPhase, AgentStatus, ModelConfig};
    use kube::core::ObjectMeta;

    fn test_agent(name: &str, namespace: &str, agent_type: &str, phase: AgentPhase) -> Agent {
        Agent {
            metadata: ObjectMeta {
                name: Some(name.to_string()),
                namespace: Some(namespace.to_string()),
                ..Default::default()
            },
            spec: AgentCrd {
                agent_type: agent_type.to_string(),
                model: ModelConfig::builder()
                    .name("claude-sonnet-4-20250514".to_string())
                    .build(),
                task_prompt: "test".to_string(),
                mcp_servers: vec![],
                resources: None,
            },
            status: Some(AgentStatus::builder().phase(phase).build()),
        }
    }

    #[test]
    fn agent_summary_includes_all_fields() {
        let agent = test_agent("code-gen-1", "task-abc", "code-generator", AgentPhase::Running);
        let summary = agent_summary(&agent);

        assert_eq!(summary["name"], "code-gen-1");
        assert_eq!(summary["namespace"], "task-abc");
        assert_eq!(summary["type"], "code-generator");
        assert_eq!(summary["phase"], "Running");
    }

    #[test]
    fn agent_summary_without_status() {
        let mut agent = test_agent("test", "ns", "orchestrator", AgentPhase::Pending);
        agent.status = None;

        let summary = agent_summary(&agent);
        assert_eq!(summary["phase"], "Unknown");
    }
}
