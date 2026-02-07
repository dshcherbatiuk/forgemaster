//! Patches the Agent CR status subresource.

use anyhow::Result;
use fm_controller_agent::crd::Agent;
use kube::Api;
use kube::api::{Patch, PatchParams};
use tracing::debug;

const FIELD_MANAGER: &str = "fm-agent-runtime";

/// Updates Agent CR status fields via status subresource patches.
pub struct StatusUpdater {
    api: Api<Agent>,
    agent_name: String,
}

impl StatusUpdater {
    /// Creates a new updater for the given agent.
    pub fn new(client: kube::Client, namespace: &str, agent_name: &str) -> Self {
        Self {
            api: Api::namespaced(client, namespace),
            agent_name: agent_name.to_string(),
        }
    }

    /// Transitions the agent to Running phase with a start time.
    pub async fn transition_to_running(&self) -> Result<()> {
        let now = chrono::Utc::now().to_rfc3339();
        let patch = build_running_patch(&now);

        self.apply_status_patch(&patch).await?;
        debug!("📝 Agent {} status → Running", self.agent_name);
        Ok(())
    }

    /// Transitions the agent to Succeeded phase with final metrics.
    pub async fn transition_to_succeeded(
        &self,
        tokens_used: i64,
        iterations_completed: i32,
    ) -> Result<()> {
        let now = chrono::Utc::now().to_rfc3339();
        let patch = build_succeeded_patch(&now, tokens_used, iterations_completed);

        self.apply_status_patch(&patch).await?;
        debug!("📝 Agent {} status → Succeeded", self.agent_name);
        Ok(())
    }

    /// Transitions the agent to Failed phase with a condition.
    pub async fn transition_to_failed(&self, reason: &str, message: &str) -> Result<()> {
        let now = chrono::Utc::now().to_rfc3339();
        let patch = build_failed_patch(&now, reason, message);

        self.apply_status_patch(&patch).await?;
        debug!("📝 Agent {} status → Failed: {reason}", self.agent_name);
        Ok(())
    }

    /// Applies a merge patch to the status subresource.
    async fn apply_status_patch(&self, patch: &serde_json::Value) -> Result<()> {
        self.api
            .patch_status(
                &self.agent_name,
                &PatchParams::apply(FIELD_MANAGER),
                &Patch::Merge(patch),
            )
            .await?;
        Ok(())
    }
}

/// Builds the JSON patch for transitioning to Running.
pub fn build_running_patch(timestamp: &str) -> serde_json::Value {
    serde_json::json!({
        "status": {
            "phase": "Running",
            "startTime": timestamp,
        }
    })
}

/// Builds the JSON patch for transitioning to Succeeded.
pub fn build_succeeded_patch(
    timestamp: &str,
    tokens_used: i64,
    iterations_completed: i32,
) -> serde_json::Value {
    serde_json::json!({
        "status": {
            "phase": "Succeeded",
            "completionTime": timestamp,
            "tokensUsed": tokens_used,
            "iterationsCompleted": iterations_completed,
        }
    })
}

/// Builds the JSON patch for transitioning to Failed.
pub fn build_failed_patch(timestamp: &str, reason: &str, message: &str) -> serde_json::Value {
    serde_json::json!({
        "status": {
            "phase": "Failed",
            "completionTime": timestamp,
            "conditions": [{
                "type": "Failed",
                "status": "True",
                "reason": reason,
                "message": message,
                "lastTransitionTime": timestamp,
            }],
        }
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn running_patch_has_phase_and_start_time() {
        let patch = build_running_patch("2026-02-07T10:00:00Z");
        let status = &patch["status"];
        assert_eq!(status["phase"], "Running");
        assert_eq!(status["startTime"], "2026-02-07T10:00:00Z");
        assert!(status.get("completionTime").is_none());
    }

    #[test]
    fn succeeded_patch_has_all_fields() {
        let patch = build_succeeded_patch("2026-02-07T10:05:00Z", 1500, 1);
        let status = &patch["status"];
        assert_eq!(status["phase"], "Succeeded");
        assert_eq!(status["completionTime"], "2026-02-07T10:05:00Z");
        assert_eq!(status["tokensUsed"], 1500);
        assert_eq!(status["iterationsCompleted"], 1);
    }

    #[test]
    fn succeeded_patch_does_not_include_conditions() {
        let patch = build_succeeded_patch("2026-02-07T10:05:00Z", 100, 1);
        assert!(patch["status"].get("conditions").is_none());
    }

    #[test]
    fn failed_patch_has_condition() {
        let patch = build_failed_patch("2026-02-07T10:01:00Z", "ExecutionError", "API timeout");
        let status = &patch["status"];
        assert_eq!(status["phase"], "Failed");
        assert_eq!(status["completionTime"], "2026-02-07T10:01:00Z");

        let conditions = status["conditions"].as_array().expect("conditions array");
        assert_eq!(conditions.len(), 1);
        assert_eq!(conditions[0]["type"], "Failed");
        assert_eq!(conditions[0]["status"], "True");
        assert_eq!(conditions[0]["reason"], "ExecutionError");
        assert_eq!(conditions[0]["message"], "API timeout");
    }

    #[test]
    fn running_patch_does_not_overwrite_tokens() {
        let patch = build_running_patch("2026-02-07T10:00:00Z");
        assert!(patch["status"].get("tokensUsed").is_none());
    }
}
