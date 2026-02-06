//! Output reference for agent results.

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use typed_builder::TypedBuilder;

/// Reference to a ConfigMap containing agent output.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema, TypedBuilder)]
#[serde(rename_all = "camelCase")]
pub struct OutputRef {
    /// ConfigMap name with agent output.
    pub config_map_ref: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn build() {
        let output = OutputRef::builder()
            .config_map_ref("agent-output-abc123".to_string())
            .build();

        assert_eq!(output.config_map_ref, "agent-output-abc123");
    }

    #[test]
    fn serialization_uses_camel_case() {
        let output = OutputRef::builder()
            .config_map_ref("output-cm".to_string())
            .build();

        let json = serde_json::to_value(&output).unwrap_or_default();
        assert_eq!(json["configMapRef"], "output-cm");
    }

    #[test]
    fn deserialization() {
        let json = r#"{"configMapRef": "my-output"}"#;
        let output: OutputRef = serde_json::from_str(json).unwrap_or_else(|_| {
            OutputRef::builder()
                .config_map_ref("fallback".to_string())
                .build()
        });
        assert_eq!(output.config_map_ref, "my-output");
    }
}
