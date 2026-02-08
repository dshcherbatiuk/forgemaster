//! MCP client for connecting to MCP servers inside the Kubernetes cluster.
//!
//! Uses the official `rmcp` SDK with Streamable HTTP transport
//! to discover tools (`tools/list`) and execute tool calls (`tools/call`).

mod client;
mod executor;
mod tool_converter;

pub use client::McpClient;
pub use executor::McpToolExecutor;
