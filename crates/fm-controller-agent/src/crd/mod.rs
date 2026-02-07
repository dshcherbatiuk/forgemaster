//! Agent CRD types.

mod agent;
mod condition;
mod mcp_server_ref;
mod model_config;
mod model_provider;
mod output_ref;
mod pod_ref;
mod resource_limits;
mod resource_requirements;

pub use agent::{Agent, AgentCrd, AgentPhase, AgentStatus};
pub use condition::Condition;
pub use mcp_server_ref::McpServerRef;
pub use model_config::ModelConfig;
pub use model_provider::ModelProvider;
pub use output_ref::OutputRef;
pub use pod_ref::PodRef;
pub use resource_limits::ResourceLimits;
pub use resource_requirements::ResourceRequirements;
