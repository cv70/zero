use crate::error::ProviderError;
/// High-level Provider interface for Agent Loop
///
/// This module provides a simplified provider interface tailored for the Agent loop,
/// abstracted away from the low-level API details of different LLM providers.
use crate::message::{ContentBlock, Message};
use async_trait::async_trait;
use futures_core::Stream;
use serde::{Deserialize, Serialize};
use std::pin::Pin;

/// Response from LLM provider
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderResponse {
    /// Content blocks in the response
    pub content: Vec<ContentBlock>,

    /// Stop reason: "end_turn", "tool_use", "max_tokens", etc.
    pub stop_reason: String,
}

impl ProviderResponse {
    /// Create a new provider response
    pub fn new(content: Vec<ContentBlock>, stop_reason: impl Into<String>) -> Self {
        Self {
            content,
            stop_reason: stop_reason.into(),
        }
    }

    /// Check if response contains tool use
    pub fn has_tool_use(&self) -> bool {
        self.content.iter().any(|block| block.is_tool_use())
    }

    /// Get all tool use blocks
    pub fn tool_uses(&self) -> Vec<(String, String, serde_json::Value)> {
        self.content
            .iter()
            .filter_map(|block| match block {
                ContentBlock::ToolUse { id, name, input } => {
                    Some((id.clone(), name.clone(), input.clone()))
                }
                _ => None,
            })
            .collect()
    }

    /// Get first text block if present
    pub fn first_text(&self) -> Option<String> {
        self.content.iter().find_map(|block| match block {
            ContentBlock::Text { text } => Some(text.clone()),
            _ => None,
        })
    }

    /// Get all text blocks
    pub fn all_text(&self) -> Vec<String> {
        self.content
            .iter()
            .filter_map(|block| match block {
                ContentBlock::Text { text } => Some(text.clone()),
                _ => None,
            })
            .collect()
    }
}

/// High-level provider interface for Agent Loop
///
/// This trait represents a simplified view of an LLM provider,
/// focused on the needs of the Agent loop rather than raw API details.
#[async_trait]
pub trait LoopProvider: Send + Sync {
    /// Get provider name
    fn name(&self) -> &str;

    /// Complete a conversation with the LLM
    ///
    /// The provider receives the full message history and returns
    /// a response with content blocks and stop reason.
    async fn complete(&self, messages: &[Message]) -> Result<ProviderResponse, ProviderError>;
}

/// Events emitted during streaming response
#[derive(Debug, Clone)]
pub enum StreamEvent {
    /// Incremental text content
    TextDelta(String),
    /// Tool use block starting
    ToolUseStart { id: String, name: String },
    /// Incremental tool input JSON
    ToolUseInputDelta(String),
    /// A content block has finished
    ContentBlockStop,
    /// The entire message is done
    MessageStop { stop_reason: String },
}

/// Provider that supports streaming responses
#[async_trait]
pub trait StreamingLoopProvider: LoopProvider {
    /// Stream a completion response
    async fn complete_stream(
        &self,
        messages: &[Message],
    ) -> Result<Pin<Box<dyn Stream<Item = Result<StreamEvent, ProviderError>> + Send>>, ProviderError>;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_provider_response_creation() {
        let response = ProviderResponse::new(vec![ContentBlock::text("Hello")], "end_turn");
        assert_eq!(response.stop_reason, "end_turn");
        assert!(!response.has_tool_use());
    }

    #[test]
    fn test_has_tool_use() {
        let response = ProviderResponse::new(
            vec![ContentBlock::tool_use(
                "1".to_string(),
                "bash".to_string(),
                serde_json::json!({}),
            )],
            "tool_use",
        );
        assert!(response.has_tool_use());
    }

    #[test]
    fn test_tool_uses_extraction() {
        let response = ProviderResponse::new(
            vec![
                ContentBlock::tool_use(
                    "1".to_string(),
                    "bash".to_string(),
                    serde_json::json!({"cmd": "ls"}),
                ),
                ContentBlock::tool_use("2".to_string(), "read".to_string(), serde_json::json!({})),
            ],
            "tool_use",
        );

        let uses = response.tool_uses();
        assert_eq!(uses.len(), 2);
        assert_eq!(uses[0].0, "1");
        assert_eq!(uses[0].1, "bash");
    }

    #[test]
    fn test_text_extraction() {
        let response = ProviderResponse::new(
            vec![
                ContentBlock::text("First"),
                ContentBlock::text("Second"),
                ContentBlock::tool_use("1".to_string(), "bash".to_string(), serde_json::json!({})),
            ],
            "mixed",
        );

        assert_eq!(response.first_text(), Some("First".to_string()));
        assert_eq!(response.all_text(), vec!["First", "Second"]);
    }

    // ── StreamEvent tests ─────────────────────────────────────────────

    #[test]
    fn test_stream_event_text_delta() {
        let event = StreamEvent::TextDelta("Hello".to_string());
        match event {
            StreamEvent::TextDelta(text) => assert_eq!(text, "Hello"),
            _ => panic!("Expected TextDelta"),
        }
    }

    #[test]
    fn test_stream_event_tool_use_start() {
        let event = StreamEvent::ToolUseStart {
            id: "toolu_1".to_string(),
            name: "bash".to_string(),
        };
        match event {
            StreamEvent::ToolUseStart { id, name } => {
                assert_eq!(id, "toolu_1");
                assert_eq!(name, "bash");
            }
            _ => panic!("Expected ToolUseStart"),
        }
    }

    #[test]
    fn test_stream_event_tool_use_input_delta() {
        let event = StreamEvent::ToolUseInputDelta("{\"command\":".to_string());
        match event {
            StreamEvent::ToolUseInputDelta(json) => assert_eq!(json, "{\"command\":"),
            _ => panic!("Expected ToolUseInputDelta"),
        }
    }

    #[test]
    fn test_stream_event_content_block_stop() {
        let event = StreamEvent::ContentBlockStop;
        assert!(matches!(event, StreamEvent::ContentBlockStop));
    }

    #[test]
    fn test_stream_event_message_stop() {
        let event = StreamEvent::MessageStop {
            stop_reason: "end_turn".to_string(),
        };
        match event {
            StreamEvent::MessageStop { stop_reason } => assert_eq!(stop_reason, "end_turn"),
            _ => panic!("Expected MessageStop"),
        }
    }

    #[test]
    fn test_stream_event_clone() {
        let event = StreamEvent::TextDelta("test".to_string());
        let cloned = event.clone();
        match cloned {
            StreamEvent::TextDelta(text) => assert_eq!(text, "test"),
            _ => panic!("Expected TextDelta"),
        }
    }
}
