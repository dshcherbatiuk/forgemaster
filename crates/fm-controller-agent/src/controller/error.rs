//! Controller error types.

use thiserror::Error;

/// Errors that can occur during Agent reconciliation.
#[derive(Debug, Error)]
pub enum ReconcileError {
    /// Failed to get the Agent resource.
    #[error("failed to get Agent: {0}")]
    GetAgent(#[source] kube::Error),

    /// Failed to update Agent status.
    #[error("failed to update Agent status: {0}")]
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

    /// Invalid agent state.
    #[error("invalid agent state: {0}")]
    InvalidState(String),
}

/// Result type for reconciliation operations.
pub type ReconcileResult<T> = Result<T, ReconcileError>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn error_display_missing_field() {
        let error = ReconcileError::MissingField("namespace".to_string());
        assert_eq!(error.to_string(), "missing required field: namespace");
    }

    #[test]
    fn error_display_invalid_state() {
        let error = ReconcileError::InvalidState("no strategy for phase".to_string());
        assert!(error.to_string().contains("no strategy for phase"));
    }

    #[test]
    fn error_display_create_resource() {
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
