/// Context manager that compacts messages when they exceed token limits
///
/// Implements a 3-layer compression strategy inspired by learn-claude-code:
/// - Layer 1: LLM summarization (intentionally omitted for now)
/// - Layer 2: Trim long tool results, keeping first + last chars
/// - Layer 3: Drop oldest messages, keeping first user message and recent context

use super::token_counter::TokenCounter;
use crate::message::{Message, ToolResultContent};

/// Context manager that compacts messages when they exceed token limits
pub struct ContextManager {
    counter: TokenCounter,
    /// Maximum context tokens before compaction triggers
    max_context_tokens: usize,
    /// Threshold ratio to trigger compaction (0.0-1.0, default 0.7)
    compaction_threshold: f64,
    /// Max chars to keep per tool result during L2 trim
    max_tool_result_chars: usize,
}

impl ContextManager {
    pub fn new(max_context_tokens: usize) -> Self {
        Self {
            counter: TokenCounter::default(),
            max_context_tokens,
            compaction_threshold: 0.7,
            max_tool_result_chars: 1000,
        }
    }

    pub fn with_counter(mut self, counter: TokenCounter) -> Self {
        self.counter = counter;
        self
    }

    pub fn with_threshold(mut self, threshold: f64) -> Self {
        self.compaction_threshold = threshold;
        self
    }

    pub fn with_max_tool_result_chars(mut self, max: usize) -> Self {
        self.max_tool_result_chars = max;
        self
    }

    /// Check if compaction is needed and apply if so
    pub fn compact_if_needed(&self, messages: &mut Vec<Message>) {
        let current_tokens = self.counter.count_messages(messages);
        let threshold = (self.max_context_tokens as f64 * self.compaction_threshold) as usize;

        if current_tokens <= threshold {
            return;
        }

        // Layer 2: Trim long tool results first (cheapest operation)
        self.trim_tool_results(messages);

        // Check again
        let current_tokens = self.counter.count_messages(messages);
        if current_tokens <= threshold {
            return;
        }

        // Layer 3: Drop oldest messages (keep first message + recent messages)
        self.drop_oldest(messages);
    }

    /// Layer 2: Trim oversized tool results, keeping first+last chars
    fn trim_tool_results(&self, messages: &mut Vec<Message>) {
        for msg in messages.iter_mut() {
            if let Message::ToolResult { content } = msg {
                for tr in content.iter_mut() {
                    if let ToolResultContent::ToolResult { content, .. } = tr {
                        if content.len() > self.max_tool_result_chars {
                            let half = self.max_tool_result_chars / 2;
                            let start = &content[..half];
                            let end = &content[content.len() - half..];
                            let truncated_count =
                                content.len() - self.max_tool_result_chars;
                            *content = format!(
                                "{}...\n[{} chars truncated]\n...{}",
                                start, truncated_count, end
                            );
                        }
                    }
                }
            }
        }
    }

    /// Layer 3: Drop oldest message pairs, keeping first user message and recent context
    fn drop_oldest(&self, messages: &mut Vec<Message>) {
        // Keep at least the first message and last 6 messages (3 turns)
        let min_keep = 6;
        if messages.len() <= min_keep + 1 {
            return;
        }

        let threshold = (self.max_context_tokens as f64 * self.compaction_threshold) as usize;

        // Remove messages from index 1 (after first) until within budget
        while messages.len() > min_keep + 1 {
            let tokens = self.counter.count_messages(messages);
            if tokens <= threshold {
                break;
            }
            messages.remove(1);
        }
    }

