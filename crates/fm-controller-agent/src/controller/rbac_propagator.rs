//! Creates RBAC resources for agent runtime pods in task namespaces.
//!
//! The controller creates a ServiceAccount and RoleBinding in each task
//! namespace so that runtime pods can patch their Agent CR status.

use k8s_openapi::api::core::v1::ServiceAccount;
use k8s_openapi::api::rbac::v1::{RoleBinding, RoleRef, Subject};
use kube::api::{Api, PostParams};
use kube::api::ObjectMeta;
use tracing::info;

use super::context::ControllerContext;
use super::error::{ReconcileError, ReconcileResult};

/// Service account name used by agent runtime pods.
pub const RUNTIME_SERVICE_ACCOUNT: &str = "fm-agent-runtime-claude";

/// ClusterRole name granting runtime pods permission to patch Agent status.
const RUNTIME_CLUSTER_ROLE: &str = "fm-agent-runtime-claude";

/// Ensures the runtime ServiceAccount and RoleBinding exist in the target namespace.
pub async fn ensure_rbac(ctx: &ControllerContext, target_namespace: &str) -> ReconcileResult<()> {
    ensure_service_account(ctx, target_namespace).await?;
    ensure_role_binding(ctx, target_namespace).await?;
    Ok(())
}

/// Creates the ServiceAccount if it does not exist.
async fn ensure_service_account(
    ctx: &ControllerContext,
    target_namespace: &str,
) -> ReconcileResult<()> {
    let sa_api: Api<ServiceAccount> =
        Api::namespaced(ctx.client().clone(), target_namespace);

    let sa = ServiceAccount {
        metadata: ObjectMeta {
            name: Some(RUNTIME_SERVICE_ACCOUNT.to_string()),
            namespace: Some(target_namespace.to_string()),
            ..Default::default()
        },
        ..Default::default()
    };

    match sa_api.create(&PostParams::default(), &sa).await {
        Ok(_) => {
            info!(
                "🔐 Created ServiceAccount {} in {}",
                RUNTIME_SERVICE_ACCOUNT, target_namespace
            );
        }
        Err(kube::Error::Api(err)) if err.reason == "AlreadyExists" => {}
        Err(source) => {
            return Err(ReconcileError::CreateResource {
                resource: format!(
                    "ServiceAccount/{RUNTIME_SERVICE_ACCOUNT} in {target_namespace}"
                ),
                source,
            });
        }
    }

    Ok(())
}

/// Creates the RoleBinding if it does not exist.
async fn ensure_role_binding(
    ctx: &ControllerContext,
    target_namespace: &str,
) -> ReconcileResult<()> {
    let rb_api: Api<RoleBinding> =
        Api::namespaced(ctx.client().clone(), target_namespace);

    let rb = RoleBinding {
        metadata: ObjectMeta {
            name: Some(RUNTIME_SERVICE_ACCOUNT.to_string()),
            namespace: Some(target_namespace.to_string()),
            ..Default::default()
        },
        role_ref: RoleRef {
            api_group: "rbac.authorization.k8s.io".to_string(),
            kind: "ClusterRole".to_string(),
            name: RUNTIME_CLUSTER_ROLE.to_string(),
        },
        subjects: Some(vec![Subject {
            kind: "ServiceAccount".to_string(),
            name: RUNTIME_SERVICE_ACCOUNT.to_string(),
            namespace: Some(target_namespace.to_string()),
            ..Default::default()
        }]),
    };

    match rb_api.create(&PostParams::default(), &rb).await {
        Ok(_) => {
            info!(
                "🔐 Created RoleBinding {} in {}",
                RUNTIME_SERVICE_ACCOUNT, target_namespace
            );
        }
        Err(kube::Error::Api(err)) if err.reason == "AlreadyExists" => {}
        Err(source) => {
            return Err(ReconcileError::CreateResource {
                resource: format!(
                    "RoleBinding/{RUNTIME_SERVICE_ACCOUNT} in {target_namespace}"
                ),
                source,
            });
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn service_account_name_constant() {
        assert_eq!(RUNTIME_SERVICE_ACCOUNT, "fm-agent-runtime-claude");
    }

    #[test]
    fn cluster_role_name_matches_service_account() {
        assert_eq!(RUNTIME_CLUSTER_ROLE, "fm-agent-runtime-claude");
    }

    #[test]
    fn service_account_metadata() {
        let sa = ServiceAccount {
            metadata: ObjectMeta {
                name: Some(RUNTIME_SERVICE_ACCOUNT.to_string()),
                namespace: Some("task-abc".to_string()),
                ..Default::default()
            },
            ..Default::default()
        };

        assert_eq!(sa.metadata.name.unwrap(), "fm-agent-runtime-claude");
        assert_eq!(sa.metadata.namespace.unwrap(), "task-abc");
    }

    #[test]
    fn role_binding_references_cluster_role() {
        let rb = RoleBinding {
            metadata: ObjectMeta {
                name: Some(RUNTIME_SERVICE_ACCOUNT.to_string()),
                namespace: Some("task-abc".to_string()),
                ..Default::default()
            },
            role_ref: RoleRef {
                api_group: "rbac.authorization.k8s.io".to_string(),
                kind: "ClusterRole".to_string(),
                name: RUNTIME_CLUSTER_ROLE.to_string(),
            },
            subjects: Some(vec![Subject {
                kind: "ServiceAccount".to_string(),
                name: RUNTIME_SERVICE_ACCOUNT.to_string(),
                namespace: Some("task-abc".to_string()),
                ..Default::default()
            }]),
        };

        assert_eq!(rb.role_ref.kind, "ClusterRole");
        assert_eq!(rb.role_ref.name, "fm-agent-runtime-claude");
        let subject = &rb.subjects.unwrap()[0];
        assert_eq!(subject.kind, "ServiceAccount");
        assert_eq!(subject.name, "fm-agent-runtime-claude");
        assert_eq!(subject.namespace.as_deref(), Some("task-abc"));
    }

    #[test]
    fn role_binding_api_group() {
        let role_ref = RoleRef {
            api_group: "rbac.authorization.k8s.io".to_string(),
            kind: "ClusterRole".to_string(),
            name: RUNTIME_CLUSTER_ROLE.to_string(),
        };

        assert_eq!(role_ref.api_group, "rbac.authorization.k8s.io");
    }
}
