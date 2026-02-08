//! Search file contents by regex action.

use std::path::Path;

use regex::Regex;
use rmcp::model::{CallToolResult, Content};
use tracing::debug;

use crate::param::SearchFilesParams;
use crate::workspace::assert_workspace_path;

/// Maximum number of matches to return.
const MAX_MATCHES: usize = 100;

/// Searches file contents recursively by regex pattern.
pub async fn execute(params: SearchFilesParams) -> CallToolResult {
    if let Err(msg) = assert_workspace_path(&params.path) {
        return CallToolResult::error(vec![Content::text(msg)]);
    }

    debug!(
        "🔍 search_files: {} pattern={}",
        params.path, params.pattern
    );

    let regex = match Regex::new(&params.pattern) {
        Ok(r) => r,
        Err(err) => {
            return CallToolResult::error(vec![Content::text(format!(
                "Invalid regex pattern '{}': {err}",
                params.pattern
            ))]);
        }
    };

    let mut matches = Vec::new();
    if let Err(err) = search_recursive(Path::new(&params.path), &regex, &mut matches).await {
        return CallToolResult::error(vec![Content::text(format!(
            "Search failed in {}: {err}",
            params.path
        ))]);
    }

    let response = if matches.is_empty() {
        format!(
            "No matches found for '{}' in {}",
            params.pattern, params.path
        )
    } else {
        let truncated = matches.len() >= MAX_MATCHES;
        let mut output = matches.join("\n");
        if truncated {
            use std::fmt::Write;
            let _ = write!(output, "\n... (truncated at {MAX_MATCHES} matches)");
        }
        output
    };

    CallToolResult::success(vec![Content::text(response)])
}

/// Recursively searches files for regex matches.
async fn search_recursive(
    path: &Path,
    regex: &Regex,
    matches: &mut Vec<String>,
) -> std::io::Result<()> {
    if matches.len() >= MAX_MATCHES {
        return Ok(());
    }

    let mut read_dir = tokio::fs::read_dir(path).await?;
    while let Some(entry) = read_dir.next_entry().await? {
        if matches.len() >= MAX_MATCHES {
            break;
        }

        let entry_path = entry.path();
        let file_type = entry.file_type().await?;

        if file_type.is_dir() {
            Box::pin(search_recursive(&entry_path, regex, matches)).await?;
        } else if file_type.is_file() {
            search_file(&entry_path, regex, matches).await;
        }
    }

    Ok(())
}

/// Searches a single file for regex matches.
async fn search_file(path: &Path, regex: &Regex, matches: &mut Vec<String>) {
    let Ok(contents) = tokio::fs::read_to_string(path).await else {
        return; // Skip binary / unreadable files
    };

    let path_str = path.to_string_lossy();
    for (line_num, line) in contents.lines().enumerate() {
        if matches.len() >= MAX_MATCHES {
            break;
        }
        if regex.is_match(line) {
            matches.push(format!("{}:{}: {}", path_str, line_num + 1, line));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn search_outside_workspace() {
        let params = SearchFilesParams {
            path: "/etc".to_string(),
            pattern: "root".to_string(),
        };
        let result = execute(params).await;
        assert!(result.is_error.unwrap_or(false));
    }

    #[tokio::test]
    async fn search_recursive_finds_matches() {
        let dir = tempfile::tempdir().expect("tempdir");
        std::fs::write(dir.path().join("hello.txt"), "hello world\ngoodbye world").expect("write");
        std::fs::write(dir.path().join("other.txt"), "no match here").expect("write");

        let regex = Regex::new("hello").expect("regex");
        let mut matches = Vec::new();
        search_recursive(dir.path(), &regex, &mut matches)
            .await
            .expect("search");

        assert_eq!(matches.len(), 1);
        assert!(matches[0].contains("hello.txt:1:"));
    }

    #[tokio::test]
    async fn search_recursive_no_matches() {
        let dir = tempfile::tempdir().expect("tempdir");
        std::fs::write(dir.path().join("file.txt"), "nothing here").expect("write");

        let regex = Regex::new("zzz_nonexistent").expect("regex");
        let mut matches = Vec::new();
        search_recursive(dir.path(), &regex, &mut matches)
            .await
            .expect("search");

        assert!(matches.is_empty());
    }

    #[test]
    fn max_matches_constant() {
        assert_eq!(MAX_MATCHES, 100);
    }
}
