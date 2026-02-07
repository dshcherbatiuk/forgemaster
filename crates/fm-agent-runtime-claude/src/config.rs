//! Runtime configuration loaded from environment variables.
//!
//! All config is injected by the Agent Controller when creating the pod.

use anyhow::{Result, bail};

const DEFAULT_API_BASE_URL: &str = "https://api.anthropic.com";
const DEFAULT_MAX_TOKENS: i32 = 4096;
const DEFAULT_TEMPERATURE: f64 = 0.7;

/// Configuration for the agent runtime, loaded from environment variables.
///
/// The Agent Controller reads the Agent CR and injects these values
/// as env vars when creating the runtime pod.
#[derive(Debug, Clone)]
pub struct RuntimeConfig {
    /// Name of the Agent CR (set by controller from `metadata.name`).
    pub agent_name: String,
    /// Kubernetes namespace (set by controller from `metadata.namespace`).
    pub namespace: String,
    /// Anthropic API key for Claude (from K8s secret).
    pub api_key: String,
    /// Base URL for the Anthropic API.
    pub api_base_url: String,
    /// Full instruction prompt: role definition + task description (from `spec.taskPrompt`).
    pub task_prompt: String,
    /// Claude model name (from `spec.model.name`).
    pub model_name: String,
    /// Sampling temperature (from `spec.model.temperature`).
    pub model_temperature: f64,
    /// Max tokens per response (from `spec.model.maxTokens`).
    pub model_max_tokens: i32,
}

impl RuntimeConfig {
    /// Loads configuration from environment variables.
    ///
    /// Required: `AGENT_NAME`, `NAMESPACE`, `ANTHROPIC_API_KEY`,
    ///           `TASK_PROMPT`, `MODEL_NAME`.
    /// Optional: `ANTHROPIC_API_BASE_URL`, `MODEL_TEMPERATURE`, `MODEL_MAX_TOKENS`.
    pub fn from_env() -> Result<Self> {
        let agent_name = require_env("AGENT_NAME")?;
        let namespace = require_env("NAMESPACE")?;
        let api_key = require_env("ANTHROPIC_API_KEY")?;
        let task_prompt = require_env("TASK_PROMPT")?;
        let model_name = require_env("MODEL_NAME")?;

        let api_base_url = std::env::var("ANTHROPIC_API_BASE_URL")
            .unwrap_or_else(|_| DEFAULT_API_BASE_URL.to_string());

        let model_temperature = std::env::var("MODEL_TEMPERATURE")
            .ok()
            .and_then(|v| v.parse().ok())
            .unwrap_or(DEFAULT_TEMPERATURE);

        let model_max_tokens = std::env::var("MODEL_MAX_TOKENS")
            .ok()
            .and_then(|v| v.parse().ok())
            .unwrap_or(DEFAULT_MAX_TOKENS);

        Ok(Self {
            agent_name,
            namespace,
            api_key,
            api_base_url,
            task_prompt,
            model_name,
            model_temperature,
            model_max_tokens,
        })
    }

    /// Creates a config from explicit values (for testing and programmatic use).
    pub fn new(
        agent_name: String,
        namespace: String,
        api_key: String,
        task_prompt: String,
        model_name: String,
    ) -> Self {
        Self {
            agent_name,
            namespace,
            api_key,
            api_base_url: DEFAULT_API_BASE_URL.to_string(),
            task_prompt,
            model_name,
            model_temperature: DEFAULT_TEMPERATURE,
            model_max_tokens: DEFAULT_MAX_TOKENS,
        }
    }
}

/// Reads a required environment variable, failing fast if missing or empty.
fn require_env(key: &str) -> Result<String> {
    match std::env::var(key) {
        Ok(val) if val.is_empty() => bail!("{key} is set but empty"),
        Ok(val) => Ok(val),
        Err(_) => bail!("{key} environment variable is required but not set"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn require_env_missing_var_fails() {
        let result = require_env("FM_TEST_NONEXISTENT_VAR_12345");
        assert!(result.is_err());
        let msg = result.unwrap_err().to_string();
        assert!(msg.contains("FM_TEST_NONEXISTENT_VAR_12345"));
        assert!(msg.contains("required but not set"));
    }

    #[test]
    fn new_with_defaults() {
        let config = RuntimeConfig::new(
            "orchestrator-task-abc".to_string(),
            "task-abc".to_string(),
            "sk-ant-test123".to_string(),
            "You are an orchestrator.\n\nTask: Build an API".to_string(),
            "claude-sonnet-4-20250514".to_string(),
        );
        assert_eq!(config.agent_name, "orchestrator-task-abc");
        assert_eq!(config.namespace, "task-abc");
        assert_eq!(config.api_key, "sk-ant-test123");
        assert_eq!(config.api_base_url, DEFAULT_API_BASE_URL);
        assert!(config.task_prompt.contains("orchestrator"));
        assert!(config.task_prompt.contains("Build an API"));
        assert_eq!(config.model_name, "claude-sonnet-4-20250514");
        assert!((config.model_temperature - DEFAULT_TEMPERATURE).abs() < f64::EPSILON);
        assert_eq!(config.model_max_tokens, DEFAULT_MAX_TOKENS);
    }

    #[test]
    fn config_is_cloneable() {
        let config = RuntimeConfig::new(
            "agent".to_string(),
            "ns".to_string(),
            "key".to_string(),
            "task".to_string(),
            "model".to_string(),
        );
        let cloned = config.clone();
        assert_eq!(cloned.agent_name, config.agent_name);
        assert_eq!(cloned.task_prompt, config.task_prompt);
    }

    #[test]
    fn config_is_debuggable() {
        let config = RuntimeConfig::new(
            "agent".to_string(),
            "ns".to_string(),
            "key".to_string(),
            "task".to_string(),
            "model".to_string(),
        );
        let debug = format!("{config:?}");
        assert!(debug.contains("RuntimeConfig"));
        assert!(debug.contains("agent"));
    }

    #[test]
    fn default_temperature() {
        assert!((DEFAULT_TEMPERATURE - 0.7).abs() < f64::EPSILON);
    }

    #[test]
    fn default_max_tokens() {
        assert_eq!(DEFAULT_MAX_TOKENS, 4096);
    }
}
