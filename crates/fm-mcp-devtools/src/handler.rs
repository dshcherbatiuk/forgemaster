//! MCP handler exposing devtools (`docker_build`, `helm_install`, `helm_status`).

use rmcp::ServerHandler;
use rmcp::handler::server::router::tool::ToolRouter;
use rmcp::handler::server::wrapper::Parameters;
use rmcp::model::{
    CallToolResult, Implementation, ProtocolVersion, ServerCapabilities, ServerInfo,
};
use rmcp::{ErrorData, tool, tool_handler, tool_router};

use crate::action;
use crate::param::{DockerBuildParams, HelmInstallParams, HelmStatusParams};

/// MCP handler for devtools operations.
#[derive(Clone)]
pub struct DevtoolsHandler {
    tool_router: ToolRouter<Self>,
}

#[tool_router]
impl DevtoolsHandler {
    /// Creates a new handler.
    #[must_use]
    pub fn new() -> Self {
        Self {
            tool_router: Self::tool_router(),
        }
    }

    #[tool(
        description = "Build a Docker image from a Dockerfile. Runs `docker build` with the given context and Dockerfile path. Returns build output including any errors."
    )]
    async fn docker_build(
        &self,
        Parameters(params): Parameters<DockerBuildParams>,
    ) -> Result<CallToolResult, ErrorData> {
        Ok(action::docker_build::execute(params).await)
    }

    #[tool(
        description = "Deploy a Helm chart to Kubernetes. Runs `helm upgrade --install` with the given release name, chart path, and namespace. Returns deployment output."
    )]
    async fn helm_install(
        &self,
        Parameters(params): Parameters<HelmInstallParams>,
    ) -> Result<CallToolResult, ErrorData> {
        Ok(action::helm_install::execute(params).await)
    }

    #[tool(description = "Check the status of a Helm release in Kubernetes.")]
    async fn helm_status(
        &self,
        Parameters(params): Parameters<HelmStatusParams>,
    ) -> Result<CallToolResult, ErrorData> {
        Ok(action::helm_status::execute(params).await)
    }
}

impl Default for DevtoolsHandler {
    fn default() -> Self {
        Self::new()
    }
}

#[tool_handler]
impl ServerHandler for DevtoolsHandler {
    fn get_info(&self) -> ServerInfo {
        ServerInfo {
            protocol_version: ProtocolVersion::V_2025_03_26,
            capabilities: ServerCapabilities::builder().enable_tools().build(),
            server_info: Implementation {
                name: "fm-mcp-devtools".to_string(),
                version: env!("CARGO_PKG_VERSION").to_string(),
                ..Default::default()
            },
            instructions: Some(
                "DevOps tools: docker_build, helm_install, helm_status".to_string(),
            ),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn handler_creation() {
        let handler = DevtoolsHandler::new();
        let info = handler.get_info();
        assert_eq!(info.server_info.name, "fm-mcp-devtools");
    }

    #[test]
    fn server_capabilities_has_tools() {
        let handler = DevtoolsHandler::new();
        let info = handler.get_info();
        assert!(info.capabilities.tools.is_some());
    }

    #[test]
    fn instructions_mention_tools() {
        let handler = DevtoolsHandler::new();
        let info = handler.get_info();
        let instructions = info.instructions.expect("should have instructions");
        assert!(instructions.contains("docker_build"));
        assert!(instructions.contains("helm_install"));
        assert!(instructions.contains("helm_status"));
    }
}
