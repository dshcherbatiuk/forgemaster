//! Helm release status check action.

use rmcp::model::{CallToolResult, Content};
use tracing::info;

use super::command_runner;
use crate::param::HelmStatusParams;

/// Checks the status of a Helm release.
///
/// # Errors
///
/// Returns an error if the command fails to spawn.
pub async fn execute(params: HelmStatusParams) -> CallToolResult {
    info!(
        "⎈ Checking Helm status for {} in namespace {}",
        params.release_name, params.namespace
    );

    let result = command_runner::run(
        "helm",
        &[
            "status",
            &params.release_name,
            "--namespace",
            &params.namespace,
        ],
    )
    .await;

    match result {
        Ok(output) => {
            let text = if output.stdout.is_empty() {
                output.stderr.clone()
            } else {
                output.stdout.clone()
            };
            if output.is_success() {
                CallToolResult::success(vec![Content::text(text)])
            } else {
                CallToolResult::error(vec![Content::text(text)])
            }
        }
        Err(err) => {
            CallToolResult::error(vec![Content::text(format!("Failed to run helm: {err}"))])
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn params_construction() {
        let params = HelmStatusParams {
            release_name: "my-app".to_string(),
            namespace: "production".to_string(),
        };
        assert_eq!(params.release_name, "my-app");
        assert_eq!(params.namespace, "production");
    }
}
