//! Builds an A2A Agent Card from agent runtime configuration.
//!
//! The Agent Card describes this agent's capabilities and skills
//! to peer agents that discover it via the A2A protocol.

use a2a_rs_core::{
    AgentCapabilities, AgentCard, AgentInterface, AgentSkill, PROTOCOL_VERSION,
};

/// Builds an [`AgentCard`] from agent runtime configuration.
///
/// Uses the agent name and type to construct a card that describes
/// this agent's identity, endpoint, and skills.
pub struct AgentCardBuilder<'a> {
    agent_name: &'a str,
    agent_type: &'a str,
}

impl<'a> AgentCardBuilder<'a> {
    /// Creates a new builder for the given agent.
    #[must_use]
    pub const fn new(agent_name: &'a str, agent_type: &'a str) -> Self {
        Self {
            agent_name,
            agent_type,
        }
    }

    /// Builds the [`AgentCard`] with the given base URL.
    ///
    /// The base URL is used to construct the JSON-RPC endpoint URL
    /// (e.g., `http://agent-name.namespace.svc.cluster.local:9090/v1/rpc`).
    #[must_use]
    pub fn build(&self, base_url: &str) -> AgentCard {
        AgentCard {
            name: self.agent_name.to_string(),
            description: format!("{} agent for ForgeMaster", self.agent_type),
            supported_interfaces: vec![AgentInterface {
                url: format!("{base_url}/v1/rpc"),
                protocol_binding: "JSONRPC".to_string(),
                protocol_version: PROTOCOL_VERSION.to_string(),
                tenant: None,
            }],
            version: PROTOCOL_VERSION.to_string(),
            capabilities: AgentCapabilities {
                streaming: Some(true),
                ..Default::default()
            },
            skills: vec![self.build_skill()],
            ..Default::default()
        }
    }

    /// Builds a skill descriptor from the agent type.
    fn build_skill(&self) -> AgentSkill {
        AgentSkill {
            id: self.agent_type.to_string(),
            name: self.agent_type.to_string(),
            description: format!("Handles {} tasks", self.agent_type),
            tags: vec![self.agent_type.to_string()],
            ..Default::default()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn builds_agent_card_with_name() {
        let builder = AgentCardBuilder::new("test-gen-task-abc", "test-generator");
        let card = builder.build("http://test-gen-task-abc.task-abc.svc.cluster.local:9090");
        assert_eq!(card.name, "test-gen-task-abc");
    }

    #[test]
    fn builds_agent_card_with_description() {
        let builder = AgentCardBuilder::new("code-gen", "code-generator");
        let card = builder.build("http://localhost:9090");
        assert_eq!(card.description, "code-generator agent for ForgeMaster");
    }

    #[test]
    fn builds_rpc_endpoint() {
        let builder = AgentCardBuilder::new("agent", "code-gen");
        let card = builder.build("http://agent.ns:9090");

        assert_eq!(card.supported_interfaces.len(), 1);
        assert_eq!(
            card.supported_interfaces[0].url,
            "http://agent.ns:9090/v1/rpc"
        );
        assert_eq!(card.supported_interfaces[0].protocol_binding, "JSONRPC");
    }

    #[test]
    fn builds_skill_from_agent_type() {
        let builder = AgentCardBuilder::new("agent", "test-generator");
        let card = builder.build("http://localhost:9090");

        assert_eq!(card.skills.len(), 1);
        assert_eq!(card.skills[0].id, "test-generator");
        assert_eq!(card.skills[0].name, "test-generator");
        assert!(card.skills[0].tags.contains(&"test-generator".to_string()));
    }

    #[test]
    fn protocol_version_is_set() {
        let builder = AgentCardBuilder::new("agent", "type");
        let card = builder.build("http://localhost:9090");
        assert_eq!(card.version, PROTOCOL_VERSION);
    }

    #[test]
    fn streaming_capability_enabled() {
        let builder = AgentCardBuilder::new("agent", "type");
        let card = builder.build("http://localhost:9090");
        assert_eq!(card.capabilities.streaming, Some(true));
    }

    #[test]
    fn skill_description_includes_type() {
        let builder = AgentCardBuilder::new("agent", "reviewer");
        let card = builder.build("http://localhost:9090");
        assert_eq!(card.skills[0].description, "Handles reviewer tasks");
    }
}
