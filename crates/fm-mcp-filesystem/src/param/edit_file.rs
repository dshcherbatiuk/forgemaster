//! Parameters for the `edit_file` MCP tool.

use serde::Deserialize;

/// Parameters for editing a file by replacing text.
#[derive(Debug, Deserialize, rmcp::schemars::JsonSchema)]
#[schemars(crate = "rmcp::schemars")]
pub struct EditFileParams {
    /// Absolute path to the file to edit (must be under workspace root).
    pub path: String,
    /// The exact text to find and replace.
    pub old_text: String,
    /// The replacement text.
    pub new_text: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn deserialize_edit_file_params() {
        let json = r#"{
            "path": "/workspace/task-1/src/main.rs",
            "old_text": "fn main() {}",
            "new_text": "fn main() { println!(\"hello\"); }"
        }"#;
        let params: EditFileParams = serde_json::from_str(json).expect("parse");
        assert_eq!(params.path, "/workspace/task-1/src/main.rs");
        assert_eq!(params.old_text, "fn main() {}");
    }

    #[test]
    fn json_schema_generates() {
        let schema = rmcp::schemars::schema_for!(EditFileParams);
        let json = serde_json::to_value(&schema).expect("json");
        assert!(json.get("properties").is_some());
    }
}
