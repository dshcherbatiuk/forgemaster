//! Copies LLM provider secrets from the controller namespace to task namespaces.

use k8s_openapi::api::core::v1::Secret;
use kube::api::{Api, ObjectMeta, PostParams};
use tracing::info;

use super::context::ControllerContext;
use super::error::{ReconcileError, ReconcileResult};

/// Ensures the LLM provider secret exists in the target namespace.
/// Copies it from the controller namespace if missing.
pub async fn ensure_secret(ctx: &ControllerContext, target_namespace: &str) -> ReconcileResult<()> {
    let secret_name = ctx.llm_provider_secret_name();
    let client = ctx.client().clone();

    let target_api: Api<Secret> = Api::namespaced(client.clone(), target_namespace);

    // Skip if secret already exists
    if target_api
        .get_opt(secret_name)
        .await
        .map_err(ReconcileError::GetAgent)?
        .is_some()
    {
        return Ok(());
    }

    // Read from controller namespace
    let source_api: Api<Secret> = Api::namespaced(client, ctx.namespace());
    let source_secret = source_api
        .get(secret_name)
        .await
        .map_err(|source| ReconcileError::CreateResource {
            resource: format!("Secret/{secret_name} (read from {})", ctx.namespace()),
            source,
        })?;

    // Create in target namespace (handle race condition)
    let target_secret = Secret {
        metadata: ObjectMeta {
            name: Some(secret_name.to_string()),
            namespace: Some(target_namespace.to_string()),
            ..Default::default()
        },
        data: source_secret.data,
        type_: source_secret.type_,
        ..Default::default()
    };

    match target_api
        .create(&PostParams::default(), &target_secret)
        .await
    {
        Ok(_) => {
            info!(
                "🔑 Copied secret {} to namespace {}",
                secret_name, target_namespace
            );
        }
        Err(kube::Error::Api(err)) if err.reason == "AlreadyExists" => {}
        Err(source) => {
            return Err(ReconcileError::CreateResource {
                resource: format!("Secret/{secret_name} in {target_namespace}"),
                source,
            });
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    #[test]
    fn secret_metadata_structure() {
        use kube::api::ObjectMeta;

        let meta = ObjectMeta {
            name: Some("anthropic-credentials".to_string()),
            namespace: Some("task-abc".to_string()),
            ..Default::default()
        };

        assert_eq!(meta.name.unwrap(), "anthropic-credentials");
        assert_eq!(meta.namespace.unwrap(), "task-abc");
        // No owner references — secret is independent of any Agent CR
        assert!(meta.owner_references.is_none());
    }
}
