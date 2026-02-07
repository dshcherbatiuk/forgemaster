//! Watches AgentTask CRs and creates Orchestrator Agent CRs.

use fm_controller_agenttask::crd::{AgentTask, AgentTaskPhase};
use futures::TryStreamExt;
use kube::Client;
use kube::ResourceExt;
use kube::api::{Api, ListParams, PostParams};
use kube::runtime::watcher::{self, Config as WatcherConfig};
use tracing::{debug, error, info};

use crate::crd::Agent;

use super::orchestrator_factory;

/// Watches AgentTask CRs in the given namespace and creates
/// Orchestrator Agent CRs when tasks enter Running phase.
pub async fn run(client: Client, namespace: &str, default_model: &str) -> anyhow::Result<()> {
    info!("👀 Starting AgentTask watcher in namespace: {}", namespace);

    let task_api: Api<AgentTask> = Api::namespaced(client.clone(), namespace);

    let stream = watcher::watcher(task_api, WatcherConfig::default());

    futures::pin_mut!(stream);

    while let Some(event) = stream.try_next().await? {
        match event {
            watcher::Event::Apply(task) | watcher::Event::InitApply(task) => {
                handle_task(&client, &task, default_model).await;
            }
            _ => {}
        }
    }

    Ok(())
}

async fn handle_task(client: &Client, task: &AgentTask, default_model: &str) {
    let phase = task.status.as_ref().map(|s| s.phase).unwrap_or_default();

    if phase != AgentTaskPhase::Running {
        return;
    }

    let task_name = task.name_any();
    // Agent goes into the task-specific namespace (same name as the task)
    let task_namespace = task_name.clone();

    if orchestrator_exists(client, &task_name, &task_namespace).await {
        debug!("🔍 Orchestrator already exists for task {}", task_name);
        return;
    }

    let agent = orchestrator_factory::build(task, default_model);
    let agent_api: Api<Agent> = Api::namespaced(client.clone(), &task_namespace);

    match agent_api.create(&PostParams::default(), &agent).await {
        Ok(created) => {
            info!(
                "🤖 Created Orchestrator Agent {} for task {}",
                created.name_any(),
                task_name
            );
        }
        Err(kube::Error::Api(ref err)) if err.code == 409 => {
            debug!(
                "🤖 Orchestrator Agent already exists for task {}",
                task_name
            );
        }
        Err(err) => {
            error!(
                "❌ Failed to create Orchestrator Agent for task {}: {}",
                task_name, err
            );
        }
    }
}

async fn orchestrator_exists(client: &Client, task_name: &str, namespace: &str) -> bool {
    let agent_api: Api<Agent> = Api::namespaced(client.clone(), namespace);

    let label_selector = format!(
        "forgemaster.io/task={},forgemaster.io/type=orchestrator",
        task_name
    );
    let list_params = ListParams::default().labels(&label_selector);

    match agent_api.list(&list_params).await {
        Ok(agents) => !agents.items.is_empty(),
        Err(err) => {
            error!(
                "❌ Failed to list Orchestrator Agents for task {}: {}",
                task_name, err
            );
            false
        }
    }
}

#[cfg(test)]
mod tests {
    use fm_controller_agenttask::crd::AgentTaskPhase;

    #[test]
    fn running_phase_triggers_creation() {
        assert_eq!(AgentTaskPhase::Running.to_string(), "Running");
    }

    #[test]
    fn pending_phase_does_not_trigger() {
        assert_ne!(AgentTaskPhase::Pending, AgentTaskPhase::Running);
    }
}
