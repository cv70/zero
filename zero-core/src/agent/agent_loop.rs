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
use crate::provider::loop_provider::{StreamEvent, StreamingLoopProvider};
use crate::tool::{ToolDispatcher, ToolCall};
use crate::agent::loop_config::AgentLoopConfig;
use crate::hooks::HookManager;
use super::context_manager::ContextManager;
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

            // Context compaction if configured
            if config.max_context_tokens > 0 {
                let manager = ContextManager::new(config.max_context_tokens);
                manager.compact_if_needed(messages);
            }

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

            // Collect tool calls first
            let tool_calls: Vec<_> = response
                .content
                .iter()
                .filter_map(|block| match block {
                    ContentBlock::ToolUse { id, name, input } => {
                        Some((id.clone(), name.clone(), input.clone()))
                    }
                    _ => None,
                })
                .collect();

            if !tool_calls.is_empty() {
                let semaphore = Arc::new(tokio::sync::Semaphore::new(config.max_concurrent_tools));
                let mut handles = Vec::new();

                for (id, name, input) in tool_calls {
                    let dispatcher = self.tool_dispatcher.clone();
                    let timeout_dur = config.tool_timeout_duration();
                    let sem = semaphore.clone();
                    let verbose = config.verbose_logging;

                    handles.push(tokio::spawn(async move {
                        let _permit = sem.acquire().await.unwrap();
                        if verbose {
                            eprintln!("[AgentLoop] Executing tool: {}", name);
                        }
                        let arguments = serde_json::to_string(&input).unwrap_or_default();
                        let result = tokio::time::timeout(
                            timeout_dur,
                            dispatcher.execute(ToolCall {
                                id: id.clone(),
                                name: name.clone(),
                                arguments,
                            }),
                        )
                        .await
                        .map_err(|_| "Tool timeout".to_string())
                        .and_then(|r| r.map_err(|e| format!("Tool error: {}", e)));

                        let result_str = result.unwrap_or_else(|e| e);

                        if verbose {
                            let preview = if result_str.len() > 100 {
                                format!("{}...", &result_str[..100])
                            } else {
                                result_str.clone()
                            };
                            eprintln!("[AgentLoop] Tool result: {}", preview);
                        }

                        (id, result_str)
                    }));
                }

                let mut tool_results = Vec::new();
                for handle in handles {
                    match handle.await {
                        Ok((id, result)) => tool_results.push((id, result)),
                        Err(e) => {
                            tool_results.push(("error".to_string(), format!("Task failed: {}", e)))
                        }
                    }
                }

                messages.push(Message::tool_results(tool_results));
            }
        }
    }
}

/// Agent loop that streams responses to a callback
///
/// Unlike `DefaultAgentLoop`, this loop uses a `StreamingLoopProvider` to
/// receive incremental events (text deltas, tool use starts, etc.) and
/// forwards them to a user-supplied callback as they arrive.
pub struct StreamingAgentLoop {
    provider: Arc<dyn StreamingLoopProvider>,
    tool_dispatcher: Arc<dyn ToolDispatcher>,
}

impl StreamingAgentLoop {
    /// Create a new StreamingAgentLoop
    pub fn new(
        provider: Arc<dyn StreamingLoopProvider>,
        tool_dispatcher: Arc<dyn ToolDispatcher>,
    ) -> Self {
        Self {
            provider,
            tool_dispatcher,
        }
    }

