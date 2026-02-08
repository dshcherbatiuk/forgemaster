//! Publishes Kubernetes Events for Agent CRs.
//!
//! Uses `kube::runtime::events::Recorder` to emit lifecycle events
//! visible via `kubectl describe agent <name>`.

use kube::Resource;
use kube::runtime::events::{Event, EventType, Recorder, Reporter};
use tracing::warn;

use crate::crd::Agent;

/// Controller name used as the reporting source for events.
const CONTROLLER_NAME: &str = "fm-controller-agent";

/// Publishes a Kubernetes Event for the given Agent CR.
///
/// Events are best-effort — failures are logged but not propagated
/// to avoid blocking reconciliation.
pub async fn publish(
    client: &kube::Client,
    agent: &Agent,
    event_type: EventType,
    reason: &str,
    message: &str,
) {
    let reporter = Reporter {
        controller: CONTROLLER_NAME.into(),
        instance: None,
    };
    let recorder = Recorder::new(client.clone(), reporter);
    let object_ref = agent.object_ref(&());

    let event = Event {
        type_: event_type,
        reason: reason.into(),
        note: Some(message.into()),
        action: "Reconciling".into(),
        secondary: None,
    };

    if let Err(err) = recorder.publish(&event, &object_ref).await {
        warn!(
            "⚠️ Failed to publish event for agent {}: {err}",
            agent.metadata.name.as_deref().unwrap_or("unknown")
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Builds an `Event` struct without publishing (for testing).
    fn build_event(event_type: EventType, reason: &str, message: &str) -> Event {
        Event {
            type_: event_type,
            reason: reason.into(),
            note: Some(message.into()),
            action: "Reconciling".into(),
            secondary: None,
        }
    }

    #[test]
    fn build_normal_event() {
        let event = build_event(EventType::Normal, "PodCreated", "Created runtime pod test-agent");
        assert_eq!(event.reason, "PodCreated");
        assert_eq!(event.note, Some("Created runtime pod test-agent".into()));
        assert_eq!(event.action, "Reconciling");
        assert!(matches!(event.type_, EventType::Normal));
        assert!(event.secondary.is_none());
    }

    #[test]
    fn build_warning_event() {
        let event = build_event(EventType::Warning, "AgentFailed", "Pod failed: OOMKilled");
        assert_eq!(event.reason, "AgentFailed");
        assert_eq!(event.note, Some("Pod failed: OOMKilled".into()));
        assert!(matches!(event.type_, EventType::Warning));
    }

    #[test]
    fn controller_name_constant() {
        assert_eq!(CONTROLLER_NAME, "fm-controller-agent");
    }

    #[test]
    fn reporter_uses_controller_name() {
        let reporter = Reporter {
            controller: CONTROLLER_NAME.into(),
            instance: None,
        };
        assert_eq!(reporter.controller, "fm-controller-agent");
        assert!(reporter.instance.is_none());
    }
}
