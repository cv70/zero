/// Approximate token counter for messages
///
/// Provides a fast, heuristic-based token estimation without requiring
/// a tokenizer library. Different LLM providers have different average
/// characters-per-token ratios, so this module supports provider-specific
/// counters.
use crate::message::{ContentBlock, Message, ToolResultContent};

/// Approximate token counter for messages
pub struct TokenCounter {
    /// Average characters per token (provider-specific)
    chars_per_token: f64,
}

impl TokenCounter {
    /// Create counter for Anthropic models (~3.5 chars per token for English)
    pub fn anthropic() -> Self {
        Self {
            chars_per_token: 3.5,
        }
    }

    /// Create counter for OpenAI models (~4 chars per token)
    pub fn openai() -> Self {
        Self {
            chars_per_token: 4.0,
        }
    }

    /// Create counter with custom ratio
    pub fn new(chars_per_token: f64) -> Self {
        Self { chars_per_token }
    }

    /// Estimate token count for a slice of messages
    pub fn count_messages(&self, messages: &[Message]) -> usize {
        let total_chars: usize = messages.iter().map(|m| self.message_chars(m)).sum();
        // Add per-message overhead (~4 tokens for role, separators)
        let overhead = messages.len() * 4;
        (total_chars as f64 / self.chars_per_token) as usize + overhead
    }

    /// Count chars in a single message
    fn message_chars(&self, message: &Message) -> usize {
        match message {
            Message::User { content } => content.len(),
            Message::Assistant { content } => content.iter().map(|b| self.block_chars(b)).sum(),
            Message::ToolResult { content } => {
                content.iter().map(|tr| self.tool_result_chars(tr)).sum()
            }
        }
    }

    fn block_chars(&self, block: &ContentBlock) -> usize {
        match block {
            ContentBlock::Text { text } => text.len(),
            ContentBlock::ToolUse { id, name, input } => {
                id.len() + name.len() + input.to_string().len()
            }
        }
    }

    fn tool_result_chars(&self, tr: &ToolResultContent) -> usize {
        match tr {
            ToolResultContent::ToolResult {
                tool_use_id,
                content,
            } => tool_use_id.len() + content.len(),
        }
    }
}

