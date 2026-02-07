//! AgentTask Controller binary.

use anyhow::Result;
use fm_controller_agenttask::task_event::TaskEvent;
use fm_controller_agenttask::ws::{TaskStateBroadcaster, WsServer};
use kube::Client;
use tokio::sync::broadcast;
use tracing::info;
use tracing_subscriber::{EnvFilter, fmt, prelude::*};

const DEFAULT_NAMESPACE: &str = "forgemaster-system";
const DEFAULT_WS_PORT: u16 = 8080;
const STATE_CHANNEL_CAPACITY: usize = 64;

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::registry()
        .with(fmt::layer())
        .with(EnvFilter::from_default_env().add_directive("info".parse()?))
        .init();

    info!("🔧 Initializing AgentTask Controller");

    let namespace = std::env::var("NAMESPACE").unwrap_or_else(|_| DEFAULT_NAMESPACE.to_string());

    let ws_port: u16 = std::env::var("WS_PORT")
        .unwrap_or_else(|_| DEFAULT_WS_PORT.to_string())
        .parse()?;

    let client = Client::try_default().await?;
    info!("📡 Connected to Kubernetes cluster");

    let (state_sender, state_receiver) = broadcast::channel::<TaskEvent>(STATE_CHANNEL_CAPACITY);

    let ws_server = WsServer::new(ws_port, client.clone(), namespace.clone());

    let broadcaster = TaskStateBroadcaster::new(
        state_receiver,
        ws_server.schema_cache(),
        ws_server.registry(),
    );

    tokio::select! {
        result = fm_controller_agenttask::controller::run(client, &namespace, state_sender) => {
            result?;
        }
        result = ws_server.run() => {
            result?;
        }
        _ = broadcaster.run() => {}
    }

    Ok(())
}
