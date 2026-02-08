//! Write file contents action.

use rmcp::model::{CallToolResult, Content};
use tracing::debug;

use crate::param::WriteFileParams;
use crate::workspace::assert_workspace_path;

/// Writes content to a file, creating parent directories as needed.
pub async fn execute(params: WriteFileParams) -> CallToolResult {
    if let Err(msg) = assert_workspace_path(&params.path) {
        return CallToolResult::error(vec![Content::text(msg)]);
    }

    debug!("📝 write_file: {}", params.path);

    // Create parent directories if they don't exist
    if let Some(parent) = std::path::Path::new(&params.path).parent()
        && let Err(err) = tokio::fs::create_dir_all(parent).await
    {
        return CallToolResult::error(vec![Content::text(format!(
            "Failed to create parent directories for {}: {err}",
            params.path
        ))]);
    }

    match tokio::fs::write(&params.path, &params.content).await {
        Ok(()) => {
            let response = format!("Written {} bytes to {}", params.content.len(), params.path);
            CallToolResult::success(vec![Content::text(response)])
        }
        Err(err) => CallToolResult::error(vec![Content::text(format!(
            "Failed to write {}: {err}",
            params.path
        ))]),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn write_outside_workspace() {
        let params = WriteFileParams {
            path: "/etc/evil.txt".to_string(),
            content: "bad".to_string(),
        };
        let result = execute(params).await;
        assert!(result.is_error.unwrap_or(false));
    }
}
