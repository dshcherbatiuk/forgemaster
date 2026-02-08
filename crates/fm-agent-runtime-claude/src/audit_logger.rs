//! Audit logger for agent conversations.
//!
//! Writes every prompt sent to the LLM and every response received
//! to a single markdown file per agent at `{workspace_dir}/audit/{agent_name}.md`.

use std::fmt::Write as FmtWrite;
use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::PathBuf;

use anyhow::{Context, Result};
use tracing::debug;

use crate::claude_api::content_block::RequestContentBlock;
use crate::claude_api::request::{Message, MessageContent};
use crate::claude_api::response::Usage;
use crate::claude_api::tool_use_block::ToolUseBlock;

/// Appends audit entries (prompts and responses) to a markdown file.
///
/// One file per agent: `{workspace_dir}/audit/{agent_name}.md`.
/// All writes use append mode — safe for incremental logging.
#[derive(Debug, Clone)]
pub struct AuditLogger {
    file_path: PathBuf,
    agent_name: String,
}

impl AuditLogger {
    /// Creates a new audit logger.
    ///
    /// Creates `{workspace_dir}/audit/` directory if it doesn't exist.
    ///
    /// # Errors
    ///
    /// Returns an error if the audit directory cannot be created.
    pub fn new(workspace_dir: &str, agent_name: &str) -> Result<Self> {
        let audit_dir = PathBuf::from(workspace_dir).join("audit");
        fs::create_dir_all(&audit_dir)
            .with_context(|| format!("failed to create audit directory: {}", audit_dir.display()))?;

        let file_path = audit_dir.join(format!("{agent_name}.md"));

        debug!("📋 Audit logger initialized: {}", file_path.display());

        Ok(Self {
            file_path,
            agent_name: agent_name.to_string(),
        })
    }

    /// Writes the audit file header with agent name, model, and system prompt.
    ///
    /// # Errors
    ///
    /// Returns an error if the file cannot be written.
    pub fn log_header(&self, model: &str, system_prompt: Option<&str>) -> Result<()> {
        let mut content = format!(
            "# Audit Log: {}\n\n**Started**: {}  \n**Model**: {}\n",
            self.agent_name,
            chrono::Utc::now().to_rfc3339(),
            model,
        );

        if let Some(prompt) = system_prompt {
            content.push_str("\n### System Prompt\n\n");
            content.push_str(prompt);
            content.push('\n');
        }

        content.push_str("\n---\n");
        self.truncate_and_write(&content)
    }

    /// Logs the request messages for an iteration.
    ///
    /// Extracts text from the last user message (the new input for this iteration).
    ///
    /// # Errors
    ///
    /// Returns an error if the file cannot be written.
    pub fn log_request(&self, iteration: u32, messages: &[Message]) -> Result<()> {
        let timestamp = chrono::Utc::now().to_rfc3339();
        let mut content =
            format!("\n## Iteration {iteration}\n\n### Request\n\n**Timestamp**: {timestamp}\n\n");

        if let Some(last_message) = messages.last() {
            content.push_str(&extract_message_text(last_message));
        }

        content.push('\n');
        self.append(&content)
    }

    /// Logs Claude's response for an iteration.
    ///
    /// # Errors
    ///
    /// Returns an error if the file cannot be written.
    pub fn log_response(
        &self,
        iteration: u32,
        text: &str,
        tool_calls: &[ToolUseBlock],
        usage: &Usage,
    ) -> Result<()> {
        let timestamp = chrono::Utc::now().to_rfc3339();
        let mut content = format!(
            "\n### Response (iteration {iteration})\n\n**Timestamp**: {timestamp}  \n**Tokens**: {} input / {} output\n",
            usage.input_tokens, usage.output_tokens,
        );

        if !text.is_empty() {
            content.push('\n');
            content.push_str(text);
            content.push('\n');
        }

        if !tool_calls.is_empty() {
            content.push_str("\n### Tool Calls\n");
            for tool in tool_calls {
                let _ = write!(content, "\n#### `{}`\n\n", tool.name);
                content.push_str("**Input**:\n```json\n");
                content.push_str(
                    &serde_json::to_string_pretty(&tool.input).unwrap_or_default(),
                );
                content.push_str("\n```\n");
            }
        }

        self.append(&content)
    }

