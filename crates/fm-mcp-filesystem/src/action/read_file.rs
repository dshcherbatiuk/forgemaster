//! Read file contents action.

use rmcp::model::{CallToolResult, Content};
use tracing::debug;

use crate::param::ReadFileParams;
use crate::workspace::assert_workspace_path;

/// Reads a file and returns its text contents.
pub async fn execute(params: ReadFileParams) -> CallToolResult {
    if let Err(msg) = assert_workspace_path(&params.path) {
        return CallToolResult::error(vec![Content::text(msg)]);
    }

    debug!("📂 read_file: {}", params.path);

    match tokio::fs::read_to_string(&params.path).await {
        Ok(contents) => CallToolResult::success(vec![Content::text(contents)]),
        Err(err) => CallToolResult::error(vec![Content::text(format!(
            "Failed to read {}: {err}",
            params.path
        ))]),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn read_nonexistent_file() {
        let params = ReadFileParams {
            path: "/workspace/nonexistent.txt".to_string(),
        };
        let result = execute(params).await;
        assert!(result.is_error.unwrap_or(false));
    }

    #[tokio::test]
    async fn read_outside_workspace() {
        let params = ReadFileParams {
            path: "/etc/passwd".to_string(),
        };
        let result = execute(params).await;
        assert!(result.is_error.unwrap_or(false));
    }
}
