/// Tool dispatcher for executing tool calls
///
/// This module provides the interface for dispatching and executing tool calls

use crate::error::ToolError;
use crate::tool::ToolCall;
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
