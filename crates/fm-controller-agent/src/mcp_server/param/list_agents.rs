//! Parameters for the `list_agents` MCP tool.

use serde::Deserialize;

/// Input parameters for listing Agent CRs.
#[derive(Debug, Deserialize, rmcp::schemars::JsonSchema)]
#[schemars(crate = "rmcp::schemars")]
pub struct ListAgentsParams {
    /// Kubernetes namespace to list agents in.
    /// If omitted, lists agents across all namespaces.
    #[schemars(description = "Namespace to list agents in (omit for all namespaces)")]
    pub namespace: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn deserialize_with_namespace() {
        let json = r#"{"namespace": "task-abc"}"#;
        let params: ListAgentsParams = serde_json::from_str(json).expect("deserialize");
        assert_eq!(params.namespace.as_deref(), Some("task-abc"));
    }

    #[test]
    fn deserialize_without_namespace() {
        let json = r#"{}"#;
        let params: ListAgentsParams = serde_json::from_str(json).expect("deserialize");
        assert!(params.namespace.is_none());
    }

    #[test]
    fn json_schema_generates() {
        let schema = rmcp::schemars::schema_for!(ListAgentsParams);
        let schema_json = serde_json::to_value(&schema).expect("schema to json");
        let properties = schema_json.get("properties").expect("has properties");
        assert!(properties.get("namespace").is_some());
    }
}
