//! Parameters for the `directory_tree` MCP tool.

use serde::Deserialize;

/// Parameters for generating a directory tree.
#[derive(Debug, Deserialize, rmcp::schemars::JsonSchema)]
#[schemars(crate = "rmcp::schemars")]
pub struct DirectoryTreeParams {
    /// Absolute path to the root directory (must be under workspace root).
    pub path: String,
    /// Maximum depth to recurse (default: 5).
    #[serde(default = "default_max_depth")]
    pub max_depth: u32,
}

const fn default_max_depth() -> u32 {
    5
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn deserialize_with_defaults() {
        let json = r#"{"path": "/workspace/task-1"}"#;
        let params: DirectoryTreeParams = serde_json::from_str(json).expect("parse");
        assert_eq!(params.path, "/workspace/task-1");
        assert_eq!(params.max_depth, 5);
    }

    #[test]
    fn deserialize_with_custom_depth() {
        let json = r#"{"path": "/workspace/task-1", "max_depth": 2}"#;
        let params: DirectoryTreeParams = serde_json::from_str(json).expect("parse");
        assert_eq!(params.max_depth, 2);
    }

    #[test]
    fn json_schema_generates() {
        let schema = rmcp::schemars::schema_for!(DirectoryTreeParams);
        let json = serde_json::to_value(&schema).expect("json");
        assert!(json.get("properties").is_some());
    }
}
