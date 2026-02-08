//! Parameters for the `create_directory` MCP tool.

use serde::Deserialize;

/// Parameters for creating a directory.
#[derive(Debug, Deserialize, rmcp::schemars::JsonSchema)]
#[schemars(crate = "rmcp::schemars")]
pub struct CreateDirectoryParams {
    /// Absolute path to the directory to create (must be under workspace root).
    /// Creates parent directories as needed.
    pub path: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn deserialize_create_directory_params() {
        let json = r#"{"path": "/workspace/task-1/src/modules"}"#;
        let params: CreateDirectoryParams = serde_json::from_str(json).expect("parse");
        assert_eq!(params.path, "/workspace/task-1/src/modules");
    }

    #[test]
    fn json_schema_generates() {
        let schema = rmcp::schemars::schema_for!(CreateDirectoryParams);
        let json = serde_json::to_value(&schema).expect("json");
        assert!(json.get("properties").is_some());
    }
}
