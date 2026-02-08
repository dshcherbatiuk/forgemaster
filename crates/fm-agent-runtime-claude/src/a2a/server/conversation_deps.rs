//! Shared dependencies for running a conversation loop from the A2A handler.

use std::sync::Arc;

use crate::claude_api::client::ClaudeClient;
use crate::conversation_loop::ConversationLoopConfig;
use crate::tool_executor::ToolExecutor;

/// Shared dependencies for running a conversation loop from the A2A handler.
///
/// Wraps the Claude client, tool executor, and loop config in `Arc`
/// because multiple concurrent A2A requests may invoke the conversation
/// loop simultaneously (`tokio::spawn` requires `'static`).
///
/// Stores `workspace_dir` instead of a shared `AuditLogger` so each
/// A2A conversation creates its own audit file (preventing truncation
/// of the main conversation's audit log).
#[derive(Clone)]
pub struct ConversationDeps {
    /// Claude API client for sending messages.
    pub client: Arc<ClaudeClient>,
    /// Tool executor (MCP servers + A2A tools).
    pub executor: Arc<dyn ToolExecutor>,
    /// Conversation loop configuration (model, max_tokens, system prompt).
    pub loop_config: Arc<ConversationLoopConfig>,
    /// Workspace directory for creating per-A2A-task audit loggers.
    pub workspace_dir: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tool_executor::NoOpToolExecutor;

    fn test_deps() -> ConversationDeps {
        ConversationDeps {
            client: Arc::new(ClaudeClient::new("test-key", "http://localhost")),
            executor: Arc::new(NoOpToolExecutor),
            loop_config: Arc::new(ConversationLoopConfig::new(
                "claude-sonnet-4-20250514".to_string(),
                4096,
            )),
            workspace_dir: None,
        }
    }

    #[test]
    fn clone_shares_same_arcs() {
        let deps = test_deps();
        let cloned = deps.clone();

        assert!(Arc::ptr_eq(&deps.client, &cloned.client));
        assert!(Arc::ptr_eq(&deps.loop_config, &cloned.loop_config));
    }

    #[test]
    fn fields_are_accessible() {
        let deps = test_deps();
        assert_eq!(deps.loop_config.max_tokens, 4096);
    }
}
