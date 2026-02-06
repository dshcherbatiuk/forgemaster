//! Controller runner for AgentTask reconciliation.

use std::sync::Arc;

use futures::StreamExt;
use kube::api::Api;
use kube::runtime::watcher::Config as WatcherConfig;
use kube::runtime::Controller;
use kube::Client;
use tracing::info;

use crate::crd::AgentTask;

use super::context::create_context;
use super::dispatcher::Dispatcher;

/// Runs the AgentTask controller.
pub async fn run(client: Client, namespace: &str) -> anyhow::Result<()> {
    info!("🚀 Starting AgentTask controller in namespace: {}", namespace);

    let ctx = create_context(client.clone(), namespace.to_string());
    let dispatcher = Arc::new(Dispatcher::new(Arc::clone(&ctx)));

    let api: Api<AgentTask> = Api::namespaced(client, namespace);

    Controller::new(api, WatcherConfig::default())
        .run(
            |task, _| {
                let dispatcher = Arc::clone(&dispatcher);
                async move { dispatcher.reconcile(task).await }
            },
            |task, error, _| Dispatcher::error_policy(task, error),
            ctx,
        )
        .for_each(|result| async move {
            match result {
                Ok((object, _action)) => {
                    info!("✅ Reconciled: {:?}", object.name);
                }
                Err(error) => {
                    tracing::error!("❌ Controller error: {:?}", error);
                }
            }
        })
        .await;

    Ok(())
}

#[cfg(test)]
mod tests {
    #[test]
    fn runner_module_exists() {
        // Integration tests would require a real cluster.
        // This test verifies the module compiles correctly.
        assert!(true);
    }
}
