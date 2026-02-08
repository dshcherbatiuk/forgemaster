//! Parameters for the `helm_status` MCP tool.

use serde::Deserialize;

/// Parameters for checking a Helm release status.
#[derive(Debug, Deserialize, rmcp::schemars::JsonSchema)]
#[schemars(crate = "rmcp::schemars")]
pub struct HelmStatusParams {
    /// Helm release name.
    pub release_name: String,
    /// Kubernetes namespace.
    pub namespace: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn deserialize() {
        let json = r#"{"release_name": "my-app", "namespace": "default"}"#;
        let params: HelmStatusParams = serde_json::from_str(json).expect("should parse");
        assert_eq!(params.release_name, "my-app");
        assert_eq!(params.namespace, "default");
    }
}
