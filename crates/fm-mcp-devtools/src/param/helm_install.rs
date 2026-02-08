//! Parameters for the `helm_install` MCP tool.

use serde::Deserialize;

/// Parameters for deploying a Helm chart.
#[derive(Debug, Deserialize, rmcp::schemars::JsonSchema)]
#[schemars(crate = "rmcp::schemars")]
pub struct HelmInstallParams {
    /// Helm release name.
    pub release_name: String,
    /// Path to Helm chart directory (absolute path under workspace).
    pub chart_path: String,
    /// Kubernetes namespace to deploy into.
    pub namespace: String,
    /// Comma-separated key=value pairs for `--set` (e.g. "image.tag=v1,replicas=2").
    pub set_values: Option<String>,
    /// Create namespace if it does not exist.
    #[serde(default = "default_true")]
    pub create_namespace: bool,
}

const fn default_true() -> bool {
    true
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn deserialize_minimal() {
        let json = r#"{
            "release_name": "my-app",
            "chart_path": "/workspace/task-1/helm",
            "namespace": "task-1"
        }"#;
        let params: HelmInstallParams = serde_json::from_str(json).expect("should parse");
        assert_eq!(params.release_name, "my-app");
        assert!(params.create_namespace);
        assert!(params.set_values.is_none());
    }

    #[test]
    fn deserialize_with_set_values() {
        let json = r#"{
            "release_name": "my-app",
            "chart_path": "/workspace/task-1/helm",
            "namespace": "production",
            "set_values": "image.tag=v2,replicas=3",
            "create_namespace": false
        }"#;
        let params: HelmInstallParams = serde_json::from_str(json).expect("should parse");
        assert_eq!(params.set_values.as_deref(), Some("image.tag=v2,replicas=3"));
        assert!(!params.create_namespace);
    }
}
