//! LLM model configuration.

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use typed_builder::TypedBuilder;

use super::model_provider::ModelProvider;

/// LLM model configuration for an agent.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema, TypedBuilder)]
#[serde(rename_all = "camelCase")]
pub struct ModelConfig {
    /// LLM provider.
    #[builder(default)]
    pub provider: ModelProvider,

    /// Model name (e.g., "claude-sonnet-4-20250514").
    pub name: String,

    /// Sampling temperature (0.0 - 2.0). Lower values produce deterministic output.
    #[builder(default = 0.7)]
    pub temperature: f64,

    /// Maximum tokens the LLM can generate per response.
    #[builder(default = 4096)]
    pub max_tokens: i32,

    /// Maximum tool-calling iterations per conversation.
    #[serde(default = "default_max_tool_iterations")]
    #[builder(default = 50)]
    pub max_tool_iterations: u32,
}

const fn default_max_tool_iterations() -> u32 {
    50
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn build_minimal() {
        let config = ModelConfig::builder()
            .name("claude-sonnet-4-20250514".to_string())
            .build();

        assert_eq!(config.provider, ModelProvider::Anthropic);
        assert_eq!(config.name, "claude-sonnet-4-20250514");
        assert!((config.temperature - 0.7).abs() < f64::EPSILON);
        assert_eq!(config.max_tokens, 4096);
        assert_eq!(config.max_tool_iterations, 50);
    }

    #[test]
    fn build_custom() {
        let config = ModelConfig::builder()
            .name("claude-opus-4-20250514".to_string())
            .temperature(0.2)
            .max_tokens(8192)
            .max_tool_iterations(100)
            .build();

        assert!((config.temperature - 0.2).abs() < f64::EPSILON);
        assert_eq!(config.max_tokens, 8192);
        assert_eq!(config.max_tool_iterations, 100);
    }

    #[test]
    fn serialization() {
        let config = ModelConfig::builder()
            .name("claude-sonnet-4-20250514".to_string())
            .build();

        let json = serde_json::to_value(&config).unwrap_or_default();
        assert_eq!(json["provider"], "anthropic");
        assert_eq!(json["name"], "claude-sonnet-4-20250514");
        assert_eq!(json["maxTokens"], 4096);
        assert_eq!(json["maxToolIterations"], 50);
    }

    #[test]
    fn deserialization() {
        let json = r#"{
            "provider": "anthropic",
            "name": "claude-sonnet-4-20250514",
            "temperature": 0.5,
            "maxTokens": 2048,
            "maxToolIterations": 75
        }"#;

        let config: ModelConfig = serde_json::from_str(json)
            .unwrap_or_else(|_| ModelConfig::builder().name("fallback".to_string()).build());

        assert_eq!(config.provider, ModelProvider::Anthropic);
        assert_eq!(config.max_tokens, 2048);
        assert_eq!(config.max_tool_iterations, 75);
    }

    #[test]
    fn deserialization_without_max_tool_iterations_uses_default() {
        let json = r#"{
            "provider": "anthropic",
            "name": "claude-sonnet-4-20250514",
            "temperature": 0.7,
            "maxTokens": 4096
        }"#;

        let config: ModelConfig = serde_json::from_str(json).expect("should deserialize");
        assert_eq!(config.max_tool_iterations, 50);
    }
}
