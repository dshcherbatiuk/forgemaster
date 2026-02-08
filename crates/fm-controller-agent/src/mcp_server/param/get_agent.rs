//! Parameters for the `get_agent_status` MCP tool.

use serde::Deserialize;

/// Input parameters for getting an Agent's status.
#[derive(Debug, Deserialize, rmcp::schemars::JsonSchema)]
#[schemars(crate = "rmcp::schemars")]
pub struct GetAgentParams {
    /// Agent name.
    #[schemars(description = "Name of the agent")]
    pub name: String,

    /// Kubernetes namespace the agent lives in.
    #[schemars(description = "Namespace of the agent")]
    pub namespace: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn deserialize_params() {
        let json = r#"{"name": "orchestrator-abc", "namespace": "task-abc"}"#;
        let params: GetAgentParams = serde_json::from_str(json).expect("deserialize");
        assert_eq!(params.name, "orchestrator-abc");
        assert_eq!(params.namespace, "task-abc");
    }

    #[test]
    fn missing_name_fails() {
        let json = r#"{"namespace": "task-abc"}"#;
        let result = serde_json::from_str::<GetAgentParams>(json);
        assert!(result.is_err());
    }

    #[test]
    fn missing_namespace_fails() {
        let json = r#"{"name": "test"}"#;
        let result = serde_json::from_str::<GetAgentParams>(json);
        assert!(result.is_err());
    }

    #[test]
    fn json_schema_generates() {
        let schema = rmcp::schemars::schema_for!(GetAgentParams);
        let schema_json = serde_json::to_value(&schema).expect("schema to json");
        let properties = schema_json.get("properties").expect("has properties");
        assert!(properties.get("name").is_some());
        assert!(properties.get("namespace").is_some());
    }
}