    /// Logs tool execution results for an iteration.
    ///
    /// # Errors
    ///
    /// Returns an error if the file cannot be written.
    pub fn log_tool_results(
        &self,
        iteration: u32,
        results: &[RequestContentBlock],
    ) -> Result<()> {
        let timestamp = chrono::Utc::now().to_rfc3339();
        let mut content = format!("\n### Tool Results (iteration {iteration})\n\n**Timestamp**: {timestamp}\n");

        for block in results {
            if let RequestContentBlock::ToolResult {
                tool_use_id,
                content: result_content,
                is_error,
            } = block
            {
                let error_marker = if *is_error == Some(true) { " [ERROR]" } else { "" };
                let _ = write!(content, "\n#### `{tool_use_id}`{error_marker}\n\n");
                content.push_str(result_content);
                content.push('\n');
            }
        }

        content.push_str("\n---\n");
        self.append(&content)
    }

    /// Truncates and writes content to the audit file.
    ///
    /// Used by `log_header` so that an agent restart produces a clean file
    /// instead of appending duplicate content.
    fn truncate_and_write(&self, content: &str) -> Result<()> {
        let mut file = OpenOptions::new()
            .create(true)
            .write(true)
            .truncate(true)
            .open(&self.file_path)
            .with_context(|| format!("failed to open audit file: {}", self.file_path.display()))?;

        file.write_all(content.as_bytes())
            .with_context(|| format!("failed to write to audit file: {}", self.file_path.display()))
    }

    /// Appends content to the audit file.
    fn append(&self, content: &str) -> Result<()> {
        let mut file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&self.file_path)
            .with_context(|| format!("failed to open audit file: {}", self.file_path.display()))?;

        file.write_all(content.as_bytes())
            .with_context(|| format!("failed to write to audit file: {}", self.file_path.display()))
    }
}

