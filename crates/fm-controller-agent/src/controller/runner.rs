//! Controller runner for Agent reconciliation.

use std::sync::Arc;

use futures::StreamExt;
use kube::api::Api;
use kube::runtime::Controller;
use kube::runtime::watcher::Config as WatcherConfig;
use kube::Client;
use tracing::info;

use crate::crd::Agent;

use super::context::create_context;
use super::dispatcher::Dispatcher;

/// Runs the Agent controller.
pub async fn run(client: Client, namespace: &str) -> anyhow::Result<()> {
    info!("🚀 Starting Agent controller in namespace: {}", namespace);

    let ctx = create_context(client.clone(), namespace.to_string());
    let dispatcher = Arc::new(Dispatcher::new(Arc::clone(&ctx)));

    let api: Api<Agent> = Api::namespaced(client, namespace);

    Controller::new(api, WatcherConfig::default())
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
        })
        .await;

    Ok(())
}

#[cfg(test)]
mod tests {
    #[test]
    fn runner_module_exists() {
        assert!(true);
    }
}
