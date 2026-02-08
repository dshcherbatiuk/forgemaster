//! Strategy for the `get_agent_status` MCP tool.

use async_trait::async_trait;
use kube::Client;
use kube::ResourceExt;
use kube::api::Api;
use rmcp::model::*;
use tracing::info;

use crate::crd::Agent;
use crate::mcp_server::GetAgentParams;

use super::McpAction;

/// Gets detailed status of a specific Agent CR.
pub struct GetAgentAction {
    client: Client,
}

impl GetAgentAction {
    /// Creates a new action with the given Kubernetes client.
    pub fn new(client: Client) -> Self {
        Self { client }
    }
}

#[async_trait]
impl McpAction for GetAgentAction {
    type Params = GetAgentParams;

    async fn execute(&self, params: Self::Params) -> CallToolResult {
        info!(
            "🔍 MCP: get_agent_status name={} namespace={}",
            params.name, params.namespace
        );

        let api: Api<Agent> = Api::namespaced(self.client.clone(), &params.namespace);

        match api.get(&params.name).await {
            Ok(agent) => {
                let status = agent.status.as_ref();

                let response = serde_json::json!({
                    "name": agent.name_any(),
                    "namespace": agent.namespace().unwrap_or_default(),
                    "type": agent.spec.agent_type,
                    "phase": status.map(|s| s.phase.to_string()).unwrap_or_else(|| "Unknown".to_string()),
                    "tokens_used": status.map(|s| s.tokens_used).unwrap_or(0),
                    "iterations_completed": status.map(|s| s.iterations_completed).unwrap_or(0),
                    "start_time": status.and_then(|s| s.start_time.as_deref()),
                    "completion_time": status.and_then(|s| s.completion_time.as_deref()),
                    "pod": status.and_then(|s| s.pod_ref.as_ref().map(|p| &p.name)),
                    "output": status.and_then(|s| s.output.as_ref().map(|o| &o.config_map_ref)),
                });
                CallToolResult::success(vec![Content::text(response.to_string())])
            }
            Err(err) => CallToolResult::error(vec![Content::text(format!(
                "Failed to get agent: {err}"
            ))]),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn get_agent_action_creates() {
        // K8s client can't be constructed without a cluster,
        // so we verify the struct exists and fields are correct.
        assert_eq!(std::mem::size_of::<GetAgentAction>(), std::mem::size_of::<Client>());
    }
}
