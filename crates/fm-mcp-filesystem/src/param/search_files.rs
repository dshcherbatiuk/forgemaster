//! Parameters for the `search_files` MCP tool.

use serde::Deserialize;

/// Parameters for searching file contents by regex.
#[derive(Debug, Deserialize, rmcp::schemars::JsonSchema)]
#[schemars(crate = "rmcp::schemars")]
pub struct SearchFilesParams {
    /// Absolute path to the directory to search in (must be under workspace root).
    pub path: String,
    /// Regex pattern to search for in file contents.
    pub pattern: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn deserialize_search_files_params() {
        let json = r#"{"path": "/workspace/task-1", "pattern": "fn main"}"#;
        let params: SearchFilesParams = serde_json::from_str(json).expect("parse");
        assert_eq!(params.path, "/workspace/task-1");
        assert_eq!(params.pattern, "fn main");
    }

    #[test]
    fn json_schema_generates() {
        let schema = rmcp::schemars::schema_for!(SearchFilesParams);
        let json = serde_json::to_value(&schema).expect("json");
        assert!(json.get("properties").is_some());
    }
}
