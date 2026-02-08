//! Workspace path validation.
//!
//! All filesystem operations are sandboxed under the workspace root.
//! The root is configurable via `WORKSPACE_ROOT` env var (default: `/workspace`).

use std::path::Path;

/// Default workspace root directory.
const DEFAULT_WORKSPACE_ROOT: &str = "/workspace";

/// Returns the workspace root path from env or default.
#[must_use]
pub fn workspace_root() -> String {
    std::env::var("WORKSPACE_ROOT").unwrap_or_else(|_| DEFAULT_WORKSPACE_ROOT.to_string())
}

/// Validates that a path is under the workspace root.
///
/// # Errors
///
/// Returns `Err` with a message if the path escapes the workspace root.
pub fn assert_workspace_path(path: &str) -> Result<(), String> {
    let root = workspace_root();
    assert_under_root(path, &root)
}

/// Validates that a path is under the given root directory.
///
/// Useful in tests where the workspace root is a temp directory.
///
/// # Errors
///
/// Returns `Err` with a message if the path escapes the root.
pub fn assert_under_root(path: &str, root: &str) -> Result<(), String> {
    let resolved = Path::new(path);

    if !resolved.starts_with(root) {
        return Err(format!("Path must be under {root}: {path}"));
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn valid_workspace_path() {
        assert!(assert_under_root("/workspace/task-1/src", "/workspace").is_ok());
    }

    #[test]
    fn valid_workspace_root_itself() {
        assert!(assert_under_root("/workspace", "/workspace").is_ok());
    }

    #[test]
    fn invalid_path_outside_workspace() {
        let result = assert_under_root("/etc/passwd", "/workspace");
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("must be under"));
    }

    #[test]
    fn invalid_tmp_path() {
        let result = assert_under_root("/tmp/evil", "/workspace");
        assert!(result.is_err());
    }

    #[test]
    fn default_workspace_root() {
        assert_eq!(DEFAULT_WORKSPACE_ROOT, "/workspace");
    }
}
