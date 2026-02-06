//! LLM provider enumeration.

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// LLM provider for agent model configuration.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "lowercase")]
pub enum ModelProvider {
    /// Anthropic Claude models.
    #[default]
    Anthropic,
}

impl std::fmt::Display for ModelProvider {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Anthropic => write!(f, "anthropic"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_provider_is_anthropic() {
        assert_eq!(ModelProvider::default(), ModelProvider::Anthropic);
    }

    #[test]
    fn display_anthropic() {
        assert_eq!(ModelProvider::Anthropic.to_string(), "anthropic");
    }

    #[test]
    fn serialization() {
        let json = serde_json::to_string(&ModelProvider::Anthropic).unwrap_or_default();
        assert_eq!(json, "\"anthropic\"");
    }

    #[test]
    fn deserialization() {
        let provider: ModelProvider =
            serde_json::from_str("\"anthropic\"").unwrap_or(ModelProvider::Anthropic);
        assert_eq!(provider, ModelProvider::Anthropic);
    }
}
