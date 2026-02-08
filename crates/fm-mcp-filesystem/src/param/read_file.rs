//! Parameters for the `read_file` MCP tool.

use serde::Deserialize;

/// Parameters for reading a file.
#[derive(Debug, Deserialize, rmcp::schemars::JsonSchema)]
#[schemars(crate = "rmcp::schemars")]
pub struct ReadFileParams {
    /// Absolute path to the file to read (must be under workspace root).
    pub path: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn deserialize_read_file_params() {
        let json = r#"{"path": "/workspace/task-1/src/main.rs"}"#;
        let params: ReadFileParams = serde_json::from_str(json).expect("parse");
        assert_eq!(params.path, "/workspace/task-1/src/main.rs");
    }

    #[test]
    fn json_schema_generates() {
        let schema = rmcp::schemars::schema_for!(ReadFileParams);
        let json = serde_json::to_value(&schema).expect("json");
        assert!(json.get("properties").is_some());
    }
}
