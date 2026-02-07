//! Kubernetes resource limits (memory, CPU).

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use typed_builder::TypedBuilder;

/// Kubernetes resource limits or requests.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema, TypedBuilder)]
pub struct ResourceLimits {
    /// Memory (e.g., "512Mi").
    #[builder(default, setter(strip_option))]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub memory: Option<String>,

    /// CPU (e.g., "500m").
    #[builder(default, setter(strip_option))]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cpu: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn build_full() {
        let limits = ResourceLimits::builder()
            .memory("512Mi".to_string())
            .cpu("500m".to_string())
            .build();

        assert_eq!(limits.memory.as_deref(), Some("512Mi"));
        assert_eq!(limits.cpu.as_deref(), Some("500m"));
    }

    #[test]
    fn build_partial() {
        let limits = ResourceLimits::builder()
            .memory("256Mi".to_string())
            .build();

        assert_eq!(limits.memory.as_deref(), Some("256Mi"));
        assert!(limits.cpu.is_none());
    }

    #[test]
    fn serialization_skips_none() {
        let limits = ResourceLimits::builder()
            .memory("512Mi".to_string())
            .build();

        let json = serde_json::to_value(&limits).unwrap_or_default();
        assert_eq!(json["memory"], "512Mi");
        assert!(json.get("cpu").is_none());
    }

    #[test]
    fn deserialization() {
        let json = r#"{"memory": "1Gi", "cpu": "1"}"#;
        let limits: ResourceLimits =
            serde_json::from_str(json).unwrap_or_else(|_| ResourceLimits::builder().build());
        assert_eq!(limits.memory.as_deref(), Some("1Gi"));
        assert_eq!(limits.cpu.as_deref(), Some("1"));
    }
}
