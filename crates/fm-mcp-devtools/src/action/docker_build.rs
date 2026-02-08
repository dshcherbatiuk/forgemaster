//! Docker image build action.

use rmcp::model::{CallToolResult, Content};
use tracing::info;

use super::command_runner;
use crate::param::DockerBuildParams;
use crate::workspace::assert_workspace_path;

/// Builds a Docker image from a Dockerfile in the workspace.
///
/// # Errors
///
/// Returns an error if the command fails to spawn.
pub async fn execute(params: DockerBuildParams) -> CallToolResult {
    if let Err(msg) = assert_workspace_path(&params.context_path) {
        return CallToolResult::error(vec![Content::text(msg)]);
    }
    if let Err(msg) = assert_workspace_path(&params.dockerfile) {
        return CallToolResult::error(vec![Content::text(msg)]);
    }

    let full_tag = format!("{}:{}", params.image_name, params.image_tag);

    info!(
        "🐳 Building Docker image {} from {}",
        full_tag, params.dockerfile
    );

    let result = command_runner::run(
        "docker",
        &[
            "build",
            "--progress=plain",
            "-f",
            &params.dockerfile,
            "-t",
            &full_tag,
            &params.context_path,
        ],
    )
    .await;

    match result {
        Ok(output) => {
            let text = format_output(&full_tag, &output);
            if output.is_success() {
                CallToolResult::success(vec![Content::text(text)])
            } else {
                CallToolResult::error(vec![Content::text(text)])
            }
        }
        Err(err) => {
            CallToolResult::error(vec![Content::text(format!("Failed to run docker: {err}"))])
        }
    }
}

fn format_output(image_tag: &str, output: &command_runner::CommandOutput) -> String {
    let mut text = format!(
        "Image: {image_tag}\nExit code: {}\n\n--- stdout ---\n{}",
        output.exit_code, output.stdout
    );
    if !output.stderr.is_empty() {
        use std::fmt::Write;
        let _ = write!(text, "\n\n--- stderr ---\n{}", output.stderr);
    }
    text
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn format_output_success() {
        let output = command_runner::CommandOutput {
            stdout: "Successfully built abc123".to_string(),
            stderr: String::new(),
            exit_code: 0,
        };
        let text = format_output("my-app:latest", &output);
        assert!(text.contains("Image: my-app:latest"));
        assert!(text.contains("Exit code: 0"));
        assert!(text.contains("Successfully built"));
        assert!(!text.contains("--- stderr ---"));
    }

    #[test]
    fn format_output_failure_with_stderr() {
        let output = command_runner::CommandOutput {
            stdout: String::new(),
            stderr: "Dockerfile not found".to_string(),
            exit_code: 1,
        };
        let text = format_output("my-app:v1", &output);
        assert!(text.contains("Exit code: 1"));
        assert!(text.contains("--- stderr ---"));
        assert!(text.contains("Dockerfile not found"));
    }

    #[test]
    fn invalid_workspace_path_returns_error() {
        let params = DockerBuildParams {
            context_path: "/etc/passwd".to_string(),
            dockerfile: "/workspace/Dockerfile".to_string(),
            image_name: "test".to_string(),
            image_tag: "latest".to_string(),
        };
        let rt = tokio::runtime::Runtime::new().expect("runtime");
        let result = rt.block_on(execute(params));
        assert!(result.is_error.unwrap_or(false));
    }
}
