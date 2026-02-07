//! Exponential backoff retry policy for Claude API rate limits.

use std::time::Duration;

const DEFAULT_MAX_RETRIES: u32 = 3;
const DEFAULT_INITIAL_BACKOFF_SECS: u64 = 1;
const DEFAULT_MAX_BACKOFF_SECS: u64 = 60;

/// Determines whether and how long to wait before retrying a failed request.
#[derive(Debug, Clone)]
pub struct RetryPolicy {
    /// Maximum number of retry attempts.
    max_retries: u32,
    /// Initial backoff duration (doubles on each attempt).
    initial_backoff: Duration,
    /// Maximum backoff duration cap.
    max_backoff: Duration,
}

impl Default for RetryPolicy {
    fn default() -> Self {
        Self {
            max_retries: DEFAULT_MAX_RETRIES,
            initial_backoff: Duration::from_secs(DEFAULT_INITIAL_BACKOFF_SECS),
            max_backoff: Duration::from_secs(DEFAULT_MAX_BACKOFF_SECS),
        }
    }
}

impl RetryPolicy {
    /// Creates a retry policy with custom parameters.
    pub fn new(max_retries: u32, initial_backoff: Duration, max_backoff: Duration) -> Self {
        Self {
            max_retries,
            initial_backoff,
            max_backoff,
        }
    }

    /// Returns the backoff duration if the request should be retried, or `None` if not.
    ///
    /// Only retries on 429 (rate limit) and 529 (overloaded) status codes.
    /// Uses exponential backoff: `min(initial * 2^attempt, max_backoff)`.
    /// If `retry_after` is provided (from the `retry-after` header), it takes precedence.
    pub fn should_retry(
        &self,
        attempt: u32,
        status_code: u16,
        retry_after: Option<Duration>,
    ) -> Option<Duration> {
        if attempt >= self.max_retries {
            return None;
        }

        if !is_retryable(status_code) {
            return None;
        }

        if let Some(server_backoff) = retry_after {
            return Some(server_backoff);
        }

        let backoff = self.initial_backoff * 2u32.saturating_pow(attempt);
        Some(backoff.min(self.max_backoff))
    }
}

/// Returns `true` for HTTP status codes that are worth retrying.
fn is_retryable(status_code: u16) -> bool {
    matches!(status_code, 429 | 529)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_policy_values() {
        let policy = RetryPolicy::default();
        assert_eq!(policy.max_retries, 3);
        assert_eq!(policy.initial_backoff, Duration::from_secs(1));
        assert_eq!(policy.max_backoff, Duration::from_secs(60));
    }

    #[test]
    fn retries_on_429() {
        let policy = RetryPolicy::default();
        let result = policy.should_retry(0, 429, None);
        assert!(result.is_some());
    }

    #[test]
    fn retries_on_529() {
        let policy = RetryPolicy::default();
        let result = policy.should_retry(0, 529, None);
        assert!(result.is_some());
    }

    #[test]
    fn no_retry_on_400() {
        let policy = RetryPolicy::default();
        assert!(policy.should_retry(0, 400, None).is_none());
    }

    #[test]
    fn no_retry_on_401() {
        let policy = RetryPolicy::default();
        assert!(policy.should_retry(0, 401, None).is_none());
    }

    #[test]
    fn no_retry_on_500() {
        let policy = RetryPolicy::default();
        assert!(policy.should_retry(0, 500, None).is_none());
    }

    #[test]
    fn exponential_backoff() {
        let policy = RetryPolicy::new(
            5,
            Duration::from_secs(1),
            Duration::from_secs(60),
        );

        assert_eq!(policy.should_retry(0, 429, None), Some(Duration::from_secs(1)));
        assert_eq!(policy.should_retry(1, 429, None), Some(Duration::from_secs(2)));
        assert_eq!(policy.should_retry(2, 429, None), Some(Duration::from_secs(4)));
        assert_eq!(policy.should_retry(3, 429, None), Some(Duration::from_secs(8)));
    }

    #[test]
    fn backoff_capped_at_max() {
        let policy = RetryPolicy::new(
            10,
            Duration::from_secs(1),
            Duration::from_secs(5),
        );

        assert_eq!(policy.should_retry(0, 429, None), Some(Duration::from_secs(1)));
        assert_eq!(policy.should_retry(1, 429, None), Some(Duration::from_secs(2)));
        assert_eq!(policy.should_retry(2, 429, None), Some(Duration::from_secs(4)));
        assert_eq!(policy.should_retry(3, 429, None), Some(Duration::from_secs(5)));
        assert_eq!(policy.should_retry(4, 429, None), Some(Duration::from_secs(5)));
    }

    #[test]
    fn max_retries_exceeded() {
        let policy = RetryPolicy::new(
            2,
            Duration::from_secs(1),
            Duration::from_secs(60),
        );

        assert!(policy.should_retry(0, 429, None).is_some());
        assert!(policy.should_retry(1, 429, None).is_some());
        assert!(policy.should_retry(2, 429, None).is_none());
    }

    #[test]
    fn retry_after_header_takes_precedence() {
        let policy = RetryPolicy::default();
        let server_says = Duration::from_secs(30);
        let result = policy.should_retry(0, 429, Some(server_says));
        assert_eq!(result, Some(Duration::from_secs(30)));
    }

    #[test]
    fn retry_after_ignored_for_non_retryable() {
        let policy = RetryPolicy::default();
        let result = policy.should_retry(0, 400, Some(Duration::from_secs(10)));
        assert!(result.is_none());
    }
}