    /// Execute the loop, calling `on_event` for each streaming event.
    ///
    /// The loop:
    /// 1. Calls `provider.complete_stream(messages)` to get a stream
    /// 2. Consumes the stream, calling `on_event` for each `StreamEvent`
    /// 3. Assembles full `ContentBlock`s from the deltas
    /// 4. When `MessageStop` arrives, checks the `stop_reason`
    /// 5. If `"tool_use"`, executes tools concurrently, appends results, and loops
    /// 6. If `"end_turn"` (or other), returns the accumulated text
    pub async fn execute_streaming<F>(
        &self,
        messages: &mut Vec<Message>,
        config: &AgentLoopConfig,
        mut on_event: F,
    ) -> Result<String, AgentError>
    where
        F: FnMut(StreamEvent) + Send,
    {
        use tokio_stream::StreamExt as _;

        let mut iteration = 0;
        let mut final_text = String::new();

        loop {
            if iteration >= config.max_iterations {
                return Err(AgentError::MaxIterationsExceeded(iteration));
            }
            iteration += 1;

            if config.verbose_logging {
                eprintln!(
                    "[StreamingAgentLoop] Iteration {} of {}",
                    iteration, config.max_iterations
                );
            }

            // Get the stream from the provider (with timeout on the initial connection)
            let mut stream = tokio::time::timeout(
                config.provider_timeout_duration(),
                self.provider.complete_stream(messages),
            )
            .await
            .map_err(|_| AgentError::ProviderTimeout)?
            .map_err(|e| AgentError::ProviderError(format!("{}", e)))?;

            // State for assembling content blocks from deltas
            let mut content_blocks: Vec<ContentBlock> = Vec::new();
            let mut current_text = String::new();
            let mut current_tool_id = String::new();
            let mut current_tool_name = String::new();
            let mut current_tool_input_json = String::new();
            let mut in_text_block = false;
            let mut in_tool_block = false;
            let mut stop_reason = String::new();

            // Consume the stream
            while let Some(event_result) = stream.next().await {
                let event = event_result
                    .map_err(|e| AgentError::ProviderError(format!("{}", e)))?;

                // Forward event to callback
                on_event(event.clone());

                match event {
                    StreamEvent::TextDelta(text) => {
                        if !in_text_block {
                            in_text_block = true;
                        }
                        current_text.push_str(&text);
                    }
                    StreamEvent::ToolUseStart { id, name } => {
                        in_tool_block = true;
                        current_tool_id = id;
                        current_tool_name = name;
                        current_tool_input_json.clear();
                    }
                    StreamEvent::ToolUseInputDelta(json_chunk) => {
                        current_tool_input_json.push_str(&json_chunk);
                    }
                    StreamEvent::ContentBlockStop => {
                        if in_text_block {
                            content_blocks.push(ContentBlock::Text {
                                text: current_text.clone(),
                            });
                            final_text = current_text.clone();
                            current_text.clear();
                            in_text_block = false;
                        }
                        if in_tool_block {
                            let input: serde_json::Value =
                                serde_json::from_str(&current_tool_input_json)
                                    .unwrap_or(serde_json::Value::Null);
                            content_blocks.push(ContentBlock::ToolUse {
                                id: current_tool_id.clone(),
                                name: current_tool_name.clone(),
                                input,
                            });
                            current_tool_id.clear();
                            current_tool_name.clear();
                            current_tool_input_json.clear();
                            in_tool_block = false;
                        }
                    }
                    StreamEvent::MessageStop {
                        stop_reason: reason,
                    } => {
                        stop_reason = reason;
                    }
                }
            }

            // Append the assembled assistant message to history
            if !content_blocks.is_empty() {
                messages.push(Message::assistant(content_blocks.clone()));
            }

            if config.verbose_logging {
                eprintln!(
                    "[StreamingAgentLoop] Stream done, stop_reason: {}",
                    stop_reason
                );
            }

            // If no tool_use, we're done
            if stop_reason != "tool_use" {
                return Ok(final_text);
            }

            // Execute tools concurrently (same pattern as DefaultAgentLoop)
            let tool_calls: Vec<_> = content_blocks
                .iter()
                .filter_map(|block| match block {
                    ContentBlock::ToolUse { id, name, input } => {
                        Some((id.clone(), name.clone(), input.clone()))
                    }
                    _ => None,
                })
                .collect();

            if !tool_calls.is_empty() {
                let semaphore =
                    Arc::new(tokio::sync::Semaphore::new(config.max_concurrent_tools));
                let mut handles = Vec::new();

                for (id, name, input) in tool_calls {
                    let dispatcher = self.tool_dispatcher.clone();
                    let timeout_dur = config.tool_timeout_duration();
                    let sem = semaphore.clone();
                    let verbose = config.verbose_logging;

                    handles.push(tokio::spawn(async move {
                        let _permit = sem.acquire().await.unwrap();
                        if verbose {
                            eprintln!("[StreamingAgentLoop] Executing tool: {}", name);
                        }
                        let arguments = serde_json::to_string(&input).unwrap_or_default();
                        let result = tokio::time::timeout(
                            timeout_dur,
                            dispatcher.execute(ToolCall {
                                id: id.clone(),
                                name: name.clone(),
                                arguments,
                            }),
                        )
                        .await
                        .map_err(|_| "Tool timeout".to_string())
                        .and_then(|r| r.map_err(|e| format!("Tool error: {}", e)));

                        (id, result.unwrap_or_else(|e| e))
                    }));
                }

                let mut tool_results = Vec::new();
                for handle in handles {
                    match handle.await {
                        Ok((id, result)) => tool_results.push((id, result)),
                        Err(e) => {
                            tool_results
                                .push(("error".to_string(), format!("Task failed: {}", e)));
                        }
                    }
                }

                messages.push(Message::tool_results(tool_results));
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

    // ── Concurrent tool execution tests ───────────────────────────────────

    /// A dispatcher that records execution order with a slight delay
    /// to test concurrency behavior.
    struct OrderTrackingDispatcher {
        execution_order: Arc<tokio::sync::Mutex<Vec<String>>>,
    }

    impl OrderTrackingDispatcher {
        fn new() -> Self {
            Self {
                execution_order: Arc::new(tokio::sync::Mutex::new(Vec::new())),
            }
        }

        fn order(&self) -> Arc<tokio::sync::Mutex<Vec<String>>> {
            self.execution_order.clone()
        }
    }

    #[async_trait]
    impl ToolDispatcher for OrderTrackingDispatcher {
        async fn execute(
            &self,
            call: ToolCall,
        ) -> Result<String, crate::error::ToolError> {
            // Record execution
            self.execution_order.lock().await.push(call.name.clone());
            Ok(format!("Result of {}", call.name))
        }
    }

    #[tokio::test]
    async fn test_concurrent_execution_all_tools_execute() {
        let tool_response = ProviderResponse {
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
                ContentBlock::tool_use(
                    "3".to_string(),
                    "tool_c".to_string(),
                    serde_json::json!({}),
                ),
            ],
            stop_reason: "tool_use".to_string(),
        };

        let final_response = ProviderResponse {
            content: vec![ContentBlock::text("Done")],
            stop_reason: "end_turn".to_string(),
        };

        let provider = Arc::new(MockProvider::new(vec![tool_response, final_response]));
        let dispatcher = Arc::new(OrderTrackingDispatcher::new());
        let order = dispatcher.order();
        let loop_impl = DefaultAgentLoop::new(provider, dispatcher);

        let mut messages = vec![Message::user("Run three tools")];
        let config = AgentLoopConfig::default().with_max_concurrent_tools(3);

        let result = loop_impl.execute(&mut messages, &config).await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "Done");

        // All three tools should have been executed
        let executed = order.lock().await;
        assert_eq!(executed.len(), 3);
        assert!(executed.contains(&"tool_a".to_string()));
        assert!(executed.contains(&"tool_b".to_string()));
        assert!(executed.contains(&"tool_c".to_string()));
    }

    #[tokio::test]
    async fn test_concurrent_execution_results_match_ids() {
        let tool_response = ProviderResponse {
            content: vec![
                ContentBlock::tool_use(
                    "id_x".to_string(),
                    "tool_x".to_string(),
                    serde_json::json!({}),
                ),
                ContentBlock::tool_use(
                    "id_y".to_string(),
                    "tool_y".to_string(),
                    serde_json::json!({}),
                ),
            ],
            stop_reason: "tool_use".to_string(),
        };

        let final_response = ProviderResponse {
            content: vec![ContentBlock::text("Complete")],
            stop_reason: "end_turn".to_string(),
        };

        let provider = Arc::new(MockProvider::new(vec![tool_response, final_response]));
        let dispatcher = Arc::new(MockDispatcher);
        let loop_impl = DefaultAgentLoop::new(provider, dispatcher);

        let mut messages = vec![Message::user("Test")];
        let config = AgentLoopConfig::default();

        let result = loop_impl.execute(&mut messages, &config).await;
        assert!(result.is_ok());

        // Check the tool_result message has both results
        let tool_result_msg = &messages[2]; // user + assistant + tool_result
        assert!(tool_result_msg.is_tool_result());

        if let Message::ToolResult { content } = tool_result_msg {
            assert_eq!(content.len(), 2);
            // Results are returned in order of handle completion (which preserves spawn order)
            let ids: Vec<&str> = content
                .iter()
                .map(|tr| match tr {
                    crate::message::ToolResultContent::ToolResult {
                        tool_use_id, ..
                    } => tool_use_id.as_str(),
                })
                .collect();
            assert!(ids.contains(&"id_x"));
            assert!(ids.contains(&"id_y"));
        }
    }

    #[tokio::test]
    async fn test_concurrent_execution_with_semaphore_limit() {
        // Use a semaphore limit of 1 to test that it still works (sequential behavior)
        let tool_response = ProviderResponse {
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
            content: vec![ContentBlock::text("Done")],
            stop_reason: "end_turn".to_string(),
        };

        let provider = Arc::new(MockProvider::new(vec![tool_response, final_response]));
        let dispatcher = Arc::new(OrderTrackingDispatcher::new());
        let order = dispatcher.order();
        let loop_impl = DefaultAgentLoop::new(provider, dispatcher);

        let mut messages = vec![Message::user("Test")];;
        let config = AgentLoopConfig::default().with_max_concurrent_tools(1);

        let result = loop_impl.execute(&mut messages, &config).await;
        assert!(result.is_ok());

        // Both tools should still execute
        let executed = order.lock().await;
        assert_eq!(executed.len(), 2);
    }

    // ── StreamingAgentLoop tests ──────────────────────────────────────────

    /// A mock streaming provider that sends predefined StreamEvents
    struct MockStreamProvider {
        call_count: Arc<AtomicUsize>,
        /// Each inner Vec is one "response" (a sequence of StreamEvents for one call)
        event_sequences: Vec<Vec<StreamEvent>>,
    }

    impl MockStreamProvider {
        fn new(event_sequences: Vec<Vec<StreamEvent>>) -> Self {
            Self {
                call_count: Arc::new(AtomicUsize::new(0)),
                event_sequences,
            }
        }
    }

    #[async_trait]
    impl LoopProvider for MockStreamProvider {
        fn name(&self) -> &str {
            "mock_stream"
        }

        async fn complete(
            &self,
            _messages: &[Message],
        ) -> Result<ProviderResponse, crate::error::ProviderError> {
            // Not used by StreamingAgentLoop, but required by trait
            Err(crate::error::ProviderError::ApiError(
                "Use complete_stream instead".to_string(),
            ))
        }
    }

    #[async_trait]
    impl StreamingLoopProvider for MockStreamProvider {
        async fn complete_stream(
            &self,
            _messages: &[Message],
        ) -> Result<
            std::pin::Pin<
                Box<dyn futures_core::Stream<Item = Result<StreamEvent, crate::error::ProviderError>> + Send>,
            >,
            crate::error::ProviderError,
        > {
            let count = self.call_count.fetch_add(1, Ordering::Relaxed);
            if count < self.event_sequences.len() {
                let events = self.event_sequences[count].clone();
                let (tx, rx) =
                    tokio::sync::mpsc::channel::<Result<StreamEvent, crate::error::ProviderError>>(64);

                tokio::spawn(async move {
                    for event in events {
                        if tx.send(Ok(event)).await.is_err() {
                            return;
                        }
                    }
                });

                Ok(Box::pin(tokio_stream::wrappers::ReceiverStream::new(rx)))
            } else {
                Err(crate::error::ProviderError::ApiError(
                    "No more event sequences".to_string(),
                ))
            }
        }
    }

    #[tokio::test]
    async fn test_streaming_loop_simple_text() {
        let events = vec![
            StreamEvent::TextDelta("Hello".to_string()),
            StreamEvent::TextDelta(", world!".to_string()),
            StreamEvent::ContentBlockStop,
            StreamEvent::MessageStop {
                stop_reason: "end_turn".to_string(),
            },
        ];

        let provider = Arc::new(MockStreamProvider::new(vec![events]));
        let dispatcher = Arc::new(MockDispatcher);
        let loop_impl = StreamingAgentLoop::new(provider, dispatcher);

        let mut messages = vec![Message::user("Say hello")];
        let config = AgentLoopConfig::default();

        let mut received_events = Vec::new();
        let result = loop_impl
            .execute_streaming(&mut messages, &config, |event| {
                received_events.push(event);
            })
            .await;

        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "Hello, world!");

        // 4 events: 2 text deltas, 1 content block stop, 1 message stop
        assert_eq!(received_events.len(), 4);

        // messages should have: user + assistant
        assert_eq!(messages.len(), 2);
    }

