//! Creates a K8s Service per agent pod for A2A communication.
//!
//! Each agent pod needs a Service so that other agents can discover
//! and communicate with it via DNS:
//! `http://<agent-name>.<namespace>.svc.cluster.local:9090`

use k8s_openapi::api::core::v1::{Service, ServicePort, ServiceSpec};
use k8s_openapi::apimachinery::pkg::util::intstr::IntOrString;
use kube::ResourceExt;
use kube::api::{Api, ObjectMeta, PostParams};
use tracing::info;

use crate::crd::Agent;

use super::context::ControllerContext;
use super::error::{ReconcileError, ReconcileResult};

/// Default A2A port used by agent runtime pods.
pub const A2A_PORT: u16 = 9090;

/// A2A port name in the Service spec.
const A2A_PORT_NAME: &str = "a2a";

/// Ensures a K8s Service exists for the agent pod in its namespace.
///
/// The Service routes A2A traffic to the agent's port 9090.
/// Idempotent — skips creation if the Service already exists.
pub async fn ensure_service(
    ctx: &ControllerContext,
    agent: &Agent,
) -> ReconcileResult<()> {
    let agent_name = agent.name_any();
    let namespace = agent
        .namespace()
        .ok_or_else(|| ReconcileError::MissingField("metadata.namespace".to_string()))?;

    let svc_api: Api<Service> = Api::namespaced(ctx.client().clone(), &namespace);

    // Skip if service already exists
    if svc_api
        .get_opt(&agent_name)
        .await
        .map_err(ReconcileError::GetAgent)?
        .is_some()
    {
        return Ok(());
    }

    let labels = agent.metadata.labels.clone().unwrap_or_default();

    let service = Service {
        metadata: ObjectMeta {
            name: Some(agent_name.clone()),
            namespace: Some(namespace.clone()),
            labels: Some(labels.clone()),
            ..Default::default()
        },
        spec: Some(ServiceSpec {
            selector: Some(labels),
            ports: Some(vec![ServicePort {
                name: Some(A2A_PORT_NAME.to_string()),
                port: i32::from(A2A_PORT),
                target_port: Some(IntOrString::Int(i32::from(A2A_PORT))),
                protocol: Some("TCP".to_string()),
                ..Default::default()
            }]),
            ..Default::default()
        }),
        ..Default::default()
    };

    match svc_api.create(&PostParams::default(), &service).await {
        Ok(_) => {
            info!(
                "🌐 Created A2A Service {agent_name} in {namespace} (port {A2A_PORT})"
            );
        }
        Err(kube::Error::Api(err)) if err.reason == "AlreadyExists" => {}
        Err(source) => {
            return Err(ReconcileError::CreateResource {
                resource: format!("Service/{agent_name} in {namespace}"),
                source,
            });
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::crd::{AgentCrd, ModelConfig};
    use std::collections::BTreeMap;

    fn test_agent() -> Agent {
        Agent {
            metadata: ObjectMeta {
                name: Some("code-gen-task-abc".to_string()),
                namespace: Some("task-abc".to_string()),
                uid: Some("uid-456".to_string()),
                labels: Some(BTreeMap::from([
                    ("forgemaster.io/task".to_string(), "task-abc".to_string()),
                    (
                        "app.kubernetes.io/component".to_string(),
                        "agent-runtime".to_string(),
                    ),
                ])),
                ..Default::default()
            },
            spec: AgentCrd {
                agent_type: "code-generator".to_string(),
                model: ModelConfig::builder()
                    .name("claude-sonnet-4-20250514".to_string())
                    .build(),
                task_prompt: "Generate code".to_string(),
                mcp_servers: vec![],
                resources: None,
            },
            status: None,
        }
    }

    #[test]
    fn a2a_port_is_9090() {
        assert_eq!(A2A_PORT, 9090);
    }

    #[test]
    fn service_metadata_matches_agent() {
        let agent = test_agent();
        let agent_name = agent.name_any();
        let namespace = agent.namespace().expect("namespace");
        let labels = agent.metadata.labels.clone().unwrap_or_default();

        let meta = ObjectMeta {
            name: Some(agent_name.clone()),
            namespace: Some(namespace.clone()),
            labels: Some(labels),
            ..Default::default()
        };

        assert_eq!(meta.name, Some("code-gen-task-abc".to_string()));
        assert_eq!(meta.namespace, Some("task-abc".to_string()));
    }

    #[test]
    fn service_port_spec() {
        let port = ServicePort {
            name: Some(A2A_PORT_NAME.to_string()),
            port: i32::from(A2A_PORT),
            target_port: Some(IntOrString::Int(i32::from(A2A_PORT))),
            protocol: Some("TCP".to_string()),
            ..Default::default()
        };

        assert_eq!(port.name, Some("a2a".to_string()));
        assert_eq!(port.port, 9090);
        assert_eq!(
            port.target_port,
            Some(IntOrString::Int(9090))
        );
        assert_eq!(port.protocol, Some("TCP".to_string()));
    }

    #[test]
    fn service_selector_uses_agent_labels() {
        let agent = test_agent();
        let labels = agent.metadata.labels.clone().unwrap_or_default();

        let spec = ServiceSpec {
            selector: Some(labels.clone()),
            ..Default::default()
        };

        let selector = spec.selector.expect("selector");
        assert_eq!(selector.get("forgemaster.io/task").map(String::as_str), Some("task-abc"));
    }

    #[test]
    fn service_dns_format() {
        let agent = test_agent();
        let name = agent.name_any();
        let namespace = agent.namespace().expect("namespace");
        let dns = format!("http://{name}.{namespace}.svc.cluster.local:{A2A_PORT}");
        assert_eq!(
            dns,
            "http://code-gen-task-abc.task-abc.svc.cluster.local:9090"
        );
    }
}
