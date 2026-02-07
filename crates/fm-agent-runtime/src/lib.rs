//! Agent Runtime for ForgeMaster.
//!
//! This crate provides the base agent execution runtime that runs inside
//! each Agent pod. It receives config from env vars (injected by the Agent
//! Controller), calls the Claude Messages API with streaming, logs output,
//! and updates Agent CR status.

pub mod claude_api;
pub mod config;
pub mod conversation;
pub mod error;
pub mod execution;
pub mod output_writer;
pub mod status_updater;
