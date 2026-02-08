//! Directory tree action.

use rmcp::model::{CallToolResult, Content};
use tracing::debug;

use crate::param::DirectoryTreeParams;
use crate::workspace::assert_workspace_path;

/// Generates a recursive directory tree string.
pub async fn execute(params: DirectoryTreeParams) -> CallToolResult {
    if let Err(msg) = assert_workspace_path(&params.path) {
        return CallToolResult::error(vec![Content::text(msg)]);
    }

    debug!(
        "🌳 directory_tree: {} (depth={})",
        params.path, params.max_depth
    );

    let mut output = String::new();
    if let Err(err) = build_tree(&params.path, "", params.max_depth, &mut output).await {
        return CallToolResult::error(vec![Content::text(format!(
            "Failed to build tree for {}: {err}",
            params.path
        ))]);
    }

    if output.is_empty() {
        output = format!("{} (empty)", params.path);
    }

    CallToolResult::success(vec![Content::text(output)])
}

/// Recursively builds a tree string.
async fn build_tree(
    path: &str,
    prefix: &str,
    remaining_depth: u32,
    output: &mut String,
) -> std::io::Result<()> {
    use std::fmt::Write;

    let mut read_dir = tokio::fs::read_dir(path).await?;
    let mut entries = Vec::new();

    while let Some(entry) = read_dir.next_entry().await? {
        entries.push(entry);
    }

    entries.sort_by_key(tokio::fs::DirEntry::file_name);

    let total = entries.len();
    for (idx, entry) in entries.iter().enumerate() {
        let name = entry.file_name().to_string_lossy().to_string();
        let is_last = idx == total - 1;
        let connector = if is_last { "└── " } else { "├── " };
        let is_dir = entry
            .file_type()
            .await
            .map(|ft| ft.is_dir())
            .unwrap_or(false);
        let suffix = if is_dir { "/" } else { "" };

        let _ = writeln!(output, "{prefix}{connector}{name}{suffix}");

        if is_dir && remaining_depth > 0 {
            let child_prefix = if is_last {
                format!("{prefix}    ")
            } else {
                format!("{prefix}│   ")
            };
            let child_path = format!("{path}/{name}");
            Box::pin(build_tree(
                &child_path,
                &child_prefix,
                remaining_depth - 1,
                output,
            ))
            .await?;
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn tree_outside_workspace() {
        let params = DirectoryTreeParams {
            path: "/etc".to_string(),
            max_depth: 1,
        };
        let result = execute(params).await;
        assert!(result.is_error.unwrap_or(false));
    }

    #[tokio::test]
    async fn build_tree_from_tempdir() {
        let dir = tempfile::tempdir().expect("tempdir");
        std::fs::write(dir.path().join("file.txt"), "").expect("write");
        std::fs::create_dir(dir.path().join("sub")).expect("mkdir");
        std::fs::write(dir.path().join("sub/nested.txt"), "").expect("write");

        let mut output = String::new();
        let path = dir.path().to_str().expect("path");
        build_tree(path, "", 5, &mut output).await.expect("tree");

        assert!(output.contains("file.txt"));
        assert!(output.contains("sub/"));
        assert!(output.contains("nested.txt"));
    }

    #[tokio::test]
    async fn build_tree_respects_depth_limit() {
        let dir = tempfile::tempdir().expect("tempdir");
        std::fs::create_dir_all(dir.path().join("a/b/c")).expect("mkdir");
        std::fs::write(dir.path().join("a/b/c/deep.txt"), "").expect("write");

        let mut output = String::new();
        let path = dir.path().to_str().expect("path");
        build_tree(path, "", 1, &mut output).await.expect("tree");

        assert!(output.contains("a/"));
        assert!(output.contains("b/"));
        assert!(!output.contains("deep.txt"));
    }
}
