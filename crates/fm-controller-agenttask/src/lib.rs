//! AgentTask CRD and Controller for ForgeMaster.
//!
//! This crate provides the AgentTask Custom Resource Definition and its controller.

pub mod controller;
pub mod crd;
pub mod task_event;
pub mod task_state_changed;
pub mod ws;
