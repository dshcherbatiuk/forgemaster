//! Controller error types.

use thiserror::Error;

/// Errors that can occur during AgentTask reconciliation.
#[derive(Debug, Error)]
pub enum ReconcileError {
    /// Failed to get the AgentTask resource.
    #[error("failed to get AgentTask: {0}")]
    GetTask(#[source] kube::Error),

    /// Failed to update AgentTask status.
    #[error("failed to update AgentTask status: {0}")]
    UpdateStatus(#[source] kube::Error),

    /// Failed to create child resource.
    #[error("failed to create {resource}: {source}")]
    CreateResource {
        /// Name of the resource that failed to create.
        resource: String,
        /// The underlying Kubernetes error.
        #[source]
        source: kube::Error,
    },

    /// Failed to delete child resource.
    #[error("failed to delete {resource}: {source}")]
    DeleteResource {
        /// Name of the resource that failed to delete.
        resource: String,
        /// The underlying Kubernetes error.
        #[source]
        source: kube::Error,
    },

    /// Missing required field.
    #[error("missing required field: {0}")]
    MissingField(String),

    /// Invalid task state.
    #[error("invalid task state: {0}")]
    InvalidState(String),
}

/// Result type for reconciliation operations.
pub type ReconcileResult<T> = Result<T, ReconcileError>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reconcile_error_display() {
        let error = ReconcileError::MissingField("namespace".to_string());
        assert_eq!(error.to_string(), "missing required field: namespace");
    }

    #[test]
    fn reconcile_error_invalid_state() {
        let error = ReconcileError::InvalidState("cannot transition from Failed to Running".to_string());
        assert!(error.to_string().contains("cannot transition"));
    }

    #[test]
    fn reconcile_error_create_resource() {
        let kube_error = kube::Error::Api(kube::error::ErrorResponse {
            status: "Failure".to_string(),
            message: "already exists".to_string(),
            reason: "AlreadyExists".to_string(),
            code: 409,
        });

        let error = ReconcileError::CreateResource {
            resource: "Pod".to_string(),
            source: kube_error,
        };

        assert!(error.to_string().contains("failed to create Pod"));
    }
}
