//! Agent spec.

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use typed_builder::TypedBuilder;

use crate::crd::{McpServerRef, ModelConfig, ResourceRequirements};

/// Spec for an Agent CR.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema, TypedBuilder)]
#[serde(rename_all = "camelCase")]
pub struct AgentSpec {
    /// Agent type (e.g., "orchestrator", "code-generator", "architect").
    #[serde(rename = "type")]
    pub agent_type: String,

    /// LLM model configuration.
    pub model: ModelConfig,

    /// Instruction text defining the agent's role and behavior.
    pub system_prompt: String,

    /// MCP servers this agent can use. Dynamically updateable.
    #[builder(default)]
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub mcp_servers: Vec<McpServerRef>,

    /// Resource limits and requests for the agent pod.
    #[builder(default, setter(strip_option))]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub resources: Option<ResourceRequirements>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn build_minimal() {
        let spec = AgentSpec::builder()
            .agent_type("orchestrator".to_string())
            .model(
                ModelConfig::builder()
                    .name("claude-sonnet-4-20250514".to_string())
                    .build(),
            )
            .system_prompt("You are an orchestrator.".to_string())
            .build();

        assert_eq!(spec.agent_type, "orchestrator");
        assert!(spec.mcp_servers.is_empty());
        assert!(spec.resources.is_none());
    }

    #[test]
    fn build_full() {
        let spec = AgentSpec::builder()
            .agent_type("code-generator".to_string())
            .model(
                ModelConfig::builder()
                    .name("claude-sonnet-4-20250514".to_string())
                    .temperature(0.3)
                    .build(),
            )
            .system_prompt("Generate code.".to_string())
            .mcp_servers(vec![
                McpServerRef::builder()
                    .name("github-mcp".to_string())
                    .build(),
            ])
            .resources(ResourceRequirements::builder().build())
            .build();

        assert_eq!(spec.mcp_servers.len(), 1);
        assert!(spec.resources.is_some());
    }

    #[test]
    fn serialization_renames_type() {
        let spec = AgentSpec::builder()
            .agent_type("reviewer".to_string())
            .model(
                ModelConfig::builder()
                    .name("claude-sonnet-4-20250514".to_string())
                    .build(),
            )
            .system_prompt("Review code.".to_string())
            .build();

        let json = serde_json::to_value(&spec).unwrap_or_default();
        assert_eq!(json["type"], "reviewer");
        assert!(json.get("agentType").is_none());
        assert_eq!(json["systemPrompt"], "Review code.");
    }

    #[test]
    fn serialization_skips_empty() {
        let spec = AgentSpec::builder()
            .agent_type("test".to_string())
            .model(ModelConfig::builder().name("model".to_string()).build())
            .system_prompt("prompt".to_string())
            .build();

        let json = serde_json::to_value(&spec).unwrap_or_default();
        assert!(json.get("mcpServers").is_none());
        assert!(json.get("resources").is_none());
    }

    #[test]
    fn deserialization() {
        let json = r#"{
            "type": "architect",
            "model": {
                "provider": "anthropic",
                "name": "claude-sonnet-4-20250514",
                "temperature": 0.5,
                "maxTokens": 8192
            },
            "systemPrompt": "Design systems.",
            "mcpServers": [{"name": "github-mcp"}]
        }"#;

        let spec: AgentSpec = serde_json::from_str(json).unwrap_or_else(|_| {
            AgentSpec::builder()
                .agent_type("fallback".to_string())
                .model(ModelConfig::builder().name("f".to_string()).build())
                .system_prompt("f".to_string())
                .build()
        });

        assert_eq!(spec.agent_type, "architect");
        assert_eq!(spec.mcp_servers.len(), 1);
        assert_eq!(spec.model.max_tokens, 8192);
    }
}
