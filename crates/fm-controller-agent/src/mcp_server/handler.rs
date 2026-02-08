//! MCP handler exposing agent lifecycle tools.

use kube::Client;
use rmcp::ServerHandler;
use rmcp::handler::server::router::tool::ToolRouter;
use rmcp::handler::server::wrapper::Parameters;
use rmcp::model::*;
use rmcp::{ErrorData, tool, tool_handler, tool_router};

use crate::controller::McpServerRefs;

use super::action::{CreateAgentAction, GetAgentAction, ListAgentsAction, McpAction};
use super::{CreateAgentParams, GetAgentParams, ListAgentsParams};

/// MCP handler for agent lifecycle management.
///
/// Provides `create_agent`, `list_agents`, and `get_agent_status` tools
/// via the MCP protocol. Each tool delegates to its own action strategy.
#[derive(Clone)]
pub struct AgentMcpHandler {
    client: Client,
    default_model: String,
    default_mcp_servers: McpServerRefs,
    tool_router: ToolRouter<Self>,
}

#[tool_router]
impl AgentMcpHandler {
    /// Creates a new handler with the given Kubernetes client, default model, and default MCP servers.
    pub fn new(client: Client, default_model: String, default_mcp_servers: McpServerRefs) -> Self {
        Self {
            client,
            default_model,
            default_mcp_servers,
            tool_router: Self::tool_router(),
        }
    }

    #[tool(description = "Create a new Agent CR in Kubernetes. Use the task namespace (e.g. task-abc123) as the namespace. Returns the created agent name and namespace.")]
    async fn create_agent(
        &self,
        Parameters(params): Parameters<CreateAgentParams>,
    ) -> Result<CallToolResult, ErrorData> {
        let action = CreateAgentAction::new(self.client.clone(), self.default_model.clone(), self.default_mcp_servers.clone());
        Ok(action.execute(params).await)
    }

    #[tool(description = "List Agent CRs. Optionally filter by namespace (omit for all namespaces).")]
    async fn list_agents(
        &self,
        Parameters(params): Parameters<ListAgentsParams>,
    ) -> Result<CallToolResult, ErrorData> {
        let action = ListAgentsAction::new(self.client.clone());
        Ok(action.execute(params).await)
    }

    #[tool(description = "Get detailed status of a specific Agent CR by name and namespace.")]
    async fn get_agent_status(
        &self,
        Parameters(params): Parameters<GetAgentParams>,
    ) -> Result<CallToolResult, ErrorData> {
        let action = GetAgentAction::new(self.client.clone());
        Ok(action.execute(params).await)
    }
}

#[tool_handler]
impl ServerHandler for AgentMcpHandler {
    fn get_info(&self) -> ServerInfo {
        ServerInfo {
            protocol_version: ProtocolVersion::V_2025_03_26,
            capabilities: ServerCapabilities::builder().enable_tools().build(),
            server_info: Implementation {
                name: "fm-controller-agent".to_string(),
                version: env!("CARGO_PKG_VERSION").to_string(),
                ..Default::default()
            },
            instructions: Some(
                "Agent lifecycle management. Tools: create_agent, list_agents, get_agent_status"
                    .to_string(),
            ),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn server_info_name() {
        let info = ServerInfo {
            protocol_version: ProtocolVersion::V_2025_03_26,
            capabilities: ServerCapabilities::builder().enable_tools().build(),
            server_info: Implementation {
                name: "fm-controller-agent".to_string(),
                version: "0.1.0".to_string(),
                ..Default::default()
            },
            instructions: Some("test".to_string()),
        };
        assert_eq!(info.server_info.name, "fm-controller-agent");
    }

    #[test]
    fn server_capabilities_has_tools() {
        let capabilities = ServerCapabilities::builder().enable_tools().build();
        assert!(capabilities.tools.is_some());
    }
}
