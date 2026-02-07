//! Agent Custom Resource Definition.

use kube::CustomResource;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use super::status::AgentStatus;

/// Agent is the CRD for managing LLM agents in ForgeMaster.
#[derive(CustomResource, Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[kube(
    group = "forgemaster.io",
    version = "v1alpha1",
    kind = "Agent",
    plural = "agents",
    shortname = "ag",
    status = "AgentStatus",
    namespaced,
    printcolumn = r#"{"name": "Type", "type": "string", "jsonPath": ".spec.type"}"#,
    printcolumn = r#"{"name": "Phase", "type": "string", "jsonPath": ".status.phase"}"#,
    printcolumn = r#"{"name": "Tokens", "type": "integer", "jsonPath": ".status.tokensUsed"}"#,
    printcolumn = r#"{"name": "Age", "type": "date", "jsonPath": ".metadata.creationTimestamp"}"#
)]
#[serde(rename_all = "camelCase")]
pub struct AgentCrd {
    /// Agent type (e.g., "orchestrator", "code-generator", "architect").
    #[serde(rename = "type")]
    pub agent_type: String,

    /// LLM model configuration.
    pub model: crate::crd::ModelConfig,

    /// Full instruction prompt: role definition + task description.
    pub task_prompt: String,

    /// MCP servers this agent can use.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub mcp_servers: Vec<crate::crd::McpServerRef>,

    /// Resource limits and requests.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub resources: Option<crate::crd::ResourceRequirements>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use kube::core::ObjectMeta;

    #[test]
    fn create_agent() {
        let agent = Agent {
            metadata: ObjectMeta {
                name: Some("orchestrator-abc123".to_string()),
                namespace: Some("task-abc123".to_string()),
                ..Default::default()
            },
            spec: AgentCrd {
                agent_type: "orchestrator".to_string(),
                model: crate::crd::ModelConfig::builder()
                    .name("claude-sonnet-4-20250514".to_string())
                    .build(),
                task_prompt: "You are an orchestrator.".to_string(),
                mcp_servers: vec![],
                resources: None,
            },
            status: None,
        };

        assert_eq!(agent.metadata.name, Some("orchestrator-abc123".to_string()));
        assert_eq!(agent.spec.agent_type, "orchestrator");
    }

    #[test]
    fn api_resource() {
        use kube::Resource;

        assert_eq!(Agent::group(&()), "forgemaster.io");
        assert_eq!(Agent::version(&()), "v1alpha1");
        assert_eq!(Agent::kind(&()), "Agent");
        assert_eq!(Agent::plural(&()), "agents");
    }

    #[test]
    fn serialization() {
        let agent = Agent {
            metadata: ObjectMeta {
                name: Some("test".to_string()),
                ..Default::default()
            },
            spec: AgentCrd {
                agent_type: "code-generator".to_string(),
                model: crate::crd::ModelConfig::builder()
                    .name("claude-sonnet-4-20250514".to_string())
                    .build(),
                task_prompt: "Generate code.".to_string(),
                mcp_servers: vec![],
                resources: None,
            },
            status: None,
        };

        let json = serde_json::to_value(&agent).unwrap_or_default();
        assert_eq!(json["spec"]["type"], "code-generator");
        assert_eq!(json["spec"]["taskPrompt"], "Generate code.");
    }
}
