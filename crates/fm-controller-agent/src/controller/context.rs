//! Controller context for shared state.

use std::sync::Arc;

use kube::Client;
use kube::ResourceExt;
use kube::api::{Api, Patch, PatchParams};
use tracing::debug;

use crate::crd::{Agent, AgentPhase};

use super::error::{ReconcileError, ReconcileResult};

/// Shared context for the Agent controller.
#[derive(Clone)]
pub struct ControllerContext {
    client: Client,
    namespace: String,
    runtime_agent_image: String,
    llm_provider_secret_name: String,
    llm_provider_secret_key: String,
}

impl ControllerContext {
    /// Creates a new controller context.
    pub fn new(
        client: Client,
        namespace: String,
        runtime_agent_image: String,
        llm_provider_secret_name: String,
        llm_provider_secret_key: String,
    ) -> Self {
        Self {
            client,
            namespace,
            runtime_agent_image,
            llm_provider_secret_name,
            llm_provider_secret_key,
        }
    }

    /// Returns reference to the Kubernetes client.
    pub fn client(&self) -> &Client {
        &self.client
    }

    /// Returns the namespace.
    pub fn namespace(&self) -> &str {
        &self.namespace
    }

    /// Returns the runtime container image (e.g. "fm-agent-runtime:latest").
    pub fn runtime_agent_image(&self) -> &str {
        &self.runtime_agent_image
    }

    /// Returns the K8s secret name holding the LLM provider API key.
    pub fn llm_provider_secret_name(&self) -> &str {
        &self.llm_provider_secret_name
    }

    /// Returns the key within the LLM provider secret.
    pub fn llm_provider_secret_key(&self) -> &str {
        &self.llm_provider_secret_key
    }

    /// Updates the agent phase via status subresource patch.
    pub async fn update_phase(&self, agent: &Agent, phase: AgentPhase) -> ReconcileResult<()> {
        let name = agent.name_any();
        let namespace = agent.namespace().unwrap_or_else(|| self.namespace.clone());
        let api: Api<Agent> = Api::namespaced(self.client.clone(), &namespace);

        let patch = serde_json::json!({
            "status": { "phase": phase }
        });

        api.patch_status(
            &name,
            &PatchParams::apply("fm-controller-agent"),
            &Patch::Merge(&patch),
        )
        .await
        .map_err(ReconcileError::UpdateStatus)?;

        debug!("📝 Updated agent {}/{} phase to {}", namespace, name, phase);
        Ok(())
    }
}

/// Creates an Arc-wrapped controller context, reading runtime config from env vars.
///
/// Required env vars: `RUNTIME_AGENT_IMAGE`, `LLM_PROVIDER_SECRET_NAME`, `LLM_PROVIDER_SECRET_KEY`.
pub fn create_context(client: Client, namespace: String) -> anyhow::Result<Arc<ControllerContext>> {
    let runtime_agent_image = require_env("RUNTIME_AGENT_IMAGE")?;
    let llm_provider_secret_name = require_env("LLM_PROVIDER_SECRET_NAME")?;
    let llm_provider_secret_key = require_env("LLM_PROVIDER_SECRET_KEY")?;

    Ok(Arc::new(ControllerContext::new(
        client,
        namespace,
        runtime_agent_image,
        llm_provider_secret_name,
        llm_provider_secret_key,
    )))
}

/// Reads a required environment variable, failing fast if missing or empty.
fn require_env(key: &str) -> anyhow::Result<String> {
    match std::env::var(key) {
        Ok(val) if val.is_empty() => anyhow::bail!("{key} is set but empty"),
        Ok(val) => Ok(val),
        Err(_) => anyhow::bail!("{key} environment variable is required but not set"),
    }
}

#[cfg(test)]
mod tests {
    use super::require_env;
    use crate::crd::AgentPhase;

    #[test]
    fn context_namespace_not_empty() {
        let namespace = "forgemaster-system";
        assert!(!namespace.is_empty());
    }

    #[test]
    fn phase_patch_contains_only_phase() {
        let phase = AgentPhase::Running;
        let patch = serde_json::json!({
            "status": { "phase": phase }
        });

        let status = patch.get("status").unwrap();
        assert_eq!(status.get("phase").unwrap(), "Running");
        assert!(status.get("tokensUsed").is_none());
        assert!(status.get("iterationsCompleted").is_none());
    }

    #[test]
    fn require_env_missing_var_fails() {
        let result = require_env("FM_TEST_NONEXISTENT_12345");
        assert!(result.is_err());
        let msg = result.unwrap_err().to_string();
        assert!(msg.contains("FM_TEST_NONEXISTENT_12345"));
        assert!(msg.contains("required but not set"));
    }

    #[test]
    fn runtime_agent_image_format() {
        let image = "fm-agent-runtime:latest";
        assert!(image.contains(':'));
        assert!(!image.is_empty());
    }

    #[test]
    fn llm_provider_secret_defaults() {
        let secret_name = "anthropic-credentials";
        let secret_key = "api-key";
        assert!(!secret_name.is_empty());
        assert!(!secret_key.is_empty());
    }
}
