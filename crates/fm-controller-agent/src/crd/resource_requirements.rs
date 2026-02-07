//! Kubernetes resource requirements (limits + requests).

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use typed_builder::TypedBuilder;

use super::resource_limits::ResourceLimits;

/// Kubernetes resource requirements for an agent pod.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema, TypedBuilder)]
pub struct ResourceRequirements {
    /// Resource limits.
    #[builder(default, setter(strip_option))]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub limits: Option<ResourceLimits>,

    /// Resource requests.
    #[builder(default, setter(strip_option))]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub requests: Option<ResourceLimits>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn build_full() {
        let requirements = ResourceRequirements::builder()
            .limits(
                ResourceLimits::builder()
                    .memory("512Mi".to_string())
                    .cpu("500m".to_string())
                    .build(),
            )
            .requests(
                ResourceLimits::builder()
                    .memory("256Mi".to_string())
                    .cpu("250m".to_string())
                    .build(),
            )
            .build();

        assert!(requirements.limits.is_some());
        assert!(requirements.requests.is_some());
    }

    #[test]
    fn build_empty() {
        let requirements = ResourceRequirements::builder().build();
        assert!(requirements.limits.is_none());
        assert!(requirements.requests.is_none());
    }

    #[test]
    fn serialization_skips_none() {
        let requirements = ResourceRequirements::builder()
            .limits(
                ResourceLimits::builder()
                    .memory("512Mi".to_string())
                    .build(),
            )
            .build();

        let json = serde_json::to_value(&requirements).unwrap_or_default();
        assert!(json.get("limits").is_some());
        assert!(json.get("requests").is_none());
    }

    #[test]
    fn deserialization() {
        let json = r#"{"limits": {"memory": "1Gi"}, "requests": {"cpu": "250m"}}"#;
        let requirements: ResourceRequirements =
            serde_json::from_str(json).unwrap_or_else(|_| ResourceRequirements::builder().build());
        assert!(requirements.limits.is_some());
        assert!(requirements.requests.is_some());
    }
}
