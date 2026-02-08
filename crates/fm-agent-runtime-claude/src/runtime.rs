//! Long-lived agent runtime.
//!
//! Sends the initial prompt to Claude via the conversation loop,
//! executing tool calls through MCP servers when configured.
//! Runs an embedded A2A server for inter-agent communication.

use anyhow::Result;
use tracing::info;

use std::sync::Arc;

use crate::a2a::client::A2aToolExecutor;
use crate::a2a::config::A2aConfig;
use crate::a2a::server::conversation_deps::ConversationDeps;
use crate::audit_logger::AuditLogger;
use crate::claude_api::client::ClaudeClient;
use crate::claude_api::request::Message;
use crate::config::RuntimeConfig;
use crate::conversation_loop::{self, ConversationLoopConfig};
use crate::output_writer::OutputWriter;
use crate::status_updater::StatusUpdater;
use crate::tool_executor::{CompositeToolExecutor, ToolExecutor};

/// Long-lived agent runtime.
///
/// Sends the initial prompt, then stays alive for MCP/A2A communication.
pub struct AgentRuntime {
    config: RuntimeConfig,
    k8s_client: kube::Client,
}

impl AgentRuntime {
    /// Creates a new runtime for the given config and K8s client.
    pub fn new(config: RuntimeConfig, k8s_client: kube::Client) -> Self {
        Self { config, k8s_client }
    }

    /// Runs the agent: sends initial prompt with tools, then stays alive.
    pub async fn run(&self) -> Result<()> {
        let status_updater = StatusUpdater::new(
            self.k8s_client.clone(),
            &self.config.namespace,
            &self.config.agent_name,
        );
        let output_writer = OutputWriter::new(&self.config.agent_name);

        info!(
            "📋 Agent: {}, model: {}, MCP servers: {}",
            self.config.agent_name,
            self.config.model_name,
            if self.config.mcp_server_urls.is_empty() {
                "none".to_string()
            } else {
                self.config.mcp_server_urls.len().to_string()
            }
        );

        // 1. Transition to Running
        status_updater.transition_to_running().await?;

        // 2. Create tool executor (MCP servers + A2A tools)
        let mut composite = if self.config.mcp_server_urls.is_empty() {
            CompositeToolExecutor::empty()
        } else {
            CompositeToolExecutor::connect(&self.config.mcp_server_urls).await?
        };

        // Always add A2A tools for inter-agent communication
        composite.add(Box::new(A2aToolExecutor::new()));

        let executor: Arc<dyn ToolExecutor> = Arc::new(composite);

        // 3. Build conversation loop config
        let loop_config = Arc::new(ConversationLoopConfig {
            model: self.config.model_name.clone(),
            max_tokens: self.config.model_max_tokens,
            system: Some(self.config.task_prompt.clone()),
            temperature: Some(self.config.model_temperature),
            max_iterations: self.config.max_tool_iterations,
        });

        let client = Arc::new(ClaudeClient::new(
            &self.config.api_key,
            &self.config.api_base_url,
        ));

        // 4. Start A2A server FIRST so peers can reach this agent immediately
        let a2a_config = A2aConfig::from_env();

        info!(
            "🌐 Starting A2A server (type={}, port={}, peers={})",
            a2a_config.agent_type,
            a2a_config.port,
            a2a_config.peer_urls.len()
        );

        // Create audit logger when workspace dir is configured
        let audit_logger = self
            .config
            .workspace_dir
            .as_ref()
            .map(|dir| AuditLogger::new(dir, &self.config.agent_name))
            .transpose()?;

        let conversation_deps = ConversationDeps {
            client: client.clone(),
            executor: executor.clone(),
            loop_config: loop_config.clone(),
            audit_logger: audit_logger.clone(),
        };

        let agent_name_for_server = self.config.agent_name.clone();
        tokio::spawn(async move {
            if let Err(e) = crate::a2a::server::start(
                &agent_name_for_server,
                &a2a_config.agent_type,
                a2a_config.port,
                Some(conversation_deps),
            )
            .await
            {
                tracing::error!("🌐 A2A server error: {e}");
            }
        });

        // 5. Run conversation loop
        let initial_messages =
            vec![Message::user("Execute the task described in the system prompt.")];

        let result = conversation_loop::run(
            &client,
            executor.as_ref(),
            &loop_config,
            initial_messages,
            audit_logger.as_ref(),
        )
        .await?;

        // 6. Log the output
        output_writer.write(&result.final_text);

        // 7. Transition to Succeeded with token metrics
        status_updater
            .transition_to_succeeded(
                result.total_usage.total(),
                result.iterations as i32,
            )
            .await?;

        info!(
            "🧠 Agent {} done — {} iteration(s), {} tokens, waiting for requests",
            self.config.agent_name,
            result.iterations,
            result.total_usage.total()
        );

        // 8. Stay alive for A2A communication (server already running in background)
        tokio::signal::ctrl_c().await?;
        info!(
            "🛑 Agent {} received shutdown signal",
            self.config.agent_name
        );

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use crate::conversation_loop::ConversationLoopConfig;

    #[test]
    fn loop_config_from_runtime_config() {
        let runtime_config = crate::config::RuntimeConfig::new(
            "agent".to_string(),
            "ns".to_string(),
            "key".to_string(),
            "You are an agent".to_string(),
            "claude-sonnet-4-20250514".to_string(),
        );

        let loop_config = ConversationLoopConfig {
            model: runtime_config.model_name.clone(),
            max_tokens: runtime_config.model_max_tokens,
            system: Some(runtime_config.task_prompt.clone()),
            temperature: Some(runtime_config.model_temperature),
            max_iterations: runtime_config.max_tool_iterations,
        };

        assert_eq!(loop_config.model, "claude-sonnet-4-20250514");
        assert_eq!(loop_config.max_tokens, 4096);
        assert_eq!(loop_config.system.as_deref(), Some("You are an agent"));
        assert_eq!(loop_config.max_iterations, 50);
    }
}
