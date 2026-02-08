//! Parameters for the `create_agent` MCP tool.

use serde::Deserialize;

/// Input parameters for creating a new Agent CR.
#[derive(Debug, Deserialize, rmcp::schemars::JsonSchema)]
#[schemars(crate = "rmcp::schemars")]
pub struct CreateAgentParams {
    /// Agent name (e.g., "code-generator-abc123").
    #[schemars(description = "Name for the new agent")]
    pub name: String,

    /// Kubernetes namespace where the agent will be created.
    #[schemars(description = "Kubernetes namespace for the agent")]
    pub namespace: String,

    /// Agent type (e.g., "orchestrator", "code-generator", "architect").
    #[schemars(description = "Type of the agent (e.g. code-generator, architect, reviewer)")]
    pub agent_type: String,

    /// Full instruction prompt for the agent.
    #[schemars(description = "Task prompt describing what the agent should do")]
    pub task_prompt: String,

    /// Optional MCP server names (comma-separated "name:port").
    #[schemars(description = "Comma-separated MCP servers (name:port format, port defaults to 3000)")]
    pub mcp_servers: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn deserialize_full_params() {
        let json = r#"{
            "name": "code-gen-1",
            "namespace": "task-abc",
            "agent_type": "code-generator",
            "task_prompt": "Write a hello world",
            "mcp_servers": "github-mcp:3000"
        }"#;

        let params: CreateAgentParams = serde_json::from_str(json).expect("deserialize");
        assert_eq!(params.name, "code-gen-1");
        assert_eq!(params.namespace, "task-abc");
        assert_eq!(params.agent_type, "code-generator");
        assert_eq!(params.task_prompt, "Write a hello world");
        assert_eq!(params.mcp_servers.as_deref(), Some("github-mcp:3000"));
    }

    #[test]
    fn deserialize_without_mcp_servers() {
        let json = r#"{
            "name": "reviewer-1",
            "namespace": "task-xyz",
            "agent_type": "reviewer",
            "task_prompt": "Review the code"
        }"#;

        let params: CreateAgentParams = serde_json::from_str(json).expect("deserialize");
        assert_eq!(params.name, "reviewer-1");
        assert!(params.mcp_servers.is_none());
    }

    #[test]
    fn missing_required_field_fails() {
        let json = r#"{"name": "test"}"#;
        let result = serde_json::from_str::<CreateAgentParams>(json);
        assert!(result.is_err());
    }

    #[test]
    fn json_schema_generates() {
        let schema = rmcp::schemars::schema_for!(CreateAgentParams);
        let schema_json = serde_json::to_value(&schema).expect("schema to json");
        let properties = schema_json.get("properties").expect("has properties");
        assert!(properties.get("name").is_some());
        assert!(properties.get("namespace").is_some());
        assert!(properties.get("agent_type").is_some());
        assert!(properties.get("task_prompt").is_some());
        assert!(properties.get("mcp_servers").is_some());
        assert!(properties.get("model_name").is_none());
    }
}
