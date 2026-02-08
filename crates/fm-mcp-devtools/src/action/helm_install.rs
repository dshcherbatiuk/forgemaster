//! Helm chart installation action.

use rmcp::model::{CallToolResult, Content};
use tracing::info;

use super::command_runner;
use crate::param::HelmInstallParams;
use crate::workspace::assert_workspace_path;

/// Deploys a Helm chart to Kubernetes.
///
/// Runs `helm upgrade --install` with the given parameters.
///
/// # Errors
///
/// Returns an error if the command fails to spawn.
pub async fn execute(params: HelmInstallParams) -> CallToolResult {
    if let Err(msg) = assert_workspace_path(&params.chart_path) {
        return CallToolResult::error(vec![Content::text(msg)]);
    }

    info!(
        "⎈ Deploying Helm release {} from {} to namespace {}",
        params.release_name, params.chart_path, params.namespace
    );

    let mut args = vec![
        "upgrade",
        "--install",
        &params.release_name,
        &params.chart_path,
        "--namespace",
        &params.namespace,
    ];

    if params.create_namespace {
        args.push("--create-namespace");
    }

    let set_values_string;
    if let Some(ref values) = params.set_values {
        set_values_string = values.clone();
        args.push("--set");
        args.push(&set_values_string);
    }

    let result = command_runner::run("helm", &args).await;

    match result {
        Ok(output) => {
            let text = format_output(&params.release_name, &params.namespace, &output);
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

fn format_output(
    release_name: &str,
    namespace: &str,
    output: &command_runner::CommandOutput,
) -> String {
    let mut text = format!(
        "Release: {release_name}\nNamespace: {namespace}\nExit code: {}\n\n--- stdout ---\n{}",
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
            stdout: "Release \"my-app\" has been upgraded.".to_string(),
            stderr: String::new(),
            exit_code: 0,
        };
        let text = format_output("my-app", "default", &output);
        assert!(text.contains("Release: my-app"));
        assert!(text.contains("Namespace: default"));
        assert!(text.contains("Exit code: 0"));
        assert!(text.contains("has been upgraded"));
    }

    #[test]
    fn format_output_failure() {
        let output = command_runner::CommandOutput {
            stdout: String::new(),
            stderr: "Error: chart not found".to_string(),
            exit_code: 1,
        };
        let text = format_output("my-app", "prod", &output);
        assert!(text.contains("Exit code: 1"));
        assert!(text.contains("chart not found"));
    }

    #[test]
    fn invalid_chart_path_returns_error() {
        let params = HelmInstallParams {
            release_name: "test".to_string(),
            chart_path: "/tmp/evil-chart".to_string(),
            namespace: "default".to_string(),
            set_values: None,
            create_namespace: true,
        };
        let rt = tokio::runtime::Runtime::new().expect("runtime");
        let result = rt.block_on(execute(params));
        assert!(result.is_error.unwrap_or(false));
    }
}
