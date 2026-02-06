//! AgentTask Controller binary.

use anyhow::Result;
use kube::Client;
use tracing::info;
use tracing_subscriber::{fmt, prelude::*, EnvFilter};

const DEFAULT_NAMESPACE: &str = "forgemaster-system";

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize tracing
    tracing_subscriber::registry()
        .with(fmt::layer())
        .with(EnvFilter::from_default_env().add_directive("info".parse()?))
        .init();

    info!("🔧 Initializing AgentTask Controller");

    // Get namespace from environment or use default
    let namespace = std::env::var("NAMESPACE").unwrap_or_else(|_| DEFAULT_NAMESPACE.to_string());

    // Create Kubernetes client
    let client = Client::try_default().await?;

    info!("📡 Connected to Kubernetes cluster");

    // Run the controller
    fm_controller_agenttask::controller::run(client, &namespace).await?;

    Ok(())
}
