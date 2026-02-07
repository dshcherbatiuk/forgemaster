//! MCP server reference.

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use typed_builder::TypedBuilder;

/// Reference to an MCPServer CR that an agent can use.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema, TypedBuilder)]
pub struct McpServerRef {
    /// MCPServer CR name.
    pub name: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn build() {
        let server = McpServerRef::builder()
            .name("github-mcp".to_string())
            .build();
        assert_eq!(server.name, "github-mcp");
    }

    #[test]
    fn serialization() {
        let server = McpServerRef::builder()
            .name("postgres-mcp".to_string())
            .build();

        let json = serde_json::to_value(&server).unwrap_or_default();
        assert_eq!(json["name"], "postgres-mcp");
    }

    #[test]
    fn deserialization() {
        let json = r#"{"name": "filesystem-mcp"}"#;
        let server: McpServerRef = serde_json::from_str(json)
            .unwrap_or_else(|_| McpServerRef::builder().name("fallback".to_string()).build());
        assert_eq!(server.name, "filesystem-mcp");
    }
}