    /// Get the current estimated token count
    pub fn estimate_tokens(&self, messages: &[Message]) -> usize {
        self.counter.count_messages(messages)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::message::ContentBlock;
    use serde_json::json;

    fn make_user_msg(text: &str) -> Message {
        Message::user(text)
    }

    fn make_assistant_msg(text: &str) -> Message {
        Message::assistant(vec![ContentBlock::text(text)])
    }

    fn make_tool_result(id: &str, content: &str) -> Message {
        Message::tool_result(id.to_string(), content)
    }

    #[test]
    fn test_new_default_values() {
        let cm = ContextManager::new(100_000);
        assert_eq!(cm.max_context_tokens, 100_000);
        assert!((cm.compaction_threshold - 0.7).abs() < f64::EPSILON);
        assert_eq!(cm.max_tool_result_chars, 1000);
    }

    #[test]
    fn test_builder_with_counter() {
        let cm = ContextManager::new(100_000)
            .with_counter(TokenCounter::openai());
        // Should use openai counter (4.0 chars per token)
        let messages = vec![Message::user("a".repeat(100))];
        let tokens = cm.estimate_tokens(&messages);
        // 100 / 4.0 = 25 + 4 overhead = 29
        assert_eq!(tokens, 29);
    }

    #[test]
    fn test_builder_with_threshold() {
        let cm = ContextManager::new(100_000)
            .with_threshold(0.5);
        assert!((cm.compaction_threshold - 0.5).abs() < f64::EPSILON);
    }

    #[test]
    fn test_builder_with_max_tool_result_chars() {
        let cm = ContextManager::new(100_000)
            .with_max_tool_result_chars(500);
        assert_eq!(cm.max_tool_result_chars, 500);
    }

    #[test]
    fn test_no_compaction_when_under_threshold() {
        let cm = ContextManager::new(100_000);
        let mut messages = vec![
            make_user_msg("Hello"),
            make_assistant_msg("Hi there!"),
        ];
        let original_len = messages.len();
        cm.compact_if_needed(&mut messages);
        assert_eq!(messages.len(), original_len);
    }

    #[test]
    fn test_estimate_tokens() {
        let cm = ContextManager::new(100_000);
        let messages = vec![make_user_msg("Hello, world!")];
        let tokens = cm.estimate_tokens(&messages);
        assert!(tokens > 0);
    }

    #[test]
    fn test_trim_tool_results_long_content() {
        let cm = ContextManager::new(100) // very small budget to force compaction
            .with_max_tool_result_chars(20);

        let long_content = "a".repeat(100);
        let mut messages = vec![
            make_user_msg("Do something"),
            make_tool_result("t1", &long_content),
        ];

        // Manually call trim_tool_results
        cm.trim_tool_results(&mut messages);

        // Check that the tool result was trimmed
        if let Message::ToolResult { content } = &messages[1] {
            if let ToolResultContent::ToolResult { content, .. } = &content[0] {
                assert!(content.len() < 100);
                assert!(content.contains("chars truncated"));
            } else {
                panic!("Expected ToolResult variant");
            }
        } else {
            panic!("Expected ToolResult message");
        }
    }

    #[test]
    fn test_trim_tool_results_short_content_untouched() {
        let cm = ContextManager::new(100_000)
            .with_max_tool_result_chars(1000);

        let short_content = "short result";
        let mut messages = vec![
            make_user_msg("Do something"),
            make_tool_result("t1", short_content),
        ];

        cm.trim_tool_results(&mut messages);

        // Short content should be untouched
        if let Message::ToolResult { content } = &messages[1] {
            if let ToolResultContent::ToolResult { content, .. } = &content[0] {
                assert_eq!(content, "short result");
            }
        }
    }

    #[test]
    fn test_trim_preserves_start_and_end() {
        let cm = ContextManager::new(100)
            .with_max_tool_result_chars(20);

        // Create content where first 10 chars are 'A' and last 10 are 'Z'
        let content = format!("{}{}{}", "A".repeat(10), "B".repeat(80), "Z".repeat(10));
        let mut messages = vec![
            make_user_msg("test"),
            make_tool_result("t1", &content),
        ];

        cm.trim_tool_results(&mut messages);

        if let Message::ToolResult { content: ref results } = messages[1] {
            if let ToolResultContent::ToolResult { content, .. } = &results[0] {
                // Should start with 'A's (first half = 10 chars)
                assert!(content.starts_with("AAAAAAAAAA"));
                // Should end with 'Z's (last half = 10 chars)
                assert!(content.ends_with("ZZZZZZZZZZ"));
            }
        }
    }

    #[test]
    fn test_drop_oldest_preserves_first_and_recent() {
        // Use a very small token budget so compaction triggers
        let cm = ContextManager::new(50)
            .with_threshold(0.1); // very low threshold = 5 tokens

        let mut messages = vec![
            make_user_msg("First message"),         // index 0 - preserved
            make_assistant_msg("Response 1"),        // index 1 - may be dropped
            make_user_msg("Second"),                 // index 2 - may be dropped
            make_assistant_msg("Response 2"),        // index 3 - may be dropped
            make_user_msg("Third"),                  // index 4
            make_assistant_msg("Response 3"),        // index 5
            make_user_msg("Fourth"),                 // index 6
            make_assistant_msg("Response 4"),        // index 7
            make_user_msg("Fifth"),                  // index 8
            make_assistant_msg("Final response"),    // index 9
        ];

        cm.drop_oldest(&mut messages);

        // First message should always be preserved
        assert!(messages[0].is_user());
        assert_eq!(messages[0].user_content(), Some("First message"));

        // Should have dropped some middle messages
        assert!(messages.len() < 10);
        // Should keep at least min_keep + 1 = 7
        assert!(messages.len() >= 7);
    }

    #[test]
    fn test_drop_oldest_no_drop_when_few_messages() {
        let cm = ContextManager::new(10)
            .with_threshold(0.1); // very low threshold

        let mut messages = vec![
            make_user_msg("First"),
            make_assistant_msg("R1"),
            make_user_msg("Second"),
            make_assistant_msg("R2"),
        ];

        let original_len = messages.len();
        cm.drop_oldest(&mut messages);
        // Only 4 messages, <= min_keep + 1 = 7, so no drop
        assert_eq!(messages.len(), original_len);
    }

    #[test]
    fn test_compact_if_needed_full_pipeline() {
        // Create a scenario with large tool results that exceeds the token budget
        let cm = ContextManager::new(200)
            .with_threshold(0.5) // threshold at 100 tokens
            .with_max_tool_result_chars(50);

        let large_result = "x".repeat(2000);
        let mut messages = vec![
            make_user_msg("Do something"),
            Message::assistant(vec![ContentBlock::tool_use(
                "t1".to_string(),
                "bash".to_string(),
                json!({"command": "ls"}),
            )]),
            make_tool_result("t1", &large_result),
            make_assistant_msg("Got it"),
            make_user_msg("Now do more"),
            Message::assistant(vec![ContentBlock::tool_use(
                "t2".to_string(),
                "bash".to_string(),
                json!({"command": "cat big.txt"}),
            )]),
            make_tool_result("t2", &large_result),
            make_assistant_msg("Here are the results"),
        ];

        let before_tokens = cm.estimate_tokens(&messages);
        cm.compact_if_needed(&mut messages);
        let after_tokens = cm.estimate_tokens(&messages);

        // After compaction, tokens should be reduced
        assert!(after_tokens <= before_tokens);
    }

    #[test]
    fn test_compact_trims_before_dropping() {
        // Set up so trimming alone is enough to bring tokens under the threshold.
        // large_result = 10000 chars => ~2857 tokens + overhead ~12 = ~2869
        // threshold = 5000 * 0.5 = 2500 tokens => compaction triggers
        // After trimming to 50 chars: ~20 tokens + overhead ~12 = ~32 => well under 2500
        let cm = ContextManager::new(5000)
            .with_threshold(0.5) // threshold = 2500 tokens
            .with_max_tool_result_chars(50);

        let large_result = "x".repeat(10_000);
        let mut messages = vec![
            make_user_msg("Hello"),
            make_tool_result("t1", &large_result),
            make_assistant_msg("Done"),
        ];

        let original_count = messages.len();
        cm.compact_if_needed(&mut messages);

        // Trimming should have been sufficient, no messages dropped
        assert_eq!(messages.len(), original_count);

        // But the tool result should be trimmed
        if let Message::ToolResult { content } = &messages[1] {
            if let ToolResultContent::ToolResult { content, .. } = &content[0] {
                assert!(content.len() < 10_000);
                assert!(content.contains("chars truncated"));
            }
        }
    }

    #[test]
    fn test_multiple_tool_results_all_trimmed() {
        let cm = ContextManager::new(100)
            .with_max_tool_result_chars(20);

        let mut messages = vec![
            make_user_msg("test"),
            Message::tool_results(vec![
                ("t1".to_string(), "a".repeat(100)),
                ("t2".to_string(), "b".repeat(100)),
            ]),
        ];

        cm.trim_tool_results(&mut messages);

        if let Message::ToolResult { content } = &messages[1] {
            for tr in content {
                if let ToolResultContent::ToolResult { content, .. } = tr {
                    assert!(content.len() < 100);
                    assert!(content.contains("chars truncated"));
                }
            }
        }
    }

    #[test]
    fn test_user_messages_not_affected_by_trim() {
        let cm = ContextManager::new(100)
            .with_max_tool_result_chars(10);

        let long_user_msg = "x".repeat(500);
        let mut messages = vec![make_user_msg(&long_user_msg)];

        cm.trim_tool_results(&mut messages);

        // User message content should be unchanged
        assert_eq!(messages[0].user_content(), Some(long_user_msg.as_str()));
    }
}
