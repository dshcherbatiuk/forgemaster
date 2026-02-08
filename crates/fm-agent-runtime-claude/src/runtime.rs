//! Long-lived agent runtime.
//!
//! Sends the initial prompt to Claude via the conversation loop,
//! executing tool calls through MCP servers when configured.

use anyhow::Result;
use tracing::info;

use crate::claude_api::client::ClaudeClient;
use crate::claude_api::request::Message;
use crate::config::RuntimeConfig;
use crate::conversation_loop::{self, ConversationLoopConfig};
use crate::output_writer::OutputWriter;
use crate::status_updater::StatusUpdater;
use crate::tool_executor::{CompositeToolExecutor, NoOpToolExecutor, ToolExecutor};

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

        // 2. Create tool executor (composite or no-op)
        let executor: Box<dyn ToolExecutor> = if self.config.mcp_server_urls.is_empty() {
            Box::new(NoOpToolExecutor)
        } else {
            Box::new(
                CompositeToolExecutor::connect(&self.config.mcp_server_urls).await?,
            )
        };

        // 3. Build conversation loop config
        let loop_config = ConversationLoopConfig {
            model: self.config.model_name.clone(),
            max_tokens: self.config.model_max_tokens,
            system: Some(self.config.task_prompt.clone()),
            temperature: Some(self.config.model_temperature),
            max_iterations: self.config.max_tool_iterations,
        };

        // 4. Run conversation loop
        let client = ClaudeClient::new(&self.config.api_key, &self.config.api_base_url);
        let initial_messages =
            vec![Message::user("Execute the task described in the system prompt.")];

        let result =
            conversation_loop::run(&client, executor.as_ref(), &loop_config, initial_messages)
                .await?;

        // 5. Log the output
        output_writer.write(&result.final_text);

        // 5. Transition to Succeeded with token metrics
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

        // 6. Stay alive — MCP/A2A communication will be handled here
        tokio::signal::ctrl_c().await?;
        info!("🛑 Agent {} received shutdown signal", self.config.agent_name);

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
        assert_eq!(loop_config.max_iterations, 25);
    }
}
