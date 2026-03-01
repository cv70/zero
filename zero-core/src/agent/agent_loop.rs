/// Core Agent execution loop
///
/// This module implements the fundamental Agent loop pattern:
/// ```text
/// while stop_reason == "tool_use":
///     response = provider.complete(messages, tools)
///     execute_tools(response)
///     append_results_to_messages()
/// return final_response
/// ```

use crate::error::AgentError;
use crate::message::{ContentBlock, Message};
use crate::provider::LoopProvider;
use crate::tool::{ToolDispatcher, ToolCall};
use crate::agent::loop_config::AgentLoopConfig;
use crate::hooks::HookManager;
use async_trait::async_trait;
use std::sync::Arc;

/// The core Agent loop trait
#[async_trait]
pub trait AgentLoop: Send + Sync {
    /// Execute the Agent loop until completion
    ///
    /// The loop will:
    /// 1. Call the LLM provider with current messages
    /// 2. Check the stop_reason:
    ///    - If "tool_use": execute tools and loop back
    ///    - Otherwise: return the final response
    ///
    /// # Arguments
    ///
    /// * `messages` - The message history (modified in place)
    /// * `config` - Configuration for loop behavior
    ///
    /// # Returns
    ///
    /// The final text response from the agent
    async fn execute(
        &self,
        messages: &mut Vec<Message>,
        config: &AgentLoopConfig,
    ) -> Result<String, AgentError>;
}

/// Default implementation of the Agent loop
pub struct DefaultAgentLoop {
    provider: Arc<dyn LoopProvider>,
    tool_dispatcher: Arc<dyn ToolDispatcher>,
    hooks: Option<Arc<HookManager>>,
}

impl DefaultAgentLoop {
    /// Create a new DefaultAgentLoop
    pub fn new(
        provider: Arc<dyn LoopProvider>,
        tool_dispatcher: Arc<dyn ToolDispatcher>,
    ) -> Self {
        Self {
            provider,
            tool_dispatcher,
            hooks: None,
        }
    }

    /// Add hook manager to this loop
    pub fn with_hooks(mut self, hooks: Arc<HookManager>) -> Self {
        self.hooks = Some(hooks);
        self
    }

    /// Get the provider
    pub fn provider(&self) -> &Arc<dyn LoopProvider> {
        &self.provider
    }

    /// Get the tool dispatcher
    pub fn tool_dispatcher(&self) -> &Arc<dyn ToolDispatcher> {
        &self.tool_dispatcher
    }

    /// Get the hooks if present
    pub fn hooks(&self) -> Option<&Arc<HookManager>> {
        self.hooks.as_ref()
    }
}

