//! Logs agent output.

use tracing::info;

/// Logs agent output to stdout via tracing.
pub struct OutputWriter {
    agent_name: String,
}

impl OutputWriter {
    /// Creates a new writer for the given agent.
    pub fn new(agent_name: &str) -> Self {
        Self {
            agent_name: agent_name.to_string(),
        }
    }

    /// Logs the agent output.
    pub fn write(&self, output: &str) {
        info!(
            agent = %self.agent_name,
            output_length = output.len(),
            "📝 Agent output:\n{output}"
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn writer_stores_agent_name() {
        let writer = OutputWriter::new("test-agent");
        assert_eq!(writer.agent_name, "test-agent");
    }

    #[test]
    fn write_does_not_panic() {
        let writer = OutputWriter::new("agent-1");
        writer.write("Hello, world!");
    }

    #[test]
    fn write_handles_empty_output() {
        let writer = OutputWriter::new("agent-1");
        writer.write("");
    }
}
