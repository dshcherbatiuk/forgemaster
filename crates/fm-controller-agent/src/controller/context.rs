//! Controller context for shared state.

use std::sync::Arc;

use kube::Client;
use kube::ResourceExt;
use kube::api::{Api, Patch, PatchParams};
use smallvec::SmallVec;
use tracing::debug;

use crate::crd::{Agent, AgentPhase, McpServerRef};

use super::error::{ReconcileError, ReconcileResult};

/// Default port for MCP servers (used when port is omitted in config).
const DEFAULT_MCP_SERVER_PORT: u16 = 3000;

/// Most orchestrators use 1–2 MCP servers; stack-allocate up to 4.
pub type McpServerRefs = SmallVec<[McpServerRef; 4]>;

/// Shared context for the Agent controller.
#[derive(Clone)]
pub struct ControllerContext {
    client: Client,
    namespace: String,
    runtime_agent_image: String,
    default_model: String,
    llm_provider_secret_name: String,
    llm_provider_secret_key: String,
    default_mcp_servers: McpServerRefs,
}

impl ControllerContext {
    /// Creates a new controller context.
    pub fn new(
        client: Client,
        namespace: String,
        runtime_agent_image: String,
        default_model: String,
        llm_provider_secret_name: String,
        llm_provider_secret_key: String,
        default_mcp_servers: McpServerRefs,
    ) -> Self {
        Self {
            client,
            namespace,
            runtime_agent_image,
            default_model,
            llm_provider_secret_name,
            llm_provider_secret_key,
            default_mcp_servers,
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

    /// Returns the runtime container image (e.g. "fm-agent-runtime-claude:latest").
    pub fn runtime_agent_image(&self) -> &str {
        &self.runtime_agent_image
    }

    /// Returns the K8s secret name holding the LLM provider API key.
    pub fn llm_provider_secret_name(&self) -> &str {
        &self.llm_provider_secret_name
    }

    /// Returns the default LLM model name (e.g. "claude-sonnet-4-20250514").
    pub fn default_model(&self) -> &str {
        &self.default_model
    }

    /// Returns the key within the LLM provider secret.
    pub fn llm_provider_secret_key(&self) -> &str {
        &self.llm_provider_secret_key
    }

    /// Returns default MCP server refs assigned to agents.
    pub fn default_mcp_servers(&self) -> &[McpServerRef] {
        &self.default_mcp_servers
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
/// Required: `RUNTIME_AGENT_IMAGE`, `LLM_PROVIDER_DEFAULT_MODEL`,
/// `LLM_PROVIDER_SECRET_NAME`, `LLM_PROVIDER_SECRET_KEY`.
///
/// Optional: `DEFAULT_MCP_SERVERS` — comma-separated `name:port` pairs
/// (port defaults to 3000). Example: `fm-controller-agent:3000,github-mcp`.
pub fn create_context(client: Client, namespace: String) -> anyhow::Result<Arc<ControllerContext>> {
    let runtime_agent_image = require_env("RUNTIME_AGENT_IMAGE")?;
    let default_model = require_env("LLM_PROVIDER_DEFAULT_MODEL")?;
    let llm_provider_secret_name = require_env("LLM_PROVIDER_SECRET_NAME")?;
    let llm_provider_secret_key = require_env("LLM_PROVIDER_SECRET_KEY")?;

    let default_mcp_servers = std::env::var("DEFAULT_MCP_SERVERS")
        .ok()
        .map(|val| parse_mcp_server_refs(&val))
        .unwrap_or_default();

    Ok(Arc::new(ControllerContext::new(
        client,
        namespace,
        runtime_agent_image,
        default_model,
        llm_provider_secret_name,
        llm_provider_secret_key,
        default_mcp_servers,
    )))
}

/// Parses a comma-separated `name:port` string into MCP server refs.
///
/// Port is optional and defaults to [`DEFAULT_MCP_SERVER_PORT`].
/// Empty segments are skipped.
///
/// Examples: `"fm-controller-agent:3000,github-mcp"`, `"single-mcp"`.
fn parse_mcp_server_refs(input: &str) -> McpServerRefs {
    input
        .split(',')
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(parse_single_mcp_ref)
        .collect()
}

/// Parses `"name:port"` or `"name"` into an `McpServerRef`.
fn parse_single_mcp_ref(entry: &str) -> McpServerRef {
    match entry.rsplit_once(':') {
        Some((name, port_str)) => {
            let port = port_str.parse().unwrap_or(DEFAULT_MCP_SERVER_PORT);
            McpServerRef::builder()
                .name(name.to_string())
                .port(port)
                .build()
        }
        None => McpServerRef::builder()
            .name(entry.to_string())
            .build(),
    }
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
    use super::*;
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
        let image = "fm-agent-runtime-claude:latest";
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

    #[test]
    fn parse_single_ref_name_only() {
        let server = parse_single_mcp_ref("github-mcp");
        assert_eq!(server.name, "github-mcp");
        assert_eq!(server.port, DEFAULT_MCP_SERVER_PORT);
    }

    #[test]
    fn parse_single_ref_name_and_port() {
        let server = parse_single_mcp_ref("github-mcp:9090");
        assert_eq!(server.name, "github-mcp");
        assert_eq!(server.port, 9090);
    }

    #[test]
    fn parse_single_ref_invalid_port_uses_default() {
        let server = parse_single_mcp_ref("github-mcp:abc");
        assert_eq!(server.name, "github-mcp");
        assert_eq!(server.port, DEFAULT_MCP_SERVER_PORT);
    }

    #[test]
    fn parse_refs_empty_string() {
        let servers = parse_mcp_server_refs("");
        assert!(servers.is_empty());
    }

    #[test]
    fn parse_refs_single_entry() {
        let servers = parse_mcp_server_refs("fm-controller-agent:3000");
        assert_eq!(servers.len(), 1);
        assert_eq!(servers[0].name, "fm-controller-agent");
        assert_eq!(servers[0].port, 3000);
    }

    #[test]
    fn parse_refs_multiple_entries() {
        let servers = parse_mcp_server_refs("fm-controller-agent:3000,github-mcp");
        assert_eq!(servers.len(), 2);
        assert_eq!(servers[0].name, "fm-controller-agent");
        assert_eq!(servers[0].port, 3000);
        assert_eq!(servers[1].name, "github-mcp");
        assert_eq!(servers[1].port, DEFAULT_MCP_SERVER_PORT);
    }

    #[test]
    fn parse_refs_trims_whitespace() {
        let servers = parse_mcp_server_refs("  a-mcp:3000 , b-mcp:4000  ");
        assert_eq!(servers.len(), 2);
        assert_eq!(servers[0].name, "a-mcp");
        assert_eq!(servers[1].name, "b-mcp");
    }

    #[test]
    fn parse_refs_skips_empty_segments() {
        let servers = parse_mcp_server_refs("a-mcp,,b-mcp,");
        assert_eq!(servers.len(), 2);
    }
}
