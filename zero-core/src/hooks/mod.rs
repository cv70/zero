use crate::error::{MemoryError, ProviderError};
use async_trait::async_trait;

/// Hook trait for all hook types
#[async_trait]
pub trait Hook: Send + Sync {
    /// Hook name
    fn name(&self) -> &str;

    /// Hook priority (lower = executed earlier)
    fn priority(&self) -> i32 {
        0
    }
}

/// Agent lifecycle hooks
#[async_trait]
pub trait AgentHook: Hook {
    /// Called before agent initialization
    async fn on_agent_init(&self, _agent_name: &str) -> Result<(), String> {
        Ok(())
    }

    /// Called after agent initialization
    async fn on_agent_init_done(&self, _agent_name: &str) -> Result<(), String> {
        Ok(())
    }

    /// Called before agent run
    async fn on_agent_run(&self, _agent_name: &str) -> Result<(), String> {
        Ok(())
    }

    /// Called after agent run
    async fn on_agent_run_done(&self, _agent_name: &str, _result: &str) -> Result<(), String> {
        Ok(())
    }

    /// Called on agent error
    async fn on_agent_error(&self, _agent_name: &str, _error: &str) -> Result<(), String> {
        Ok(())
    }
}

/// Tool execution hooks
#[async_trait]
pub trait ToolHook: Hook {
    /// Called before tool validation
    async fn on_tool_validate(&self, _tool_name: &str, _input: &str) -> Result<(), String> {
        Ok(())
    }

    /// Called after tool validation
    async fn on_tool_validate_done(&self, _tool_name: &str, _input: &str) -> Result<(), String> {
        Ok(())
    }

    /// Called before tool execution
    async fn on_tool_execute(&self, _tool_name: &str, _input: &str) -> Result<(), String> {
        Ok(())
    }

    /// Called after tool execution
    async fn on_tool_execute_done(
        &self,
        _tool_name: &str,
        _input: &str,
        _result: &str,
    ) -> Result<(), String> {
        Ok(())
    }

    /// Called on tool error
    async fn on_tool_error(
        &self,
        _tool_name: &str,
        _input: &str,
        _error: &str,
    ) -> Result<(), String> {
        Ok(())
    }
}

/// Channel message hooks
#[async_trait]
pub trait ChannelHook: Hook {
    /// Called before message sending
    async fn on_message_send(
        &self,
        _channel_name: &str,
        _to: &str,
        _content: &str,
    ) -> Result<(), String> {
        Ok(())
    }

    /// Called after message sending
    async fn on_message_sent(
        &self,
        _channel_name: &str,
        _to: &str,
        _content: &str,
    ) -> Result<(), String> {
        Ok(())
    }

    /// Called before message receiving
    async fn on_message_receive(&self, _channel_name: &str) -> Result<(), String> {
        Ok(())
    }

    /// Called after message receiving
    async fn on_message_received(
        &self,
        _channel_name: &str,
        _from: &str,
        _content: &str,
    ) -> Result<(), String> {
        Ok(())
    }

    /// Called on channel error
    async fn on_channel_error(&self, _channel_name: &str, _error: &str) -> Result<(), String> {
        Ok(())
    }
}

/// LLM Provider execution hooks

/// Memory execution hooks
#[async_trait]
pub trait MemoryHook: Hook {
    /// Called before memory get
    async fn on_memory_get(&self, _memory_name: &str, _key: &str) -> Result<(), MemoryError> {
        Ok(())
    }

    /// Called after memory get
    async fn on_memory_get_done(
        &self,
        _memory_name: &str,
        _key: &str,
        _value: &str,
    ) -> Result<(), MemoryError> {
        Ok(())
    }

    /// Called before memory set
    async fn on_memory_set(
        &self,
        _memory_name: &str,
        _key: &str,
        _value: &str,
    ) -> Result<(), MemoryError> {
        Ok(())
    }

    /// Called after memory set
    async fn on_memory_set_done(
        &self,
        _memory_name: &str,
        _key: &str,
        _value: &str,
    ) -> Result<(), MemoryError> {
        Ok(())
    }

    /// Called before memory delete
    async fn on_memory_delete(&self, _memory_name: &str, _key: &str) -> Result<(), MemoryError> {
        Ok(())
    }

    /// Called after memory delete
    async fn on_memory_delete_done(
        &self,
        _memory_name: &str,
        _key: &str,
        _result: &str,
    ) -> Result<(), MemoryError> {
        Ok(())
    }

    /// Called on memory error
    async fn on_memory_error(
        &self,
        _memory_name: &str,
        _key: &str,
        _error: &str,
    ) -> Result<(), MemoryError> {
        Ok(())
    }
}

/// LLM Provider execution hooks
#[async_trait]
pub trait ProviderHook: Hook {
    /// Called before provider call
    async fn on_provider_call(
        &self,
        _provider_name: &str,
        _request: &str,
    ) -> Result<(), ProviderError> {
        Ok(())
    }

    /// Called after provider response
    async fn on_provider_response(
        &self,
        _provider_name: &str,
        _request: &str,
        _response: &str,
    ) -> Result<(), ProviderError> {
        Ok(())
    }

    /// Called on provider error
    async fn on_provider_error(
        &self,
        _provider_name: &str,
        _request: &str,
        _error: &str,
    ) -> Result<(), ProviderError> {
        Ok(())
    }
}

/// Hook manager
pub mod manager;
pub use manager::HookManager;
