//! Agent Runtime binary.
//!
//! Reads the Agent CR, calls Claude API, logs output, and updates status.

use anyhow::Result;
use tracing::{error, info};
use tracing_subscriber::{EnvFilter, fmt, prelude::*};

use fm_agent_runtime_claude::config::RuntimeConfig;
use fm_agent_runtime_claude::runtime::AgentRuntime;
use fm_agent_runtime_claude::status_updater::StatusUpdater;

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::registry()
        .with(fmt::layer())
        .with(EnvFilter::from_default_env().add_directive("info".parse()?))
        .init();

    info!("🚀 Starting Agent Runtime");

    let config = RuntimeConfig::from_env()?;
    info!(
        "📦 Agent: {}, Namespace: {}",
        config.agent_name, config.namespace
    );

    let k8s_client = kube::Client::try_default().await?;

    let runtime = AgentRuntime::new(config.clone(), k8s_client.clone());

    tokio::select! {
        result = runtime.run() => {
            if let Err(err) = &result {
                error!("❌ Agent runtime failed: {err:#}");
                attempt_failure_status(&config, &k8s_client, err).await;
                return result;
            }
            info!("🏁 Agent Runtime finished");
        }
        _ = tokio::signal::ctrl_c() => {
            info!("🛑 Received shutdown signal");
        }
    }

    Ok(())
}

/// Best-effort attempt to mark the agent as Failed on error.
async fn attempt_failure_status(config: &RuntimeConfig, client: &kube::Client, err: &anyhow::Error) {
    let updater = StatusUpdater::new(client.clone(), &config.namespace, &config.agent_name);
    if let Err(status_err) = updater
        .transition_to_failed("RuntimeError", &format!("{err:#}"))
        .await
    {
        error!("⚠️ Could not update status to Failed: {status_err:#}");
    }
}
