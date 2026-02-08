//! SSE stream consumer for A2A task updates.
//!
//! Connects to a peer agent's SSE endpoint and collects
//! [`StreamResponse`] events until the task reaches a terminal state
//! or the timeout expires.

use std::time::Duration;

use a2a_rs_core::{StreamResponse, TaskState};
use anyhow::{Context, Result};
use eventsource_stream::Eventsource;
use futures::StreamExt;
use tracing::{debug, warn};

/// Default timeout for SSE subscriptions.
const DEFAULT_TIMEOUT: Duration = Duration::from_secs(30);

/// Subscribes to SSE task updates from a peer agent.
///
/// Connects to `GET /v1/tasks/{task_id}/subscribe` on the peer agent
/// and collects events until a terminal state or timeout.
///
/// # Errors
///
/// Returns an error if the HTTP request fails or SSE parsing fails.
pub async fn subscribe(
    agent_url: &str,
    task_id: &str,
    timeout: Option<Duration>,
) -> Result<Vec<StreamResponse>> {
    let timeout = timeout.unwrap_or(DEFAULT_TIMEOUT);
    let url = format!("{agent_url}/v1/tasks/{task_id}/subscribe");

    debug!("📡 SSE subscribing to {url} (timeout: {timeout:?})");

    let response = reqwest::Client::new()
        .get(&url)
        .send()
        .await
        .with_context(|| format!("SSE connect to {url}"))?;

    if !response.status().is_success() {
        anyhow::bail!(
            "SSE subscribe failed: {} {}",
            response.status(),
            url
        );
    }

    let mut stream = response.bytes_stream().eventsource();
    let mut events = Vec::new();
    let deadline = tokio::time::Instant::now() + timeout;

    loop {
        tokio::select! {
            event = stream.next() => {
                match event {
                    Some(Ok(sse_event)) => {
                        match serde_json::from_str::<StreamResponse>(&sse_event.data) {
                            Ok(response) => {
                                debug!("📡 SSE event for task {task_id}: {:?}", event_state(&response));
                                let terminal = is_terminal(&response);
                                events.push(response);
                                if terminal {
                                    debug!("📡 SSE terminal state reached for task {task_id}");
                                    break;
                                }
                            }
                            Err(err) => {
                                warn!("📡 SSE parse error: {err} (data: {})", truncate(&sse_event.data, 100));
                            }
                        }
                    }
                    Some(Err(err)) => {
                        anyhow::bail!("SSE stream error for task {task_id}: {err}");
                    }
                    None => {
                        debug!("📡 SSE stream closed for task {task_id}");
                        break;
                    }
                }
            }
            () = tokio::time::sleep_until(deadline) => {
                debug!("📡 SSE timeout reached for task {task_id}");
                break;
            }
        }
    }

    Ok(events)
}

/// Checks if a stream response contains a terminal task state.
fn is_terminal(response: &StreamResponse) -> bool {
    match response {
        StreamResponse::Task(t) => t.status.state.is_terminal(),
        StreamResponse::StatusUpdate(e) => e.status.state.is_terminal(),
        StreamResponse::ArtifactUpdate(_) | StreamResponse::Message(_) => false,
    }
}

/// Extracts the task state from a stream response for logging.
fn event_state(response: &StreamResponse) -> Option<TaskState> {
    match response {
        StreamResponse::Task(t) => Some(t.status.state),
        StreamResponse::StatusUpdate(e) => Some(e.status.state),
        StreamResponse::ArtifactUpdate(_) | StreamResponse::Message(_) => None,
    }
}

/// Truncates a string for log output.
fn truncate(text: &str, max_len: usize) -> &str {
    if text.len() <= max_len {
        text
    } else {
        let mut end = max_len;
        while !text.is_char_boundary(end) && end > 0 {
            end -= 1;
        }
        &text[..end]
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use a2a_rs_core::{Task, TaskStatus, TaskStatusUpdateEvent};

    #[test]
    fn is_terminal_completed_task() {
        let response = StreamResponse::Task(Task {
            id: "t1".to_string(),
            context_id: "c1".to_string(),
            status: TaskStatus {
                state: TaskState::Completed,
                message: None,
                timestamp: None,
            },
            artifacts: None,
            history: None,
            metadata: None,
        });
        assert!(is_terminal(&response));
    }

    #[test]
    fn is_terminal_working_task() {
        let response = StreamResponse::Task(Task {
            id: "t1".to_string(),
            context_id: "c1".to_string(),
            status: TaskStatus {
                state: TaskState::Working,
                message: None,
                timestamp: None,
            },
            artifacts: None,
            history: None,
            metadata: None,
        });
        assert!(!is_terminal(&response));
    }

    #[test]
    fn is_terminal_completed_status_update() {
        let response = StreamResponse::StatusUpdate(TaskStatusUpdateEvent {
            task_id: "t1".to_string(),
            context_id: "c1".to_string(),
            status: TaskStatus {
                state: TaskState::Completed,
                message: None,
                timestamp: None,
            },
            metadata: None,
        });
        assert!(is_terminal(&response));
    }

    #[test]
    fn is_terminal_working_status_update() {
        let response = StreamResponse::StatusUpdate(TaskStatusUpdateEvent {
            task_id: "t1".to_string(),
            context_id: "c1".to_string(),
            status: TaskStatus {
                state: TaskState::Working,
                message: None,
                timestamp: None,
            },
            metadata: None,
        });
        assert!(!is_terminal(&response));
    }

    #[test]
    fn is_terminal_failed_status_update() {
        let response = StreamResponse::StatusUpdate(TaskStatusUpdateEvent {
            task_id: "t1".to_string(),
            context_id: "c1".to_string(),
            status: TaskStatus {
                state: TaskState::Failed,
                message: None,
                timestamp: None,
            },
            metadata: None,
        });
        assert!(is_terminal(&response));
    }

    #[test]
    fn event_state_extracts_task_state() {
        let response = StreamResponse::Task(Task {
            id: "t1".to_string(),
            context_id: "c1".to_string(),
            status: TaskStatus {
                state: TaskState::Working,
                message: None,
                timestamp: None,
            },
            artifacts: None,
            history: None,
            metadata: None,
        });
        assert_eq!(event_state(&response), Some(TaskState::Working));
    }

    #[test]
    fn event_state_extracts_status_update_state() {
        let response = StreamResponse::StatusUpdate(TaskStatusUpdateEvent {
            task_id: "t1".to_string(),
            context_id: "c1".to_string(),
            status: TaskStatus {
                state: TaskState::Completed,
                message: None,
                timestamp: None,
            },
            metadata: None,
        });
        assert_eq!(event_state(&response), Some(TaskState::Completed));
    }

    #[test]
    fn truncate_short_text_unchanged() {
        assert_eq!(truncate("hello", 10), "hello");
    }

    #[test]
    fn truncate_long_text_cuts() {
        let long = "a".repeat(200);
        let result = truncate(&long, 100);
        assert_eq!(result.len(), 100);
    }

    #[test]
    fn default_timeout_is_30_seconds() {
        assert_eq!(DEFAULT_TIMEOUT, Duration::from_secs(30));
    }
}
