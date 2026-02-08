//! Runs external commands and captures output.

use anyhow::Result;
use tracing::info;

/// Result of an external command execution.
#[derive(Debug)]
pub struct CommandOutput {
    /// Standard output.
    pub stdout: String,
    /// Standard error.
    pub stderr: String,
    /// Process exit code (0 = success).
    pub exit_code: i32,
}

impl CommandOutput {
    /// Returns `true` if the command exited with code 0.
    #[must_use]
    pub const fn is_success(&self) -> bool {
        self.exit_code == 0
    }
}

/// Runs a command with the given arguments and returns captured output.
///
/// # Errors
///
/// Returns an error if the command cannot be spawned (e.g. binary not found).
pub async fn run(command: &str, args: &[&str]) -> Result<CommandOutput> {
    info!("🔧 Running: {} {}", command, args.join(" "));

    let output = tokio::process::Command::new(command)
        .args(args)
        .output()
        .await?;

    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
    let stderr = String::from_utf8_lossy(&output.stderr).to_string();
    let exit_code = output.status.code().unwrap_or(1);

    Ok(CommandOutput {
        stdout,
        stderr,
        exit_code,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn run_echo_succeeds() {
        let output = run("echo", &["hello"]).await.expect("echo should work");
        assert!(output.is_success());
        assert_eq!(output.stdout.trim(), "hello");
    }

    #[tokio::test]
    async fn run_nonexistent_command_fails() {
        let result = run("nonexistent_command_xyz_12345", &[]).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn run_false_returns_nonzero() {
        let output = run("false", &[]).await.expect("false should spawn");
        assert!(!output.is_success());
        assert_ne!(output.exit_code, 0);
    }

    #[test]
    fn command_output_success_check() {
        let output = CommandOutput {
            stdout: String::new(),
            stderr: String::new(),
            exit_code: 0,
        };
        assert!(output.is_success());
    }

    #[test]
    fn command_output_failure_check() {
        let output = CommandOutput {
            stdout: String::new(),
            stderr: "error".to_string(),
            exit_code: 1,
        };
        assert!(!output.is_success());
    }
}