/// Extracts displayable text from a Claude API message.
fn extract_message_text(message: &Message) -> String {
    match &message.content {
        MessageContent::Text(text) => text.clone(),
        MessageContent::Blocks(blocks) => {
            let texts: Vec<&str> = blocks
                .iter()
                .filter_map(|block| match block {
                    RequestContentBlock::Text { text } => Some(text.as_str()),
                    RequestContentBlock::ToolResult { content, .. } => Some(content.as_str()),
                    RequestContentBlock::ToolUse { .. } => None,
                })
                .collect();
            texts.join("\n")
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn test_logger() -> (AuditLogger, TempDir) {
        let tmp = TempDir::new().expect("create temp dir");
        let logger =
            AuditLogger::new(tmp.path().to_str().unwrap(), "test-agent").expect("create logger");
        (logger, tmp)
    }

    #[test]
    fn new_creates_audit_directory() {
        let tmp = TempDir::new().expect("create temp dir");
        let _logger =
            AuditLogger::new(tmp.path().to_str().unwrap(), "agent-1").expect("create logger");

        assert!(tmp.path().join("audit").exists());
        assert!(tmp.path().join("audit").is_dir());
    }

    #[test]
    fn new_sets_file_path() {
        let (logger, tmp) = test_logger();
        assert_eq!(
            logger.file_path,
            tmp.path().join("audit").join("test-agent.md")
        );
    }

    #[test]
    fn new_stores_agent_name() {
        let (logger, _tmp) = test_logger();
        assert_eq!(logger.agent_name, "test-agent");
    }

    #[test]
    fn new_fails_on_invalid_path() {
        let result = AuditLogger::new("/nonexistent/path/that/cannot/exist", "agent");
        assert!(result.is_err());
    }

    #[test]
    fn log_header_writes_file() {
        let (logger, _tmp) = test_logger();
        logger.log_header("claude-sonnet-4-20250514", None).expect("log header");

        let content = fs::read_to_string(&logger.file_path).expect("read file");
        assert!(content.contains("# Audit Log: test-agent"));
        assert!(content.contains("**Model**: claude-sonnet-4-20250514"));
        assert!(content.contains("**Started**:"));
    }

    #[test]
    fn log_header_with_system_prompt() {
        let (logger, _tmp) = test_logger();
        logger
            .log_header("model", Some("You are an orchestrator."))
            .expect("log header");

        let content = fs::read_to_string(&logger.file_path).expect("read file");
        assert!(content.contains("### System Prompt"));
        assert!(content.contains("You are an orchestrator."));
    }

    #[test]
    fn log_header_without_system_prompt() {
        let (logger, _tmp) = test_logger();
        logger.log_header("model", None).expect("log header");

        let content = fs::read_to_string(&logger.file_path).expect("read file");
        assert!(!content.contains("### System Prompt"));
    }

    #[test]
    fn log_request_writes_iteration_with_timestamp() {
        let (logger, _tmp) = test_logger();
        let messages = vec![Message::user("Execute the task.")];
        logger.log_request(1, &messages).expect("log request");

        let content = fs::read_to_string(&logger.file_path).expect("read file");
        assert!(content.contains("## Iteration 1"));
        assert!(content.contains("### Request"));
        assert!(content.contains("**Timestamp**:"));
        assert!(content.contains("Execute the task."));
    }

    #[test]
    fn log_request_extracts_last_message() {
        let (logger, _tmp) = test_logger();
        let messages = vec![
            Message::user("First message"),
            Message::assistant("Reply"),
            Message::user("Second message"),
        ];
        logger.log_request(2, &messages).expect("log request");

        let content = fs::read_to_string(&logger.file_path).expect("read file");
        assert!(content.contains("Second message"));
        assert!(!content.contains("First message"));
    }

    #[test]
    fn log_request_empty_messages() {
        let (logger, _tmp) = test_logger();
        logger.log_request(1, &[]).expect("log request");

        let content = fs::read_to_string(&logger.file_path).expect("read file");
        assert!(content.contains("## Iteration 1"));
    }

    #[test]
    fn log_response_text_only_with_timestamp() {
        let (logger, _tmp) = test_logger();
        let usage = Usage {
            input_tokens: 100,
            output_tokens: 250,
        };
        logger
            .log_response(1, "Hello from Claude", &[], &usage)
            .expect("log response");

        let content = fs::read_to_string(&logger.file_path).expect("read file");
        assert!(content.contains("### Response (iteration 1)"));
        assert!(content.contains("**Timestamp**:"));
        assert!(content.contains("**Tokens**: 100 input / 250 output"));
        assert!(content.contains("Hello from Claude"));
        assert!(!content.contains("### Tool Calls"));
    }

    #[test]
    fn log_response_with_tool_calls() {
        let (logger, _tmp) = test_logger();
        let usage = Usage {
            input_tokens: 50,
            output_tokens: 100,
        };
        let tool_calls = vec![ToolUseBlock {
            id: "toolu_123".to_string(),
            name: "create_agent".to_string(),
            input: serde_json::json!({"type": "test-generator"}),
        }];
        logger
            .log_response(1, "I'll create an agent.", &tool_calls, &usage)
            .expect("log response");

        let content = fs::read_to_string(&logger.file_path).expect("read file");
        assert!(content.contains("### Tool Calls"));
        assert!(content.contains("#### `create_agent`"));
        assert!(content.contains("test-generator"));
    }

    #[test]
    fn log_response_empty_text() {
        let (logger, _tmp) = test_logger();
        let usage = Usage::default();
        logger
            .log_response(1, "", &[], &usage)
            .expect("log response");

        let content = fs::read_to_string(&logger.file_path).expect("read file");
        assert!(content.contains("### Response"));
        assert!(content.contains("**Tokens**: 0 input / 0 output"));
    }

    #[test]
    fn log_tool_results_writes_results_with_timestamp() {
        let (logger, _tmp) = test_logger();
        let results = vec![RequestContentBlock::ToolResult {
            tool_use_id: "toolu_123".to_string(),
            content: "Agent created successfully".to_string(),
            is_error: None,
        }];
        logger.log_tool_results(1, &results).expect("log results");

        let content = fs::read_to_string(&logger.file_path).expect("read file");
        assert!(content.contains("### Tool Results (iteration 1)"));
        assert!(content.contains("**Timestamp**:"));
        assert!(content.contains("`toolu_123`"));
        assert!(content.contains("Agent created successfully"));
    }

    #[test]
    fn log_tool_results_error_marker() {
        let (logger, _tmp) = test_logger();
        let results = vec![RequestContentBlock::ToolResult {
            tool_use_id: "toolu_456".to_string(),
            content: "Connection refused".to_string(),
            is_error: Some(true),
        }];
        logger.log_tool_results(1, &results).expect("log results");

        let content = fs::read_to_string(&logger.file_path).expect("read file");
        assert!(content.contains("[ERROR]"));
        assert!(content.contains("Connection refused"));
    }

    #[test]
    fn log_tool_results_skips_non_tool_result_blocks() {
        let (logger, _tmp) = test_logger();
        let results = vec![RequestContentBlock::Text {
            text: "ignored".to_string(),
        }];
        logger.log_tool_results(1, &results).expect("log results");

        let content = fs::read_to_string(&logger.file_path).expect("read file");
        assert!(!content.contains("ignored"));
    }

    #[test]
    fn append_creates_file_on_first_write() {
        let (logger, _tmp) = test_logger();
        assert!(!logger.file_path.exists());

        logger.append("hello").expect("append");
        assert!(logger.file_path.exists());

        let content = fs::read_to_string(&logger.file_path).expect("read");
        assert_eq!(content, "hello");
    }

    #[test]
    fn append_accumulates_content() {
        let (logger, _tmp) = test_logger();
        logger.append("first\n").expect("first");
        logger.append("second\n").expect("second");

        let content = fs::read_to_string(&logger.file_path).expect("read");
        assert_eq!(content, "first\nsecond\n");
    }

    #[test]
    fn full_audit_flow() {
        let (logger, _tmp) = test_logger();

        logger
            .log_header("claude-sonnet-4-20250514", Some("You are an agent."))
            .expect("header");
        logger
            .log_request(1, &[Message::user("Do the task.")])
            .expect("request");

        let usage = Usage {
            input_tokens: 100,
            output_tokens: 200,
        };
        let tool_calls = vec![ToolUseBlock {
            id: "toolu_1".to_string(),
            name: "read_file".to_string(),
            input: serde_json::json!({"path": "/src/main.rs"}),
        }];
        logger
            .log_response(1, "I'll read the file.", &tool_calls, &usage)
            .expect("response");

        let results = vec![RequestContentBlock::ToolResult {
            tool_use_id: "toolu_1".to_string(),
            content: "fn main() {}".to_string(),
            is_error: None,
        }];
        logger.log_tool_results(1, &results).expect("tool results");

        let content = fs::read_to_string(&logger.file_path).expect("read file");
        assert!(content.contains("# Audit Log: test-agent"));
        assert!(content.contains("## Iteration 1"));
        assert!(content.contains("Do the task."));
        assert!(content.contains("I'll read the file."));
        assert!(content.contains("read_file"));
        assert!(content.contains("fn main() {}"));
    }

    #[test]
    fn log_header_truncates_on_restart() {
        let (logger, _tmp) = test_logger();

        logger
            .log_header("model-v1", Some("First run"))
            .expect("first header");
        logger
            .log_request(1, &[Message::user("task 1")])
            .expect("first request");

        // Simulate agent restart — log_header called again
        logger
            .log_header("model-v1", Some("Second run"))
            .expect("second header");

        let content = fs::read_to_string(&logger.file_path).expect("read file");
        let header_count = content.matches("# Audit Log:").count();
        assert_eq!(header_count, 1, "restart should truncate, not append");
        assert!(content.contains("Second run"));
        assert!(!content.contains("First run"));
        assert!(!content.contains("task 1"));
    }

    #[test]
    fn is_cloneable() {
        let (logger, _tmp) = test_logger();
        let cloned = logger.clone();
        assert_eq!(cloned.file_path, logger.file_path);
        assert_eq!(cloned.agent_name, logger.agent_name);
    }

    #[test]
    fn extract_message_text_from_plain_text() {
        let msg = Message::user("Hello");
        assert_eq!(extract_message_text(&msg), "Hello");
    }

    #[test]
    fn extract_message_text_from_tool_result_blocks() {
        let msg = Message::tool_results(vec![RequestContentBlock::ToolResult {
            tool_use_id: "toolu_1".to_string(),
            content: "Result text".to_string(),
            is_error: None,
        }]);
        assert_eq!(extract_message_text(&msg), "Result text");
    }

    #[test]
    fn extract_message_text_skips_tool_use_blocks() {
        let msg = Message::assistant_blocks(vec![
            RequestContentBlock::Text {
                text: "Visible".to_string(),
            },
            RequestContentBlock::ToolUse {
                id: "toolu_1".to_string(),
                name: "tool".to_string(),
                input: serde_json::json!({}),
            },
        ]);
        assert_eq!(extract_message_text(&msg), "Visible");
    }
}
