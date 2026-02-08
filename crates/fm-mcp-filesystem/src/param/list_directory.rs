//! Parameters for the `list_directory` MCP tool.

use serde::Deserialize;

/// Parameters for listing directory entries.
#[derive(Debug, Deserialize, rmcp::schemars::JsonSchema)]
#[schemars(crate = "rmcp::schemars")]
pub struct ListDirectoryParams {
    /// Absolute path to the directory to list (must be under workspace root).
    pub path: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn deserialize_list_directory_params() {
        let json = r#"{"path": "/workspace/task-1"}"#;
        let params: ListDirectoryParams = serde_json::from_str(json).expect("parse");
        assert_eq!(params.path, "/workspace/task-1");
    }

    #[test]
    fn json_schema_generates() {
        let schema = rmcp::schemars::schema_for!(ListDirectoryParams);
        let json = serde_json::to_value(&schema).expect("json");
        assert!(json.get("properties").is_some());
    }
}
