//! Edit file contents action (find-and-replace).

use rmcp::model::{CallToolResult, Content};
use tracing::debug;

use crate::param::EditFileParams;
use crate::workspace::assert_workspace_path;

/// Replaces the first occurrence of `old_text` with `new_text` in a file.
pub async fn execute(params: EditFileParams) -> CallToolResult {
    if let Err(msg) = assert_workspace_path(&params.path) {
        return CallToolResult::error(vec![Content::text(msg)]);
    }

    debug!("✏️ edit_file: {}", params.path);

    let contents = match tokio::fs::read_to_string(&params.path).await {
        Ok(c) => c,
        Err(err) => {
            return CallToolResult::error(vec![Content::text(format!(
                "Failed to read {}: {err}",
                params.path
            ))]);
        }
    };

    if !contents.contains(&params.old_text) {
        return CallToolResult::error(vec![Content::text(format!(
            "Text not found in {}: {:?}",
            params.path,
            truncate(&params.old_text, 100)
        ))]);
    }

    let updated = contents.replacen(&params.old_text, &params.new_text, 1);

    match tokio::fs::write(&params.path, &updated).await {
        Ok(()) => {
            let response = format!("Edited {}: replaced 1 occurrence", params.path);
            CallToolResult::success(vec![Content::text(response)])
        }
        Err(err) => CallToolResult::error(vec![Content::text(format!(
            "Failed to write {}: {err}",
            params.path
        ))]),
    }
}

/// Truncates a string for display in error messages.
fn truncate(s: &str, max_len: usize) -> &str {
    if s.len() <= max_len { s } else { &s[..max_len] }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn edit_outside_workspace() {
        let params = EditFileParams {
            path: "/etc/passwd".to_string(),
            old_text: "root".to_string(),
            new_text: "admin".to_string(),
        };
        let result = execute(params).await;
        assert!(result.is_error.unwrap_or(false));
    }

    #[test]
    fn truncate_short_string() {
        assert_eq!(truncate("hello", 10), "hello");
    }

    #[test]
    fn truncate_long_string() {
        let long = "a".repeat(200);
        assert_eq!(truncate(&long, 100).len(), 100);
    }
}