impl Default for TokenCounter {
    fn default() -> Self {
        Self::anthropic()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_anthropic_default() {
        let counter = TokenCounter::anthropic();
        assert!((counter.chars_per_token - 3.5).abs() < f64::EPSILON);
    }

    #[test]
    fn test_openai_default() {
        let counter = TokenCounter::openai();
        assert!((counter.chars_per_token - 4.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_custom_ratio() {
        let counter = TokenCounter::new(2.0);
        assert!((counter.chars_per_token - 2.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_default_is_anthropic() {
        let counter = TokenCounter::default();
        assert!((counter.chars_per_token - 3.5).abs() < f64::EPSILON);
    }

    #[test]
    fn test_count_empty_messages() {
        let counter = TokenCounter::default();
        let messages: Vec<Message> = vec![];
        assert_eq!(counter.count_messages(&messages), 0);
    }

    #[test]
    fn test_count_single_user_message() {
        let counter = TokenCounter::default();
        // "Hello, world!" is 13 chars => 13/3.5 = 3.71 => 3 + 4 overhead = 7
        let messages = vec![Message::user("Hello, world!")];
        let count = counter.count_messages(&messages);
        // 13 chars / 3.5 = 3 (truncated) + 4 overhead = 7
        assert_eq!(count, 7);
    }

    #[test]
    fn test_count_assistant_with_text() {
        let counter = TokenCounter::default();
        let messages = vec![Message::assistant(vec![ContentBlock::text(
            "This is a response from the assistant.",
        )])];
        let count = counter.count_messages(&messages);
        // 38 chars / 3.5 = 10 (truncated) + 4 overhead = 14
        let text = "This is a response from the assistant.";
        let expected = (text.len() as f64 / 3.5) as usize + 4;
        assert_eq!(count, expected);
    }

    #[test]
    fn test_count_assistant_with_tool_use() {
        let counter = TokenCounter::default();
        let messages = vec![Message::assistant(vec![
            ContentBlock::text("Let me check."),
            ContentBlock::tool_use(
                "toolu_1".to_string(),
                "bash".to_string(),
                json!({"command": "ls -la"}),
            ),
        ])];
        let count = counter.count_messages(&messages);
        // Text: 13 chars
        // ToolUse: "toolu_1" (7) + "bash" (4) + json string len
        let json_str = json!({"command": "ls -la"}).to_string();
        let total_chars = 13 + 7 + 4 + json_str.len();
        let expected = (total_chars as f64 / 3.5) as usize + 4;
        assert_eq!(count, expected);
    }

    #[test]
    fn test_count_tool_result() {
        let counter = TokenCounter::default();
        let messages = vec![Message::tool_result(
            "toolu_1".to_string(),
            "file1.txt\nfile2.txt",
        )];
        let count = counter.count_messages(&messages);
        // "toolu_1" (7) + "file1.txt\nfile2.txt" (19) = 26 chars
        let expected = (26_f64 / 3.5) as usize + 4;
        assert_eq!(count, expected);
    }

    #[test]
    fn test_count_multiple_tool_results() {
        let counter = TokenCounter::default();
        let messages = vec![Message::tool_results(vec![
            ("toolu_1".to_string(), "result 1".to_string()),
            ("toolu_2".to_string(), "result 2".to_string()),
        ])];
        let count = counter.count_messages(&messages);
        // "toolu_1" (7) + "result 1" (8) + "toolu_2" (7) + "result 2" (8) = 30
        let expected = (30_f64 / 3.5) as usize + 4;
        assert_eq!(count, expected);
    }

    #[test]
    fn test_count_full_conversation() {
        let counter = TokenCounter::default();
        let messages = vec![
            Message::user("List files"),
            Message::assistant(vec![
                ContentBlock::text("I'll check."),
                ContentBlock::tool_use(
                    "t1".to_string(),
                    "bash".to_string(),
                    json!({"command": "ls"}),
                ),
            ]),
            Message::tool_result("t1".to_string(), "README.md\nsrc/"),
            Message::assistant(vec![ContentBlock::text("Here are the files.")]),
        ];
        let count = counter.count_messages(&messages);
        // Should be > 0 and reasonable
        assert!(count > 0);
        // 4 messages * 4 overhead = 16 tokens just from overhead
        assert!(count >= 16);
    }

    #[test]
    fn test_estimates_are_reasonable() {
        let counter = TokenCounter::anthropic();
        // ~100 chars of English text should be roughly 28 tokens + 4 overhead = 32
        let text = "a".repeat(100);
        let messages = vec![Message::user(text)];
        let count = counter.count_messages(&messages);
        // 100 / 3.5 = 28 + 4 = 32
        assert_eq!(count, 32);

        // OpenAI counter: 100 / 4.0 = 25 + 4 = 29
        let openai_counter = TokenCounter::openai();
        let messages2 = vec![Message::user("a".repeat(100))];
        let count2 = openai_counter.count_messages(&messages2);
        assert_eq!(count2, 29);
    }

    #[test]
    fn test_overhead_per_message() {
        let counter = TokenCounter::default();
        // Two empty-ish messages should show overhead clearly
        let messages = vec![Message::user(""), Message::user("")];
        let count = counter.count_messages(&messages);
        // 0 chars / 3.5 = 0 + 2 * 4 overhead = 8
        assert_eq!(count, 8);
    }

    #[test]
    fn test_large_tool_result() {
        let counter = TokenCounter::default();
        let large_content = "x".repeat(10_000);
        let messages = vec![Message::tool_result("id".to_string(), large_content)];
        let count = counter.count_messages(&messages);
        // 2 + 10000 = 10002 chars / 3.5 = 2857 + 4 = 2861
        let expected = (10002_f64 / 3.5) as usize + 4;
        assert_eq!(count, expected);
    }
}
