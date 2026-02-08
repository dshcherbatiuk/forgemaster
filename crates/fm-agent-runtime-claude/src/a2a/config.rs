//! A2A (Agent-to-Agent) communication configuration.
//!
//! Loaded from environment variables injected by the Agent Controller.

use smallvec::SmallVec;

/// Default port for the A2A server.
const DEFAULT_PORT: u16 = 9090;

/// Most agents communicate with 1–3 peers; stack-allocate up to 4.
pub type PeerUrls = SmallVec<[String; 4]>;

/// A2A communication configuration.
///
/// Loaded from env vars: `A2A_PORT`, `AGENT_TYPE`, `A2A_PEER_URLS`.
#[derive(Debug, Clone)]
pub struct A2aConfig {
    /// Port for the A2A server.
    pub port: u16,
    /// Agent type (e.g. "orchestrator", "code-generator").
    pub agent_type: String,
    /// Peer agent A2A URLs for direct communication.
    pub peer_urls: PeerUrls,
}

impl A2aConfig {
    /// Loads A2A config from environment variables.
    pub fn from_env() -> Self {
        let port = std::env::var("A2A_PORT")
            .ok()
            .and_then(|v| v.parse().ok())
            .unwrap_or(DEFAULT_PORT);

        let agent_type = std::env::var("AGENT_TYPE")
            .unwrap_or_else(|_| "unknown".to_string());

        let peer_urls = std::env::var("A2A_PEER_URLS")
            .ok()
            .map(|val| parse_peer_urls(&val))
            .unwrap_or_default();

        Self { port, agent_type, peer_urls }
    }

    /// Creates a config for testing.
    #[must_use]
    pub fn for_test(agent_type: &str) -> Self {
        Self {
            port: DEFAULT_PORT,
            agent_type: agent_type.to_string(),
            peer_urls: PeerUrls::new(),
        }
    }
}

/// Parses comma-separated peer URLs into a `PeerUrls` collection.
fn parse_peer_urls(input: &str) -> PeerUrls {
    input
        .split(',')
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(String::from)
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_port_is_9090() {
        assert_eq!(DEFAULT_PORT, 9090);
    }

    #[test]
    fn for_test_uses_defaults() {
        let config = A2aConfig::for_test("code-generator");
        assert_eq!(config.port, 9090);
        assert_eq!(config.agent_type, "code-generator");
        assert!(config.peer_urls.is_empty());
    }

    #[test]
    fn parse_peer_urls_single() {
        let urls = parse_peer_urls("http://test-generator.task-abc.svc.cluster.local:9090");
        assert_eq!(urls.len(), 1);
        assert!(urls[0].contains("test-generator"));
    }

    #[test]
    fn parse_peer_urls_multiple() {
        let urls = parse_peer_urls(
            "http://test-gen.ns:9090,http://code-gen.ns:9090"
        );
        assert_eq!(urls.len(), 2);
    }

    #[test]
    fn parse_peer_urls_empty() {
        let urls = parse_peer_urls("");
        assert!(urls.is_empty());
    }

    #[test]
    fn parse_peer_urls_trims_whitespace() {
        let urls = parse_peer_urls("  http://a:9090 , http://b:9090  ");
        assert_eq!(urls.len(), 2);
        assert_eq!(urls[0], "http://a:9090");
        assert_eq!(urls[1], "http://b:9090");
    }

    #[test]
    fn parse_peer_urls_skips_empty_segments() {
        let urls = parse_peer_urls("http://a:9090,,http://b:9090,");
        assert_eq!(urls.len(), 2);
    }
}
