/// Tool dispatcher for executing tool calls
///
/// This module provides the interface for dispatching and executing tool calls

use std::sync::Arc;

use crate::error::ToolError;
use crate::tool::{ToolCall, ToolContext, ToolOutput};
use crate::tool::registry::ToolRegistry;
use async_trait::async_trait;

/// Tool dispatcher trait
#[async_trait]
pub trait ToolDispatcher: Send + Sync {
    /// Execute a tool call
    ///
    /// # Arguments
    ///
    /// * `call` - The tool call to execute
    ///
    /// # Returns
    ///
    /// The tool output as a string
    async fn execute(&self, call: ToolCall) -> Result<String, ToolError>;
}

/// Simple in-memory tool dispatcher for testing and basic use
pub struct SimpleToolDispatcher;

#[async_trait]
impl ToolDispatcher for SimpleToolDispatcher {
    async fn execute(&self, _call: ToolCall) -> Result<String, ToolError> {
        Ok("Tool execution placeholder".to_string())
    }
}

/// Registry-backed tool dispatcher that routes `ToolCall`s to registered tools.
///
/// Uses an `Arc<ToolRegistry>` to look up tools by name and execute them.
pub struct RegistryToolDispatcher {
    registry: Arc<ToolRegistry>,
}

impl RegistryToolDispatcher {
    /// Create a new `RegistryToolDispatcher` wrapping the given registry.
    pub fn new(registry: Arc<ToolRegistry>) -> Self {
        Self { registry }
    }
}

#[async_trait]
impl ToolDispatcher for RegistryToolDispatcher {
    async fn execute(&self, call: ToolCall) -> Result<String, ToolError> {
        let ctx = ToolContext::new(call.id.clone());
        let output = self.registry.execute_tool(&call.name, &call.arguments, &ctx).await?;
        Ok(tool_output_to_string(output))
    }
}

/// Convert a `ToolOutput` into a plain string representation.
fn tool_output_to_string(output: ToolOutput) -> String {
    match output {
        ToolOutput::Text(text) => text,
        ToolOutput::Image { mime_type, data } => {
            format!("[Image: {}, {} bytes]", mime_type, data.len())
        }
        ToolOutput::Video { mime_type, data } => {
            format!("[Video: {}, {} bytes]", mime_type, data.len())
        }
        ToolOutput::Audio { mime_type, data } => {
            format!("[Audio: {}, {} bytes]", mime_type, data.len())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tool::{Tool, ToolMetadata};
    use serde_json::json;

    /// A simple echo tool for testing that returns whatever input it receives.
    struct EchoTool;

    #[async_trait]
    impl Tool for EchoTool {
        fn metadata(&self) -> ToolMetadata {
            ToolMetadata {
                name: "echo".to_string(),
                description: "Echoes the input back".to_string(),
                input_schema: json!({"type": "object"}),
            }
        }

        async fn execute(&self, input: &str, _ctx: &ToolContext) -> Result<ToolOutput, ToolError> {
            Ok(ToolOutput::Text(input.to_string()))
        }
    }

    /// A tool that always fails, for testing error propagation.
    struct FailTool;

    #[async_trait]
    impl Tool for FailTool {
        fn metadata(&self) -> ToolMetadata {
            ToolMetadata {
                name: "fail".to_string(),
                description: "Always fails".to_string(),
                input_schema: json!({"type": "object"}),
            }
        }

        async fn execute(&self, _input: &str, _ctx: &ToolContext) -> Result<ToolOutput, ToolError> {
            Err(ToolError::ExecutionFailed("intentional failure".to_string()))
        }
    }

    #[tokio::test]
    async fn test_registry_dispatcher_executes_registered_tool() {
        let registry = Arc::new(ToolRegistry::new());
        registry.register(Box::new(EchoTool)).await;

        let dispatcher = RegistryToolDispatcher::new(registry);
        let call = ToolCall {
            id: "call-1".to_string(),
            name: "echo".to_string(),
            arguments: "hello world".to_string(),
        };

        let result = dispatcher.execute(call).await.unwrap();
        assert_eq!(result, "hello world");
    }

    #[tokio::test]
    async fn test_registry_dispatcher_returns_not_found_for_unknown_tool() {
        let registry = Arc::new(ToolRegistry::new());
        let dispatcher = RegistryToolDispatcher::new(registry);
        let call = ToolCall {
            id: "call-2".to_string(),
            name: "nonexistent".to_string(),
            arguments: "{}".to_string(),
        };

        let err = dispatcher.execute(call).await.unwrap_err();
        match err {
            ToolError::NotFound(name) => assert_eq!(name, "nonexistent"),
            other => panic!("expected NotFound, got: {:?}", other),
        }
    }

    #[tokio::test]
    async fn test_registry_dispatcher_propagates_tool_errors() {
        let registry = Arc::new(ToolRegistry::new());
        registry.register(Box::new(FailTool)).await;

        let dispatcher = RegistryToolDispatcher::new(registry);
        let call = ToolCall {
            id: "call-3".to_string(),
            name: "fail".to_string(),
            arguments: "{}".to_string(),
        };

        let err = dispatcher.execute(call).await.unwrap_err();
        match err {
            ToolError::ExecutionFailed(msg) => assert_eq!(msg, "intentional failure"),
            other => panic!("expected ExecutionFailed, got: {:?}", other),
        }
    }

    #[tokio::test]
    async fn test_simple_dispatcher_returns_placeholder() {
        let dispatcher = SimpleToolDispatcher;
        let call = ToolCall {
            id: "call-4".to_string(),
            name: "anything".to_string(),
            arguments: "{}".to_string(),
        };

        let result = dispatcher.execute(call).await.unwrap();
        assert_eq!(result, "Tool execution placeholder");
    }

    #[test]
    fn test_tool_output_to_string_text() {
        let output = ToolOutput::Text("hello".to_string());
        assert_eq!(tool_output_to_string(output), "hello");
    }

    #[test]
    fn test_tool_output_to_string_image() {
        let output = ToolOutput::Image {
            data: vec![0u8; 100],
            mime_type: "image/png".to_string(),
        };
        assert_eq!(tool_output_to_string(output), "[Image: image/png, 100 bytes]");
    }

    #[test]
    fn test_tool_output_to_string_video() {
        let output = ToolOutput::Video {
            data: vec![0u8; 2048],
            mime_type: "video/mp4".to_string(),
        };
        assert_eq!(tool_output_to_string(output), "[Video: video/mp4, 2048 bytes]");
    }

    #[test]
    fn test_tool_output_to_string_audio() {
        let output = ToolOutput::Audio {
            data: vec![0u8; 512],
            mime_type: "audio/wav".to_string(),
        };
        assert_eq!(tool_output_to_string(output), "[Audio: audio/wav, 512 bytes]");
    }
}
