//! Manages conversation message history and token tracking.

use crate::claude_api::request::{Message, Role};
use crate::claude_api::response::Usage;

/// Tracks conversation state: system prompt, messages, and cumulative token usage.
pub struct Conversation {
    system_prompt: String,
    messages: Vec<Message>,
    total_input_tokens: i64,
    total_output_tokens: i64,
}

impl Conversation {
    /// Creates a new conversation with the given system prompt.
    pub fn new(system_prompt: String) -> Self {
        Self {
            system_prompt,
            messages: Vec::new(),
            total_input_tokens: 0,
            total_output_tokens: 0,
        }
    }

    /// Returns the system prompt.
    pub fn system_prompt(&self) -> &str {
        &self.system_prompt
    }

    /// Returns the message history.
    pub fn messages(&self) -> &[Message] {
        &self.messages
    }

    /// Adds a user message to the conversation.
    pub fn add_user_message(&mut self, content: String) {
        self.messages.push(Message {
            role: Role::User,
            content,
        });
    }

    /// Adds an assistant message to the conversation.
    pub fn add_assistant_message(&mut self, content: String) {
        self.messages.push(Message {
            role: Role::Assistant,
            content,
        });
    }

    /// Accumulates token usage from a response.
    pub fn add_usage(&mut self, usage: Usage) {
        self.total_input_tokens += usage.input_tokens;
        self.total_output_tokens += usage.output_tokens;
    }

    /// Returns the total tokens consumed (input + output).
    pub fn total_tokens(&self) -> i64 {
        self.total_input_tokens + self.total_output_tokens
    }

    /// Returns the number of complete turns (user + assistant pairs).
    pub fn turn_count(&self) -> usize {
        self.messages.len() / 2
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_conversation_has_empty_messages() {
        let conv = Conversation::new("You are helpful.".to_string());
        assert!(conv.messages().is_empty());
        assert_eq!(conv.system_prompt(), "You are helpful.");
    }

    #[test]
    fn add_user_then_assistant_maintains_order() {
        let mut conv = Conversation::new(String::new());
        conv.add_user_message("hello".to_string());
        conv.add_assistant_message("hi".to_string());

        assert_eq!(conv.messages().len(), 2);
        assert_eq!(conv.messages()[0].role, Role::User);
        assert_eq!(conv.messages()[0].content, "hello");
        assert_eq!(conv.messages()[1].role, Role::Assistant);
        assert_eq!(conv.messages()[1].content, "hi");
    }

    #[test]
    fn token_accumulation() {
        let mut conv = Conversation::new(String::new());
        conv.add_usage(Usage {
            input_tokens: 100,
            output_tokens: 50,
        });
        conv.add_usage(Usage {
            input_tokens: 200,
            output_tokens: 80,
        });

        assert_eq!(conv.total_tokens(), 430);
    }

    #[test]
    fn turn_count_zero_for_empty() {
        let conv = Conversation::new(String::new());
        assert_eq!(conv.turn_count(), 0);
    }

    #[test]
    fn turn_count_after_one_exchange() {
        let mut conv = Conversation::new(String::new());
        conv.add_user_message("q".to_string());
        conv.add_assistant_message("a".to_string());
        assert_eq!(conv.turn_count(), 1);
    }

    #[test]
    fn turn_count_with_pending_user_message() {
        let mut conv = Conversation::new(String::new());
        conv.add_user_message("q".to_string());
        // No assistant response yet
        assert_eq!(conv.turn_count(), 0);
    }

    #[test]
    fn total_tokens_starts_at_zero() {
        let conv = Conversation::new(String::new());
        assert_eq!(conv.total_tokens(), 0);
    }
}
