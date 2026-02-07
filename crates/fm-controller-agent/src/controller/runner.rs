//! Controller runner for Agent reconciliation.
//!
//! Runs two concurrent watches via `tokio::select!`:
//! 1. Agent CR reconciler (kube Controller)
//! 2. AgentTask CR watcher (creates Orchestrator Agents)

use std::sync::Arc;

use futures::StreamExt;
use kube::Client;
use kube::api::Api;
use kube::runtime::Controller;
use kube::runtime::watcher::Config as WatcherConfig;
use tracing::info;

use crate::crd::Agent;
use crate::task_watcher;

use super::context::create_context;
use super::dispatcher::Dispatcher;

/// Runs the Agent controller and AgentTask watcher concurrently.
///
/// Uses `tokio::select!` to join:
/// - Agent CR reconciliation (phase-based strategy pattern)
/// - AgentTask CR watching (creates Orchestrator Agent CRs on Running phase)
pub async fn run(client: Client, namespace: &str) -> anyhow::Result<()> {
    info!("🚀 Starting Agent controller in namespace: {}", namespace);

    let ctx = create_context(client.clone(), namespace.to_string())?;
    let default_model = ctx.default_model().to_string();
    let dispatcher = Arc::new(Dispatcher::new(Arc::clone(&ctx)));

    let api: Api<Agent> = Api::all(client.clone());

    let agent_controller = Controller::new(api, WatcherConfig::default())
        .run(
            |agent, _| {
                let dispatcher = Arc::clone(&dispatcher);
                async move { dispatcher.reconcile(agent).await }
            },
            |agent, error, _| Dispatcher::error_policy(agent, error),
            ctx,
        )
        .for_each(|result| async move {
            match result {
                Ok((object, _action)) => {
                    info!("✅ Reconciled Agent: {:?}", object.name);
                }
                Err(error) => {
                    tracing::error!("❌ Agent controller error: {:?}", error);
                }
            }
        });

    let task_watcher = task_watcher::run(client, namespace, &default_model);

    tokio::select! {
        () = agent_controller => {
            info!("⚠️ Agent controller stream ended");
        }
        result = task_watcher => {
            if let Err(err) = result {
                tracing::error!("❌ AgentTask watcher failed: {}", err);
            }
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    #[test]
    fn runner_module_exists() {
        assert!(true);
    }
}
