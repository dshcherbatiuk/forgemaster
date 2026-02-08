//! Parameters for the `write_file` MCP tool.

use serde::Deserialize;

/// Parameters for writing a file.
#[derive(Debug, Deserialize, rmcp::schemars::JsonSchema)]
#[schemars(crate = "rmcp::schemars")]
pub struct WriteFileParams {
    /// Absolute path to the file to write (must be under workspace root).
    pub path: String,
    /// Content to write to the file. Overwrites existing content.
    pub content: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn deserialize_write_file_params() {
        let json = r#"{"path": "/workspace/task-1/hello.txt", "content": "Hello, world!"}"#;
        let params: WriteFileParams = serde_json::from_str(json).expect("parse");
        assert_eq!(params.path, "/workspace/task-1/hello.txt");
        assert_eq!(params.content, "Hello, world!");
    }

    #[test]
    fn json_schema_generates() {
        let schema = rmcp::schemars::schema_for!(WriteFileParams);
        let json = serde_json::to_value(&schema).expect("json");
        assert!(json.get("properties").is_some());
    }
}
