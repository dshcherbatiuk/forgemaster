//! Create directory action.

use rmcp::model::{CallToolResult, Content};
use tracing::debug;

use crate::param::CreateDirectoryParams;
use crate::workspace::assert_workspace_path;

/// Creates a directory recursively (like `mkdir -p`).
pub async fn execute(params: CreateDirectoryParams) -> CallToolResult {
    if let Err(msg) = assert_workspace_path(&params.path) {
        return CallToolResult::error(vec![Content::text(msg)]);
    }

    debug!("📁 create_directory: {}", params.path);

    match tokio::fs::create_dir_all(&params.path).await {
        Ok(()) => {
            let response = format!("Created directory: {}", params.path);
            CallToolResult::success(vec![Content::text(response)])
        }
        Err(err) => CallToolResult::error(vec![Content::text(format!(
            "Failed to create directory {}: {err}",
            params.path
        ))]),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn create_outside_workspace() {
        let params = CreateDirectoryParams {
            path: "/tmp/evil-dir".to_string(),
        };
        let result = execute(params).await;
        assert!(result.is_error.unwrap_or(false));
    }
}
