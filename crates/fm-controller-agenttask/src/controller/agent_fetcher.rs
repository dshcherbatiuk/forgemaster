//! Fetches Agent CRs from Kubernetes using `DynamicObject`.
//!
//! Uses dynamic typing to avoid a circular dependency on `fm-controller-agent`.

use kube::Client;
use kube::api::{Api, ApiResource, DynamicObject, ListParams};
use tracing::warn;

use crate::agent_info::{AgentInfo, AgentInfoList};

/// Agent CRD group.
const AGENT_GROUP: &str = "forgemaster.io";
/// Agent CRD version.
const AGENT_VERSION: &str = "v1alpha1";
/// Agent CRD plural name.
const AGENT_PLURAL: &str = "agents";

/// Fetches Agent CRs for a given task using `DynamicObject`.
pub struct AgentFetcher;

impl AgentFetcher {
    /// Lists agents in the task's namespace and extracts summary info.
    ///
    /// Agents live in namespace `<task_name>` with label `forgemaster.io/task=<task_name>`.
    /// Returns an empty list on error (graceful degradation).
    pub async fn list_for_task(client: &Client, task_name: &str) -> AgentInfoList {
        let api_resource = ApiResource::from_gvk_with_plural(&Self::gvk(), AGENT_PLURAL);

        let api: Api<DynamicObject> =
            Api::namespaced_with(client.clone(), task_name, &api_resource);

        let label_selector = format!("forgemaster.io/task={task_name}");
        let list_params = ListParams::default().labels(&label_selector);

        match api.list(&list_params).await {
            Ok(agent_list) => agent_list.items.iter().map(Self::extract_info).collect(),
            Err(err) => {
                warn!("⚠️ Failed to list agents for task {task_name}: {err}");
                AgentInfoList::new()
            }
        }
    }

    /// Extracts agent summary from a `DynamicObject`.
    fn extract_info(obj: &DynamicObject) -> AgentInfo {
        let name = obj.metadata.name.clone().unwrap_or_default();
        let data = &obj.data;

        let agent_type = data["spec"]["type"]
            .as_str()
            .unwrap_or("unknown")
            .to_string();

        let phase = data["status"]["phase"]
            .as_str()
            .unwrap_or("Pending")
            .to_string();

        AgentInfo {
            name,
            agent_type,
            phase,
        }
    }

    fn gvk() -> kube::api::GroupVersionKind {
        kube::api::GroupVersionKind::gvk(AGENT_GROUP, AGENT_VERSION, "Agent")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use kube::core::ObjectMeta;
    use serde_json::json;

    fn dynamic_agent(name: &str, agent_type: &str, phase: &str) -> DynamicObject {
        let api_resource = ApiResource::from_gvk_with_plural(&AgentFetcher::gvk(), AGENT_PLURAL);

        let mut obj = DynamicObject::new(name, &api_resource);
        obj.metadata = ObjectMeta {
            name: Some(name.to_string()),
            ..Default::default()
        };
        obj.data = json!({
            "spec": { "type": agent_type },
            "status": { "phase": phase }
        });
        obj
    }

    #[test]
    fn extract_info_from_dynamic_object() {
        let obj = dynamic_agent("orchestrator-task-abc", "orchestrator", "Running");
        let info = AgentFetcher::extract_info(&obj);

        assert_eq!(info.name, "orchestrator-task-abc");
        assert_eq!(info.agent_type, "orchestrator");
        assert_eq!(info.phase, "Running");
    }

    #[test]
    fn extract_info_missing_status_defaults_to_pending() {
        let api_resource = ApiResource::from_gvk_with_plural(&AgentFetcher::gvk(), AGENT_PLURAL);

        let mut obj = DynamicObject::new("agent-no-status", &api_resource);
        obj.metadata.name = Some("agent-no-status".to_string());
        obj.data = json!({ "spec": { "type": "code-generator" } });

        let info = AgentFetcher::extract_info(&obj);
        assert_eq!(info.phase, "Pending");
    }

    #[test]
    fn extract_info_missing_type_defaults_to_unknown() {
        let api_resource = ApiResource::from_gvk_with_plural(&AgentFetcher::gvk(), AGENT_PLURAL);

        let mut obj = DynamicObject::new("agent-no-type", &api_resource);
        obj.metadata.name = Some("agent-no-type".to_string());
        obj.data = json!({ "spec": {}, "status": { "phase": "Failed" } });

        let info = AgentFetcher::extract_info(&obj);
        assert_eq!(info.agent_type, "unknown");
        assert_eq!(info.phase, "Failed");
    }

    #[test]
    fn extract_info_empty_data() {
        let api_resource = ApiResource::from_gvk_with_plural(&AgentFetcher::gvk(), AGENT_PLURAL);

        let mut obj = DynamicObject::new("empty", &api_resource);
        obj.metadata.name = Some("empty".to_string());
        obj.data = json!({});

        let info = AgentFetcher::extract_info(&obj);
        assert_eq!(info.name, "empty");
        assert_eq!(info.agent_type, "unknown");
        assert_eq!(info.phase, "Pending");
    }

    #[test]
    fn gvk_matches_agent_crd() {
        let gvk = AgentFetcher::gvk();
        assert_eq!(gvk.group, "forgemaster.io");
        assert_eq!(gvk.version, "v1alpha1");
        assert_eq!(gvk.kind, "Agent");
    }
}
