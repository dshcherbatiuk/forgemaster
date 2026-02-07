//! Top-level error types for the Agent Runtime.

use std::time::Duration;

/// Errors that can occur during agent runtime execution.
#[derive(Debug, thiserror::Error)]
pub enum RuntimeError {
    /// Configuration is missing or invalid.
    #[error("configuration error: {0}")]
    Config(String),

    /// Kubernetes API call failed.
    #[error("kubernetes error: {0}")]
    Kubernetes(#[from] kube::Error),

    /// Claude API call failed.
    #[error("claude API error: {0}")]
    ClaudeApi(#[from] crate::claude_api::ClaudeApiError),

    /// The Agent CR was not found in the cluster.
    #[error("agent CR not found: {name} in namespace {namespace}")]
    AgentNotFound {
        /// Agent name.
        name: String,
        /// Namespace.
        namespace: String,
    },

    /// Execution exceeded the allowed duration.
    #[error("execution timeout after {0:?}")]
    Timeout(Duration),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn config_error_formats() {
        let err = RuntimeError::Config("missing AGENT_NAME".to_string());
        assert_eq!(err.to_string(), "configuration error: missing AGENT_NAME");
    }

    #[test]
    fn agent_not_found_formats() {
        let err = RuntimeError::AgentNotFound {
            name: "orchestrator-task-abc".to_string(),
            namespace: "task-abc".to_string(),
        };
        assert!(err.to_string().contains("orchestrator-task-abc"));
        assert!(err.to_string().contains("task-abc"));
    }

    #[test]
    fn timeout_formats() {
        let err = RuntimeError::Timeout(Duration::from_secs(300));
        assert!(err.to_string().contains("300"));
    }
}
