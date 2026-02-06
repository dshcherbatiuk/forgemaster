//! Pending clarification struct for unresolved questions.

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use typed_builder::TypedBuilder;

use super::AttemptedSource;

/// A pending clarification question that needs an answer.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema, TypedBuilder)]
#[serde(rename_all = "camelCase")]
pub struct PendingClarification {
    /// Unique question ID.
    pub id: String,

    /// Question text.
    pub question: String,

    /// Suggested options (optional).
    #[builder(default, setter(strip_option))]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub options: Option<Vec<String>>,

    /// Whether answer is required to proceed.
    #[builder(default = true)]
    #[serde(default = "default_required")]
    pub required: bool,

    /// Sources already tried to resolve this.
    #[builder(default)]
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub attempted_sources: Vec<AttemptedSource>,

    /// True if user input is needed (last resort).
    #[builder(default)]
    #[serde(default)]
    pub blocked_on_user: bool,
}

const fn default_required() -> bool {
    true
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::crd::ClarificationSource;

    #[test]
    fn build_pending_clarification_minimal() {
        let pending = PendingClarification::builder()
            .id("db-choice".to_string())
            .question("Which database should be used?".to_string())
            .build();

        assert_eq!(pending.id, "db-choice");
        assert_eq!(pending.question, "Which database should be used?");
        assert!(pending.options.is_none());
        assert!(pending.required);
        assert!(pending.attempted_sources.is_empty());
        assert!(!pending.blocked_on_user);
    }

    #[test]
    fn build_pending_clarification_full() {
        let pending = PendingClarification::builder()
            .id("db-choice".to_string())
            .question("Which database?".to_string())
            .options(vec![
                "PostgreSQL".to_string(),
                "MySQL".to_string(),
                "MongoDB".to_string(),
            ])
            .required(true)
            .attempted_sources(vec![
                AttemptedSource::builder()
                    .source(ClarificationSource::System)
                    .result("No match".to_string())
                    .build(),
            ])
            .blocked_on_user(true)
            .build();

        assert_eq!(pending.options.as_ref().unwrap().len(), 3);
        assert!(pending.blocked_on_user);
        assert_eq!(pending.attempted_sources.len(), 1);
    }

    #[test]
    fn pending_clarification_serialization() {
        let pending = PendingClarification::builder()
            .id("test".to_string())
            .question("Test question?".to_string())
            .blocked_on_user(true)
            .build();

        let json = serde_json::to_string(&pending).expect("serialize");
        assert!(json.contains("\"blockedOnUser\":true"));
        assert!(!json.contains("attemptedSources")); // Empty vec should be skipped
    }

    #[test]
    fn pending_clarification_deserialization() {
        let json = r#"{
            "id": "framework",
            "question": "Which framework?",
            "options": ["Axum", "Actix"],
            "required": false,
            "blockedOnUser": true
        }"#;

        let pending: PendingClarification = serde_json::from_str(json).expect("deserialize");
        assert_eq!(pending.id, "framework");
        assert!(!pending.required);
        assert!(pending.blocked_on_user);
        assert_eq!(pending.options.unwrap().len(), 2);
    }
}
