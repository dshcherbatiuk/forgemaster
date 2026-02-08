//! Converts Agent CRD resource requirements to K8s resource requirements.

use std::collections::BTreeMap;

use k8s_openapi::apimachinery::pkg::api::resource::Quantity;

use crate::crd::Agent;

/// Converts Agent CRD resource requirements to K8s resource requirements.
///
/// Returns `None` when the agent has no resource requirements configured.
pub fn build(agent: &Agent) -> Option<k8s_openapi::api::core::v1::ResourceRequirements> {
    let resources = agent.spec.resources.as_ref()?;

    Some(k8s_openapi::api::core::v1::ResourceRequirements {
        limits: resources.limits.as_ref().map(limits_to_quantity_map),
        requests: resources.requests.as_ref().map(limits_to_quantity_map),
        ..Default::default()
    })
}

/// Converts CRD resource limits to a K8s quantity map.
fn limits_to_quantity_map(limits: &crate::crd::ResourceLimits) -> BTreeMap<String, Quantity> {
    let mut map = BTreeMap::new();
    if let Some(memory) = &limits.memory {
        map.insert("memory".to_string(), Quantity(memory.clone()));
    }
    if let Some(cpu) = &limits.cpu {
        map.insert("cpu".to_string(), Quantity(cpu.clone()));
    }
    map
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::crd::{
        AgentCrd, ModelConfig,
        ResourceLimits, ResourceRequirements,
    };
    use kube::api::ObjectMeta;

    fn test_agent() -> Agent {
        Agent {
            metadata: ObjectMeta {
                name: Some("test-agent".to_string()),
                namespace: Some("task-abc".to_string()),
                ..Default::default()
            },
            spec: AgentCrd {
                agent_type: "orchestrator".to_string(),
                model: ModelConfig::builder()
                    .name("claude-sonnet-4-20250514".to_string())
                    .build(),
                task_prompt: "Build API".to_string(),
                mcp_servers: vec![],
                resources: None,
            },
            status: None,
        }
    }

    #[test]
    fn none_when_no_resources() {
        assert!(build(&test_agent()).is_none());
    }

    #[test]
    fn limits_memory_and_cpu() {
        let mut agent = test_agent();
        agent.spec.resources = Some(
            ResourceRequirements::builder()
                .limits(
                    ResourceLimits::builder()
                        .memory("512Mi".to_string())
                        .cpu("500m".to_string())
                        .build(),
                )
                .build(),
        );

        let resources = build(&agent).unwrap();
        let limits = resources.limits.unwrap();
        assert_eq!(limits["memory"].0, "512Mi");
        assert_eq!(limits["cpu"].0, "500m");
    }

    #[test]
    fn requests_memory_only() {
        let mut agent = test_agent();
        agent.spec.resources = Some(
            ResourceRequirements::builder()
                .requests(
                    ResourceLimits::builder()
                        .memory("256Mi".to_string())
                        .build(),
                )
                .build(),
        );

        let resources = build(&agent).unwrap();
        let requests = resources.requests.unwrap();
        assert_eq!(requests["memory"].0, "256Mi");
        assert!(requests.get("cpu").is_none());
    }

    #[test]
    fn limits_to_quantity_map_empty_fields() {
        let limits = ResourceLimits::builder().build();
        let map = limits_to_quantity_map(&limits);
        assert!(map.is_empty());
    }

    #[test]
    fn limits_to_quantity_map_cpu_only() {
        let limits = ResourceLimits::builder()
            .cpu("250m".to_string())
            .build();
        let map = limits_to_quantity_map(&limits);
        assert_eq!(map.len(), 1);
        assert_eq!(map["cpu"].0, "250m");
    }
}
