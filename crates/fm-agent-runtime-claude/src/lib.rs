//! Agent Runtime for ForgeMaster.
//!
//! This crate provides the base agent execution runtime that runs inside
//! each Agent pod. It receives config from env vars (injected by the Agent
//! Controller), calls the Claude Messages API with streaming, logs output,
//! and updates Agent CR status.

pub mod a2a;
pub mod audit_logger;
pub mod claude_api;
pub mod config;
pub mod conversation;
pub mod conversation_loop;
pub mod error;
pub mod mcp_client;
pub mod output_writer;
pub mod runtime;
pub mod status_updater;
pub mod tool_executor;