#[async_trait]
impl AgentLoop for DefaultAgentLoop {
    async fn execute(
        &self,
        messages: &mut Vec<Message>,
        config: &AgentLoopConfig,
    ) -> Result<String, AgentError> {
        let mut iteration = 0;
        let mut final_response = String::new();

        loop {
            // Check iteration limit
            if iteration >= config.max_iterations {
                return Err(AgentError::MaxIterationsExceeded(iteration));
            }
            iteration += 1;

            if config.verbose_logging {
                eprintln!("[AgentLoop] Iteration {} of {}", iteration, config.max_iterations);
            }

            // Call the LLM provider with timeout
            let response = tokio::time::timeout(
                config.provider_timeout_duration(),
                self.provider.complete(messages),
            )
            .await
            .map_err(|_| AgentError::ProviderTimeout)?
            .map_err(|e| AgentError::ProviderError(format!("{}", e)))?;

            if config.verbose_logging {
                eprintln!(
                    "[AgentLoop] Provider returned stop_reason: {}",
                    response.stop_reason
                );
            }

            // Append assistant message
            messages.push(Message::assistant(response.content.clone()));

            // Extract text response
            for block in &response.content {
                if let ContentBlock::Text { text } = block {
                    final_response = text.clone();
                }
            }

            // Check if we need to execute tools
            if response.stop_reason != "tool_use" {
                if config.verbose_logging {
                    eprintln!("[AgentLoop] Conversation complete");
                }
                return Ok(final_response);
            }

            // Execute tools and collect results
            let mut tool_results = Vec::new();

            for block in response.content {
                if let ContentBlock::ToolUse { id, name, input } = block {
                    if config.verbose_logging {
                        eprintln!("[AgentLoop] Executing tool: {}", name);
                    }

                    // Parse tool call arguments
                    let arguments = serde_json::to_string(&input)
                        .map_err(|e| AgentError::SerializationError(e.to_string()))?;

                    // Execute tool with timeout
                    let result = tokio::time::timeout(
                        config.tool_timeout_duration(),
                        self.tool_dispatcher.execute(ToolCall {
                            id: id.clone(),
                            name: name.clone(),
                            arguments,
                        }),
                    )
                    .await
                    .map_err(|_| AgentError::ToolTimeout)?
                    .unwrap_or_else(|e| format!("Tool error: {}", e));

                    if config.verbose_logging {
                        let preview = if result.len() > 100 {
                            format!("{}...", &result[..100])
                        } else {
                            result.clone()
                        };
                        eprintln!("[AgentLoop] Tool result: {}", preview);
                    }

                    tool_results.push((id, result));
                }
            }

            // Append tool results as a new message
            if !tool_results.is_empty() {
                let tool_result_contents = tool_results
                    .into_iter()
                    .map(|(id, content)| (id, content))
                    .collect();
                messages.push(Message::tool_results(tool_result_contents));
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::provider::{LoopProvider, ProviderResponse};
    use async_trait::async_trait;
    use std::sync::{atomic::AtomicUsize, atomic::Ordering};

    struct MockProvider {
        call_count: Arc<AtomicUsize>,
        responses: Vec<ProviderResponse>,
    }

    impl MockProvider {
        fn new(responses: Vec<ProviderResponse>) -> Self {
            Self {
                call_count: Arc::new(AtomicUsize::new(0)),
                responses,
            }
        }

        fn call_count(&self) -> usize {
            self.call_count.load(Ordering::Relaxed)
        }
    }

    #[async_trait]
    impl LoopProvider for MockProvider {
        async fn complete(&self, _messages: &[Message]) -> Result<ProviderResponse, crate::error::ProviderError> {
            let count = self.call_count.fetch_add(1, Ordering::Relaxed);
            if count < self.responses.len() {
                Ok(self.responses[count].clone())
            } else {
                Err(crate::error::ProviderError::ApiError(
                    "No more responses".to_string(),
                ))
            }
        }

        fn name(&self) -> &str {
            "mock"
        }
    }

    struct MockDispatcher;

    #[async_trait]
    impl ToolDispatcher for MockDispatcher {
        async fn execute(
            &self,
            call: ToolCall,
        ) -> Result<String, crate::error::ToolError> {
            Ok(format!("Executed {}", call.name))
        }
    }

    #[tokio::test]
    async fn test_simple_loop_single_response() {
        let response = ProviderResponse {
            content: vec![ContentBlock::text("Hello, world!")],
            stop_reason: "end_turn".to_string(),
        };

        let provider = Arc::new(MockProvider::new(vec![response]));
        let dispatcher = Arc::new(MockDispatcher);
        let loop_impl = DefaultAgentLoop::new(provider.clone(), dispatcher);

        let mut messages = vec![Message::user("Say hello")];
        let config = AgentLoopConfig::default();

        let result = loop_impl.execute(&mut messages, &config).await;

        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "Hello, world!");
        assert_eq!(messages.len(), 2); // user + assistant
        assert_eq!(provider.call_count(), 1);
    }

    #[tokio::test]
    async fn test_loop_with_tool_execution() {
        let tool_response = ProviderResponse {
            content: vec![ContentBlock::tool_use(
                "1".to_string(),
                "test_tool".to_string(),
                serde_json::json!({"param": "value"}),
            )],
            stop_reason: "tool_use".to_string(),
        };

        let final_response = ProviderResponse {
            content: vec![ContentBlock::text("Task completed")],
            stop_reason: "end_turn".to_string(),
        };

        let provider = Arc::new(MockProvider::new(vec![tool_response, final_response]));
        let dispatcher = Arc::new(MockDispatcher);
        let loop_impl = DefaultAgentLoop::new(provider.clone(), dispatcher);

        let mut messages = vec![Message::user("Do something")];
        let config = AgentLoopConfig::default();

        let result = loop_impl.execute(&mut messages, &config).await;

        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "Task completed");
        // user + tool_use + tool_result + final_response
        assert_eq!(messages.len(), 4);
        assert_eq!(provider.call_count(), 2);
    }

    #[tokio::test]
    async fn test_max_iterations_exceeded() {
        let response = ProviderResponse {
            content: vec![ContentBlock::tool_use(
                "1".to_string(),
                "test".to_string(),
                serde_json::json!({}),
            )],
            stop_reason: "tool_use".to_string(),
        };

        let provider = Arc::new(MockProvider::new(vec![response; 100]));
        let dispatcher = Arc::new(MockDispatcher);
        let loop_impl = DefaultAgentLoop::new(provider, dispatcher);

        let mut messages = vec![Message::user("Infinite loop")];
        let config = AgentLoopConfig::default().with_max_iterations(5);

        let result = loop_impl.execute(&mut messages, &config).await;

        assert!(matches!(result, Err(AgentError::MaxIterationsExceeded(_))));
    }

    #[tokio::test]
    async fn test_builder_pattern() {
        let provider = Arc::new(MockProvider::new(vec![]));
        let dispatcher = Arc::new(MockDispatcher);
        let hooks = Arc::new(HookManager::new());

        let loop_impl = DefaultAgentLoop::new(
            provider as Arc<dyn LoopProvider>,
            dispatcher as Arc<dyn ToolDispatcher>,
        ).with_hooks(hooks.clone());

        assert!(loop_impl.hooks().is_some());
    }

    #[tokio::test]
    async fn test_verbose_logging() {
        let response = ProviderResponse {
            content: vec![ContentBlock::text("Hello")],
            stop_reason: "end_turn".to_string(),
        };

        let provider = Arc::new(MockProvider::new(vec![response]));
        let dispatcher = Arc::new(MockDispatcher);
        let loop_impl = DefaultAgentLoop::new(provider, dispatcher);

        let mut messages = vec![Message::user("Test")];
        let config = AgentLoopConfig::default().with_verbose_logging(true);

        let result = loop_impl.execute(&mut messages, &config).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_multiple_tool_uses() {
        let tool_response1 = ProviderResponse {
            content: vec![
                ContentBlock::tool_use(
                    "1".to_string(),
                    "tool_a".to_string(),
                    serde_json::json!({}),
                ),
                ContentBlock::tool_use(
                    "2".to_string(),
                    "tool_b".to_string(),
                    serde_json::json!({}),
                ),
            ],
            stop_reason: "tool_use".to_string(),
        };

        let final_response = ProviderResponse {
            content: vec![ContentBlock::text("All done")],
            stop_reason: "end_turn".to_string(),
        };

        let provider = Arc::new(MockProvider::new(vec![tool_response1, final_response]));
        let dispatcher = Arc::new(MockDispatcher);
        let loop_impl = DefaultAgentLoop::new(provider, dispatcher);

        let mut messages = vec![Message::user("Do multiple tasks")];
        let config = AgentLoopConfig::default();

        let result = loop_impl.execute(&mut messages, &config).await;

        assert!(result.is_ok());
        // user + tool_use + tool_result (with 2 results) + final_response
        assert_eq!(messages.len(), 4);
    }
}
