//! Error types for Claude API interactions.

/// Errors that can occur when calling the Claude Messages API.
#[derive(Debug, thiserror::Error)]
pub enum ClaudeApiError {
    /// HTTP request failed at the transport level.
    #[error("HTTP request failed: {0}")]
    HttpRequest(#[from] reqwest::Error),

    /// API returned a non-success status code.
    #[error("API error {status}: {message}")]
    ApiResponse {
        /// HTTP status code.
        status: u16,
        /// Error message from the API.
        message: String,
    },

    /// Failed to parse an SSE event from the stream.
    #[error("SSE parse error: {0}")]
    SseParse(String),

    /// Rate limit retries exhausted.
    #[error("rate limited after {retries} retries")]
    RateLimitExhausted {
        /// Number of retries attempted.
        retries: u32,
    },

    /// Authentication failed (invalid API key).
    #[error("authentication failed: invalid API key")]
    Authentication,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn api_response_formats() {
        let err = ClaudeApiError::ApiResponse {
            status: 400,
            message: "invalid request".to_string(),
        };
        assert_eq!(err.to_string(), "API error 400: invalid request");
    }

    #[test]
    fn sse_parse_formats() {
        let err = ClaudeApiError::SseParse("unexpected EOF".to_string());
        assert!(err.to_string().contains("unexpected EOF"));
    }

    #[test]
    fn rate_limit_exhausted_formats() {
        let err = ClaudeApiError::RateLimitExhausted { retries: 3 };
        assert!(err.to_string().contains("3"));
    }

    #[test]
    fn authentication_formats() {
        let err = ClaudeApiError::Authentication;
        assert!(err.to_string().contains("invalid API key"));
    }
}
