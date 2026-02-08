//! List directory entries action.

use rmcp::model::{CallToolResult, Content};
use tracing::debug;

use crate::param::ListDirectoryParams;
use crate::workspace::assert_workspace_path;

/// Lists entries in a directory with type indicators.
pub async fn execute(params: ListDirectoryParams) -> CallToolResult {
    if let Err(msg) = assert_workspace_path(&params.path) {
        return CallToolResult::error(vec![Content::text(msg)]);
    }

    debug!("📂 list_directory: {}", params.path);

    let mut read_dir = match tokio::fs::read_dir(&params.path).await {
        Ok(rd) => rd,
        Err(err) => {
            return CallToolResult::error(vec![Content::text(format!(
                "Failed to list {}: {err}",
                params.path
            ))]);
        }
    };

    let mut entries = Vec::new();
    while let Ok(Some(entry)) = read_dir.next_entry().await {
        let name = entry.file_name().to_string_lossy().to_string();
        let is_dir = entry
            .file_type()
            .await
            .map(|ft| ft.is_dir())
            .unwrap_or(false);
        let suffix = if is_dir { "/" } else { "" };
        entries.push(format!("{name}{suffix}"));
    }

    entries.sort();

    let response = if entries.is_empty() {
        format!("{} (empty directory)", params.path)
    } else {
        entries.join("\n")
    };

    CallToolResult::success(vec![Content::text(response)])
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn list_outside_workspace() {
        let params = ListDirectoryParams {
            path: "/etc".to_string(),
        };
        let result = execute(params).await;
        assert!(result.is_error.unwrap_or(false));
    }
}