    #[tokio::test]
    async fn test_streaming_loop_with_tool_use() {
        // First response: tool use
        let events1 = vec![
            StreamEvent::TextDelta("Let me check.".to_string()),
            StreamEvent::ContentBlockStop,
            StreamEvent::ToolUseStart {
                id: "toolu_1".to_string(),
                name: "test_tool".to_string(),
            },
            StreamEvent::ToolUseInputDelta(r#"{"param":"value"}"#.to_string()),
            StreamEvent::ContentBlockStop,
            StreamEvent::MessageStop {
                stop_reason: "tool_use".to_string(),
            },
        ];

        // Second response: final text
        let events2 = vec![
            StreamEvent::TextDelta("Task completed".to_string()),
            StreamEvent::ContentBlockStop,
            StreamEvent::MessageStop {
                stop_reason: "end_turn".to_string(),
            },
        ];

        let provider = Arc::new(MockStreamProvider::new(vec![events1, events2]));
        let dispatcher = Arc::new(MockDispatcher);
        let loop_impl = StreamingAgentLoop::new(provider, dispatcher);

        let mut messages = vec![Message::user("Do something")];
        let config = AgentLoopConfig::default();

        let mut received_events = Vec::new();
        let result = loop_impl
            .execute_streaming(&mut messages, &config, |event| {
                received_events.push(event);
            })
            .await;

        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "Task completed");

        // messages: user + assistant(tool_use) + tool_result + assistant(text)
        assert_eq!(messages.len(), 4);

        // Check we got events from both rounds
        assert!(received_events.len() >= 6 + 3); // 6 from first round, 3 from second
    }

    #[tokio::test]
    async fn test_streaming_loop_max_iterations() {
        // Always returns tool_use, should hit max iterations
        let events = vec![
            StreamEvent::ToolUseStart {
                id: "toolu_1".to_string(),
                name: "test_tool".to_string(),
            },
            StreamEvent::ToolUseInputDelta("{}".to_string()),
            StreamEvent::ContentBlockStop,
            StreamEvent::MessageStop {
                stop_reason: "tool_use".to_string(),
            },
        ];

        let provider = Arc::new(MockStreamProvider::new(vec![
            events.clone(),
            events.clone(),
            events.clone(),
            events.clone(),
            events.clone(),
        ]));
        let dispatcher = Arc::new(MockDispatcher);
        let loop_impl = StreamingAgentLoop::new(provider, dispatcher);

        let mut messages = vec![Message::user("Loop forever")];
        let config = AgentLoopConfig::default().with_max_iterations(3);

        let result = loop_impl
            .execute_streaming(&mut messages, &config, |_| {})
            .await;

        assert!(matches!(result, Err(AgentError::MaxIterationsExceeded(_))));
    }
}
