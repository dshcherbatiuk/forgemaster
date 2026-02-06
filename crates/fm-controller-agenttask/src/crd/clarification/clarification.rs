//! Clarification answer struct.

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use typed_builder::TypedBuilder;

use super::ClarificationSource;

/// A resolved clarification answer.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema, TypedBuilder)]
#[serde(rename_all = "camelCase")]
pub struct Clarification {
    /// ID of the question being answered.
    pub question_id: String,

    /// The answer value.
    pub answer: String,

    /// Who provided the answer.
    pub source: ClarificationSource,

    /// Explanation of how answer was resolved.
    #[builder(default, setter(strip_option))]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source_detail: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn build_clarification_minimal() {
        let clarification = Clarification::builder()
            .question_id("db-choice".to_string())
            .answer("PostgreSQL".to_string())
            .source(ClarificationSource::System)
            .build();

        assert_eq!(clarification.question_id, "db-choice");
        assert_eq!(clarification.answer, "PostgreSQL");
        assert_eq!(clarification.source, ClarificationSource::System);
        assert!(clarification.source_detail.is_none());
    }

    #[test]
    fn build_clarification_with_detail() {
        let clarification = Clarification::builder()
            .question_id("db-choice".to_string())
            .answer("PostgreSQL".to_string())
            .source(ClarificationSource::External)
            .source_detail("Detected from docker-compose.yml".to_string())
            .build();

        assert_eq!(
            clarification.source_detail,
            Some("Detected from docker-compose.yml".to_string())
        );
    }

    #[test]
    fn clarification_serialization() {
        let clarification = Clarification::builder()
            .question_id("db-choice".to_string())
            .answer("PostgreSQL".to_string())
            .source(ClarificationSource::User)
            .build();

        let json = serde_json::to_string(&clarification).expect("serialize");
        assert!(json.contains("\"questionId\":\"db-choice\""));
        assert!(json.contains("\"source\":\"user\""));
        assert!(!json.contains("sourceDetail")); // Should be skipped when None
    }

    #[test]
    fn clarification_deserialization() {
        let json = r#"{
            "questionId": "framework",
            "answer": "Axum",
            "source": "system",
            "sourceDetail": "Matched from task description"
        }"#;

        let clarification: Clarification = serde_json::from_str(json).expect("deserialize");
        assert_eq!(clarification.question_id, "framework");
        assert_eq!(clarification.answer, "Axum");
        assert_eq!(clarification.source, ClarificationSource::System);
        assert_eq!(
            clarification.source_detail,
            Some("Matched from task description".to_string())
        );
    }
}
