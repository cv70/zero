/// Message types for Agent communication
///
/// This module defines the message types used throughout the Agent loop.
/// Each message represents a turn in the conversation between the user, the Agent,
/// and various tools.
use serde::{Deserialize, Serialize};

/// Represents a single message in the Agent conversation.
///
/// Messages flow in this pattern:
/// 1. User sends a request
/// 2. Agent responds (may contain tool calls or text)
/// 3. Tools are executed
/// 4. Tool results are appended
/// 5. Loop continues until agent returns end_turn
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Message {
    /// User message with text content
    User { content: String },

    /// Assistant message with content blocks (text and tool calls)
    Assistant { content: Vec<ContentBlock> },

    /// Tool execution result
    ToolResult { content: Vec<ToolResultContent> },
}

/// Content blocks within an Assistant message
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ContentBlock {
    /// Text content from the assistant
    Text { text: String },

    /// Tool use request from the assistant
    ToolUse {
        id: String,
        name: String,
        input: serde_json::Value,
    },
}

/// Tool result content for the ToolResult message
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ToolResultContent {
    /// Tool execution result
    ToolResult {
        tool_use_id: String,
        content: String,
    },
}

// Convenient constructors and helpers
impl Message {
    /// Create a new user message
    pub fn user(content: impl Into<String>) -> Self {
        Message::User {
            content: content.into(),
        }
    }

    /// Create a new assistant message with content blocks
    pub fn assistant(blocks: Vec<ContentBlock>) -> Self {
        Message::Assistant { content: blocks }
    }

    /// Create a new tool result message
    pub fn tool_result(tool_use_id: String, content: impl Into<String>) -> Self {
        Message::ToolResult {
            content: vec![ToolResultContent::ToolResult {
                tool_use_id,
                content: content.into(),
            }],
        }
    }

    /// Create a new tool result message with multiple results
    pub fn tool_results(results: Vec<(String, String)>) -> Self {
        let content = results
            .into_iter()
            .map(|(tool_use_id, content)| ToolResultContent::ToolResult {
                tool_use_id,
                content,
            })
            .collect();
        Message::ToolResult { content }
    }

    /// Check if this is a user message
    pub fn is_user(&self) -> bool {
        matches!(self, Message::User { .. })
    }

    /// Check if this is an assistant message
    pub fn is_assistant(&self) -> bool {
        matches!(self, Message::Assistant { .. })
    }

    /// Check if this is a tool result message
    pub fn is_tool_result(&self) -> bool {
        matches!(self, Message::ToolResult { .. })
    }

    /// Get the string content if this is a user message
    pub fn user_content(&self) -> Option<&str> {
        match self {
            Message::User { content } => Some(content),
            _ => None,
        }
    }

    /// Get all text blocks from an assistant message
    pub fn assistant_text_blocks(&self) -> Vec<String> {
        match self {
            Message::Assistant { content } => content
                .iter()
                .filter_map(|block| match block {
                    ContentBlock::Text { text } => Some(text.clone()),
                    _ => None,
                })
                .collect(),
            _ => vec![],
        }
    }

    /// Get all tool use blocks from an assistant message
    pub fn assistant_tool_uses(&self) -> Vec<(String, String, serde_json::Value)> {
        match self {
            Message::Assistant { content } => content
                .iter()
                .filter_map(|block| match block {
                    ContentBlock::ToolUse { id, name, input } => {
                        Some((id.clone(), name.clone(), input.clone()))
                    }
                    _ => None,
                })
                .collect(),
            _ => vec![],
        }
    }
}

impl ContentBlock {
    /// Create a new text block
    pub fn text(text: impl Into<String>) -> Self {
        ContentBlock::Text { text: text.into() }
    }

    /// Create a new tool use block
    pub fn tool_use(id: String, name: String, input: serde_json::Value) -> Self {
        ContentBlock::ToolUse { id, name, input }
    }

    /// Check if this is a text block
    pub fn is_text(&self) -> bool {
        matches!(self, ContentBlock::Text { .. })
    }

    /// Check if this is a tool use block
    pub fn is_tool_use(&self) -> bool {
        matches!(self, ContentBlock::ToolUse { .. })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_user_message_creation() {
        let msg = Message::user("Hello, world!");
        assert!(msg.is_user());
        assert_eq!(msg.user_content(), Some("Hello, world!"));
    }

    #[test]
    fn test_assistant_message_with_text() {
        let msg = Message::assistant(vec![ContentBlock::text("Response text")]);
        assert!(msg.is_assistant());
        assert_eq!(msg.assistant_text_blocks(), vec!["Response text"]);
    }

    #[test]
    fn test_assistant_message_with_tool_use() {
        let tool_use = ContentBlock::tool_use(
            "1".to_string(),
            "bash".to_string(),
            serde_json::json!({"command": "ls"}),
        );
        let msg = Message::assistant(vec![tool_use]);
        assert!(msg.is_assistant());

        let tool_uses = msg.assistant_tool_uses();
        assert_eq!(tool_uses.len(), 1);
        assert_eq!(tool_uses[0].0, "1");
        assert_eq!(tool_uses[0].1, "bash");
    }

    #[test]
    fn test_tool_result_message() {
        let msg = Message::tool_result("1".to_string(), "result content");
        assert!(msg.is_tool_result());
    }

    #[test]
    fn test_tool_result_serialization() {
        let msg = Message::tool_result("1".to_string(), "output");
        let json = serde_json::to_string(&msg).unwrap();
        let parsed: Message = serde_json::from_str(&json).unwrap();
        assert!(parsed.is_tool_result());
    }

    #[test]
    fn test_message_serialization_roundtrip() {
        let original = Message::assistant(vec![
            ContentBlock::text("Hello"),
            ContentBlock::tool_use(
                "1".to_string(),
                "bash".to_string(),
                serde_json::json!({"command": "echo test"}),
            ),
        ]);

        let json = serde_json::to_string(&original).unwrap();
        let deserialized: Message = serde_json::from_str(&json).unwrap();

        match deserialized {
            Message::Assistant { content } => {
                assert_eq!(content.len(), 2);
                assert!(content[0].is_text());
                assert!(content[1].is_tool_use());
            }
            _ => panic!("Expected assistant message"),
        }
    }

    #[test]
    fn test_mixed_content_blocks() {
        let blocks = vec![
            ContentBlock::text("Starting task"),
            ContentBlock::tool_use(
                "1".to_string(),
                "read_file".to_string(),
                serde_json::json!({"path": "README.md"}),
            ),
            ContentBlock::text("Analysis complete"),
        ];
        let msg = Message::assistant(blocks);

        assert_eq!(msg.assistant_text_blocks().len(), 2);
        assert_eq!(msg.assistant_tool_uses().len(), 1);
    }
}
