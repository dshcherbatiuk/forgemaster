//! Agent Controller binary.

use anyhow::Result;
use kube::Client;
use tracing::info;
use tracing_subscriber::{EnvFilter, fmt, prelude::*};

const DEFAULT_NAMESPACE: &str = "forgemaster-system";

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::registry()
        .with(fmt::layer())
        .with(EnvFilter::from_default_env().add_directive("info".parse()?))
        .init();

    info!("🔧 Initializing Agent Controller");

    let namespace = std::env::var("NAMESPACE").unwrap_or_else(|_| DEFAULT_NAMESPACE.to_string());

    let client = Client::try_default().await?;
    info!("📡 Connected to Kubernetes cluster");

    fm_controller_agent::controller::run(client, &namespace).await?;

    Ok(())
}
