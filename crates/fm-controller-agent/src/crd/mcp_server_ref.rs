//! MCP server reference.

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use typed_builder::TypedBuilder;

/// Default port for MCP servers inside the cluster.
const DEFAULT_PORT: u16 = 3000;

fn default_port() -> u16 {
    DEFAULT_PORT
}

/// Reference to an MCPServer CR that an agent can use.
///
/// Each reference carries a service name and port. The controller builds
/// full K8s DNS URLs at pod creation time using these values.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema, TypedBuilder)]
pub struct McpServerRef {
    /// K8s service name of the MCP server.
    pub name: String,

    /// Port the MCP server listens on.
    #[serde(default = "default_port")]
    #[builder(default = DEFAULT_PORT)]
    pub port: u16,
}

impl McpServerRef {
    /// Builds a full MCP server URL for the given namespace.
    ///
    /// Uses K8s DNS: `http://<name>.<namespace>.svc.cluster.local:<port>/mcp`.
    /// The `/mcp` path matches the rmcp Streamable HTTP transport convention.
    pub fn url(&self, namespace: &str) -> String {
        format!(
            "http://{}.{}.svc.cluster.local:{}/mcp",
            self.name, namespace, self.port
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn build_with_default_port() {
        let server = McpServerRef::builder()
            .name("github-mcp".to_string())
            .build();
        assert_eq!(server.name, "github-mcp");
        assert_eq!(server.port, DEFAULT_PORT);
    }

    #[test]
    fn build_with_custom_port() {
        let server = McpServerRef::builder()
            .name("github-mcp".to_string())
            .port(9090)
            .build();
        assert_eq!(server.name, "github-mcp");
        assert_eq!(server.port, 9090);
    }

    #[test]
    fn url_builds_k8s_dns_with_mcp_path() {
        let server = McpServerRef::builder()
            .name("github-mcp".to_string())
            .build();
        assert_eq!(
            server.url("task-abc"),
            "http://github-mcp.task-abc.svc.cluster.local:3000/mcp"
        );
    }

    #[test]
    fn url_with_custom_port() {
        let server = McpServerRef::builder()
            .name("fs-mcp".to_string())
            .port(9090)
            .build();
        assert_eq!(
            server.url("task-xyz"),
            "http://fs-mcp.task-xyz.svc.cluster.local:9090/mcp"
        );
    }

    #[test]
    fn serialization() {
        let server = McpServerRef::builder()
            .name("postgres-mcp".to_string())
            .build();

        let json = serde_json::to_value(&server).unwrap_or_default();
        assert_eq!(json["name"], "postgres-mcp");
        assert_eq!(json["port"], DEFAULT_PORT);
    }

    #[test]
    fn deserialization_with_port() {
        let json = r#"{"name": "filesystem-mcp", "port": 4000}"#;
        let server: McpServerRef = serde_json::from_str(json).unwrap();
        assert_eq!(server.name, "filesystem-mcp");
        assert_eq!(server.port, 4000);
    }

    #[test]
    fn deserialization_without_port_uses_default() {
        let json = r#"{"name": "filesystem-mcp"}"#;
        let server: McpServerRef = serde_json::from_str(json).unwrap();
        assert_eq!(server.name, "filesystem-mcp");
        assert_eq!(server.port, DEFAULT_PORT);
    }
}
