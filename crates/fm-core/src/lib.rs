//! `ForgeMaster` Core
//!
//! Core types, traits, and configuration for the `ForgeMaster` meta-agent system.

/// Returns a greeting message.
#[must_use]
pub const fn hello() -> &'static str {
    "Hello from ForgeMaster!"
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hello() {
        assert_eq!(hello(), "Hello from ForgeMaster!");
    }
}
