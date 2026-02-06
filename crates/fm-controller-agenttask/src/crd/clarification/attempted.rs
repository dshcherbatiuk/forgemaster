//! Attempted source struct for tracking clarification attempts.

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use typed_builder::TypedBuilder;

use super::ClarificationSource;

/// A source that was attempted to resolve a clarification.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema, TypedBuilder)]
pub struct AttemptedSource {
    /// The source that was tried.
    pub source: ClarificationSource,

    /// Why this source could not answer.
    pub result: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn build_attempted_source() {
        let attempted = AttemptedSource::builder()
            .source(ClarificationSource::System)
            .result("No matching pattern found".to_string())
            .build();

        assert_eq!(attempted.source, ClarificationSource::System);
        assert_eq!(attempted.result, "No matching pattern found");
    }

    #[test]
    fn attempted_source_serialization() {
        let attempted = AttemptedSource::builder()
            .source(ClarificationSource::External)
            .result("API unavailable".to_string())
            .build();

        let json = serde_json::to_string(&attempted).expect("serialize");
        assert!(json.contains("\"source\":\"external\""));
        assert!(json.contains("\"result\":\"API unavailable\""));
    }

    #[test]
    fn attempted_source_deserialization() {
        let json = r#"{"source": "system", "result": "Config not found"}"#;

        let attempted: AttemptedSource = serde_json::from_str(json).expect("deserialize");
        assert_eq!(attempted.source, ClarificationSource::System);
        assert_eq!(attempted.result, "Config not found");
    }
}
