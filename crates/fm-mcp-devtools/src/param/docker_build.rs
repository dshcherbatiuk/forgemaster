//! Parameters for the `docker_build` MCP tool.

use serde::Deserialize;

/// Parameters for building a Docker image.
#[derive(Debug, Deserialize, rmcp::schemars::JsonSchema)]
#[schemars(crate = "rmcp::schemars")]
pub struct DockerBuildParams {
    /// Build context directory (absolute path under workspace).
    pub context_path: String,
    /// Path to Dockerfile (absolute path under workspace).
    pub dockerfile: String,
    /// Image name (e.g. "my-service").
    pub image_name: String,
    /// Image tag (defaults to "latest").
    #[serde(default = "default_tag")]
    pub image_tag: String,
}

fn default_tag() -> String {
    "latest".to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn deserialize_with_defaults() {
        let json = r#"{
            "context_path": "/workspace/task-1/src",
            "dockerfile": "/workspace/task-1/Dockerfile",
            "image_name": "calculator"
        }"#;
        let params: DockerBuildParams = serde_json::from_str(json).expect("should parse");
        assert_eq!(params.image_tag, "latest");
    }

    #[test]
    fn deserialize_with_custom_tag() {
        let json = r#"{
            "context_path": "/workspace/task-1",
            "dockerfile": "/workspace/task-1/Dockerfile",
            "image_name": "calculator",
            "image_tag": "v1.0.0"
        }"#;
        let params: DockerBuildParams = serde_json::from_str(json).expect("should parse");
        assert_eq!(params.image_tag, "v1.0.0");
        assert_eq!(params.image_name, "calculator");
    }
}
