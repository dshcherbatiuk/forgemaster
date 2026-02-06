//! Pod reference for agent status.

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use typed_builder::TypedBuilder;

/// Reference to the Pod running an agent.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema, TypedBuilder)]
pub struct PodRef {
    /// Pod name.
    pub name: String,

    /// Pod UID.
    pub uid: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn build() {
        let pod = PodRef::builder()
            .name("agent-pod-abc123".to_string())
            .uid("uid-123".to_string())
            .build();

        assert_eq!(pod.name, "agent-pod-abc123");
        assert_eq!(pod.uid, "uid-123");
    }

    #[test]
    fn serialization() {
        let pod = PodRef::builder()
            .name("agent-pod".to_string())
            .uid("uid-456".to_string())
            .build();

        let json = serde_json::to_value(&pod).unwrap_or_default();
        assert_eq!(json["name"], "agent-pod");
        assert_eq!(json["uid"], "uid-456");
    }

    #[test]
    fn deserialization() {
        let json = r#"{"name": "my-pod", "uid": "pod-uid"}"#;
        let pod: PodRef = serde_json::from_str(json).unwrap_or_else(|_| {
            PodRef::builder()
                .name("fallback".to_string())
                .uid("fallback".to_string())
                .build()
        });
        assert_eq!(pod.name, "my-pod");
    }
}
