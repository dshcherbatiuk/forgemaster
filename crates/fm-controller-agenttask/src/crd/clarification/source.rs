//! Clarification source enumeration.

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// Source of a clarification answer.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "lowercase")]
pub enum ClarificationSource {
    /// Answer resolved by system (config, patterns, history).
    System,
    /// Answer resolved from external APIs or services.
    External,
    /// Answer provided by user (last resort).
    User,
}

impl std::fmt::Display for ClarificationSource {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::System => write!(f, "system"),
            Self::External => write!(f, "external"),
            Self::User => write!(f, "user"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn source_display() {
        assert_eq!(ClarificationSource::System.to_string(), "system");
        assert_eq!(ClarificationSource::External.to_string(), "external");
        assert_eq!(ClarificationSource::User.to_string(), "user");
    }

    #[test]
    fn source_serialization() {
        let source = ClarificationSource::External;
        let json = serde_json::to_string(&source).expect("serialize");
        assert_eq!(json, "\"external\"");
    }

    #[test]
    fn source_deserialization() {
        let source: ClarificationSource = serde_json::from_str("\"user\"").expect("deserialize");
        assert_eq!(source, ClarificationSource::User);
    }
}
