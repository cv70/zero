/// Anthropic LLM provider implementations
///
/// This module provides two implementations:
/// - `AnthropicProvider`: Legacy `LLMProvider` implementation (simple string-in/string-out)
/// - `AnthropicLoopProvider`: Full `LoopProvider` implementation with tool calling support
use crate::error::ProviderError;
use crate::message::{ContentBlock, Message, ToolResultContent};
use crate::provider::LLMProvider;
use crate::provider::loop_provider::{
    LoopProvider, ProviderResponse, StreamEvent, StreamingLoopProvider,
};
use async_trait::async_trait;
use futures_core::Stream;
use serde::Deserialize;
use std::pin::Pin;

// ──────────────────────────────────────────────────────────────────────────────
// Legacy LLMProvider (kept for backward compat)
// ──────────────────────────────────────────────────────────────────────────────

/// Legacy Anthropic provider implementing the simple `LLMProvider` trait.
pub struct AnthropicProvider {
    api_key: String,
}

impl AnthropicProvider {
    pub fn new(api_key: String) -> Self {
        Self { api_key }
    }
}

#[async_trait]
impl LLMProvider for AnthropicProvider {
    fn name(&self) -> &str {
        "anthropic"
    }

    fn capabilities(&self) -> crate::provider::ModelCapability {
        crate::provider::ModelCapability::Multimodal
    }

    fn available_models(&self) -> Vec<String> {
        vec!["claude-3-opus".to_string()]
    }

    async fn complete(
        &self,
        prompt: &str,
        _opts: crate::provider::CompleteOpts,
    ) -> Result<String, ProviderError> {
        // Placeholder
        Ok(format!("Response to: {}", prompt))
    }
}

// ──────────────────────────────────────────────────────────────────────────────
// Real LoopProvider for Agent Loop
// ──────────────────────────────────────────────────────────────────────────────

const DEFAULT_MODEL: &str = "claude-sonnet-4-20250514";
const DEFAULT_MAX_TOKENS: u32 = 8192;
const ANTHROPIC_API_URL: &str = "https://api.anthropic.com/v1/messages";
const ANTHROPIC_VERSION: &str = "2023-06-01";

/// Anthropic provider implementing `LoopProvider` for the Agent loop.
///
/// This provider calls the Anthropic Messages API via HTTP, handles tool-use
/// round-trips, and converts between the internal `Message` format and the
/// Anthropic wire format.
pub struct AnthropicLoopProvider {
    client: reqwest::Client,
    api_key: String,
    model: String,
    system_prompt: Option<String>,
    max_tokens: u32,
    tools: Vec<serde_json::Value>,
    /// Whether to enable Anthropic prompt caching (default: true)
    enable_caching: bool,
}

impl AnthropicLoopProvider {
    /// Create a new provider with sensible defaults.
    pub fn new(api_key: String) -> Self {
        Self {
            client: reqwest::Client::new(),
            api_key,
            model: DEFAULT_MODEL.to_string(),
            system_prompt: None,
            max_tokens: DEFAULT_MAX_TOKENS,
            tools: Vec::new(),
            enable_caching: true,
        }
    }

    /// Set the system prompt.
    pub fn with_system_prompt(mut self, prompt: impl Into<String>) -> Self {
        self.system_prompt = Some(prompt.into());
        self
    }

    /// Set the tool definitions to include with every request.
    pub fn with_tools(mut self, tools: Vec<serde_json::Value>) -> Self {
        self.tools = tools;
        self
    }

    /// Set the model name.
    pub fn with_model(mut self, model: impl Into<String>) -> Self {
        self.model = model.into();
        self
    }

    /// Set the max tokens for completions.
    pub fn with_max_tokens(mut self, max_tokens: u32) -> Self {
        self.max_tokens = max_tokens;
        self
    }

    /// Override the HTTP client (useful for testing).
    pub fn with_client(mut self, client: reqwest::Client) -> Self {
        self.client = client;
        self
    }

    /// Enable or disable Anthropic prompt caching.
    pub fn with_caching(mut self, enable: bool) -> Self {
        self.enable_caching = enable;
        self
    }

    // ── Internal helpers ─────────────────────────────────────────────────

    /// Convert internal `Message` slice to the Anthropic messages JSON array.
    fn build_messages(messages: &[Message]) -> Vec<serde_json::Value> {
        messages
            .iter()
            .map(|msg| match msg {
                Message::User { content } => {
                    serde_json::json!({
                        "role": "user",
                        "content": content,
                    })
                }
                Message::Assistant { content } => {
                    let blocks: Vec<serde_json::Value> = content
                        .iter()
                        .map(|block| match block {
                            ContentBlock::Text { text } => {
                                serde_json::json!({
                                    "type": "text",
                                    "text": text,
                                })
                            }
                            ContentBlock::ToolUse { id, name, input } => {
                                serde_json::json!({
                                    "type": "tool_use",
                                    "id": id,
                                    "name": name,
                                    "input": input,
                                })
                            }
                        })
                        .collect();
                    serde_json::json!({
                        "role": "assistant",
                        "content": blocks,
                    })
                }
                Message::ToolResult { content } => {
                    // Tool results are sent as a "user" role message with
                    // content blocks of type "tool_result".
                    let blocks: Vec<serde_json::Value> = content
                        .iter()
                        .map(|tr| match tr {
                            ToolResultContent::ToolResult {
                                tool_use_id,
                                content,
                            } => {
                                serde_json::json!({
                                    "type": "tool_result",
                                    "tool_use_id": tool_use_id,
                                    "content": content,
                                })
                            }
                        })
                        .collect();
                    serde_json::json!({
                        "role": "user",
                        "content": blocks,
                    })
                }
            })
            .collect()
    }

    /// Build the full request body JSON.
    fn build_request_body(&self, messages: &[Message]) -> serde_json::Value {
        self.build_request_body_inner(messages, false)
    }

    /// Build the request body JSON with optional streaming flag.
    fn build_request_body_inner(&self, messages: &[Message], stream: bool) -> serde_json::Value {
        let mut body = serde_json::json!({
            "model": self.model,
            "max_tokens": self.max_tokens,
            "messages": Self::build_messages(messages),
        });

        if let Some(ref system) = self.system_prompt {
            if self.enable_caching {
                // Wrap system prompt in array with cache_control for prompt caching
                body["system"] = serde_json::json!([{
                    "type": "text",
                    "text": system,
                    "cache_control": {"type": "ephemeral"}
                }]);
            } else {
                body["system"] = serde_json::json!(system);
            }
        }

        if !self.tools.is_empty() {
            if self.enable_caching {
                // Add cache_control to the last tool definition
                let mut tools = self.tools.clone();
                if let Some(last_tool) = tools.last_mut() {
                    if let serde_json::Value::Object(map) = last_tool {
                        map.insert(
                            "cache_control".to_string(),
                            serde_json::json!({"type": "ephemeral"}),
                        );
                    }
                }
                body["tools"] = serde_json::json!(tools);
            } else {
                body["tools"] = serde_json::json!(self.tools);
            }
        }

        if stream {
            body["stream"] = serde_json::json!(true);
        }

        body
    }

    /// Parse the API response body into a `ProviderResponse`.
    fn parse_response(body: &ApiResponse) -> Result<ProviderResponse, ProviderError> {
        let content: Vec<ContentBlock> = body
            .content
            .iter()
            .map(|block| match block.r#type.as_str() {
                "text" => Ok(ContentBlock::Text {
                    text: block.text.clone().unwrap_or_default(),
                }),
                "tool_use" => Ok(ContentBlock::ToolUse {
                    id: block.id.clone().unwrap_or_default(),
                    name: block.name.clone().unwrap_or_default(),
                    input: block.input.clone().unwrap_or(serde_json::Value::Null),
                }),
                other => Err(ProviderError::InvalidResponse(format!(
                    "Unknown content block type: {}",
                    other
                ))),
            })
            .collect::<Result<Vec<_>, _>>()?;

        Ok(ProviderResponse::new(content, &body.stop_reason))
    }
}

#[async_trait]
impl LoopProvider for AnthropicLoopProvider {
    fn name(&self) -> &str {
        "anthropic"
    }

    async fn complete(&self, messages: &[Message]) -> Result<ProviderResponse, ProviderError> {
        let request_body = self.build_request_body(messages);

        let mut request = self
            .client
            .post(ANTHROPIC_API_URL)
            .header("x-api-key", &self.api_key)
            .header("anthropic-version", ANTHROPIC_VERSION)
            .header("content-type", "application/json");

        if self.enable_caching {
            request = request.header("anthropic-beta", "prompt-caching-2024-07-31");
        }

        let response = request
            .json(&request_body)
            .send()
            .await
            .map_err(|e| ProviderError::RequestFailed(e.to_string()))?;

        let status = response.status();

        if status == reqwest::StatusCode::TOO_MANY_REQUESTS {
            let body_text = response
                .text()
                .await
                .unwrap_or_else(|_| "rate limited".to_string());
            return Err(ProviderError::RateLimited(body_text));
        }

        if !status.is_success() {
            let body_text = response
                .text()
                .await
                .unwrap_or_else(|_| "unknown error".to_string());
            return Err(ProviderError::ApiError(format!(
                "HTTP {}: {}",
                status, body_text
            )));
        }

        let api_response: ApiResponse = response
            .json()
            .await
            .map_err(|e| ProviderError::InvalidResponse(e.to_string()))?;

        Self::parse_response(&api_response)
    }
}

// ──────────────────────────────────────────────────────────────────────────────
// Streaming implementation
// ──────────────────────────────────────────────────────────────────────────────

/// Parse an SSE data line into a `StreamEvent`.
///
/// Returns `None` for events we don't care about (e.g. `ping`, `message_start`).
fn parse_sse_data(event_type: &str, data: &str) -> Option<Result<StreamEvent, ProviderError>> {
    match event_type {
        "content_block_start" => {
            let v: serde_json::Value = serde_json::from_str(data).ok()?;
            let block = v.get("content_block")?;
            let block_type = block.get("type")?.as_str()?;
            match block_type {
                "text" => None, // text block start doesn't carry useful data
                "tool_use" => {
                    let id = block.get("id")?.as_str()?.to_string();
                    let name = block.get("name")?.as_str()?.to_string();
                    Some(Ok(StreamEvent::ToolUseStart { id, name }))
                }
                _ => None,
            }
        }
        "content_block_delta" => {
            let v: serde_json::Value = serde_json::from_str(data).ok()?;
            let delta = v.get("delta")?;
            let delta_type = delta.get("type")?.as_str()?;
            match delta_type {
                "text_delta" => {
                    let text = delta.get("text")?.as_str()?.to_string();
                    Some(Ok(StreamEvent::TextDelta(text)))
                }
                "input_json_delta" => {
                    let json = delta.get("partial_json")?.as_str()?.to_string();
                    Some(Ok(StreamEvent::ToolUseInputDelta(json)))
                }
                _ => None,
            }
        }
        "content_block_stop" => Some(Ok(StreamEvent::ContentBlockStop)),
        "message_delta" => {
            let v: serde_json::Value = serde_json::from_str(data).ok()?;
            let delta = v.get("delta")?;
            let stop_reason = delta.get("stop_reason")?.as_str()?.to_string();
            Some(Ok(StreamEvent::MessageStop { stop_reason }))
        }
        "message_stop" => {
            // message_stop is a terminal event; the stop_reason was already
            // delivered via message_delta, so we can ignore this.
            None
        }
        _ => None, // ping, message_start, etc.
    }
}

#[async_trait]
impl StreamingLoopProvider for AnthropicLoopProvider {
    async fn complete_stream(
        &self,
        messages: &[Message],
    ) -> Result<Pin<Box<dyn Stream<Item = Result<StreamEvent, ProviderError>> + Send>>, ProviderError>
    {
        let request_body = self.build_request_body_inner(messages, true);

        let mut request = self
            .client
            .post(ANTHROPIC_API_URL)
            .header("x-api-key", &self.api_key)
            .header("anthropic-version", ANTHROPIC_VERSION)
            .header("content-type", "application/json");

        if self.enable_caching {
            request = request.header("anthropic-beta", "prompt-caching-2024-07-31");
        }

        let response = request
            .json(&request_body)
            .send()
            .await
            .map_err(|e| ProviderError::RequestFailed(e.to_string()))?;

        let status = response.status();

        if status == reqwest::StatusCode::TOO_MANY_REQUESTS {
            let body_text = response
                .text()
                .await
                .unwrap_or_else(|_| "rate limited".to_string());
            return Err(ProviderError::RateLimited(body_text));
        }

        if !status.is_success() {
            let body_text = response
                .text()
                .await
                .unwrap_or_else(|_| "unknown error".to_string());
            return Err(ProviderError::ApiError(format!(
                "HTTP {}: {}",
                status, body_text
            )));
        }

        // Bridge the response body into an mpsc channel.
        // We read the full response body as text, then parse SSE events from it.
        // This avoids lifetime issues with the streaming byte stream and pin.
        let (tx, rx) = tokio::sync::mpsc::channel::<Result<StreamEvent, ProviderError>>(64);

        // Read the full response body as bytes and parse SSE from it.
        // For truly incremental streaming, we spawn a task that uses
        // chunk_stream (via reqwest's chunk() method).
        tokio::spawn(async move {
            let mut response = response;
            let mut buffer = String::new();
            let mut current_event_type = String::new();

            // Read response body chunk by chunk
            loop {
                match response.chunk().await {
                    Ok(Some(bytes)) => {
                        buffer.push_str(&String::from_utf8_lossy(&bytes));

                        // Process complete SSE events (delimited by \n\n)
                        while let Some(pos) = buffer.find("\n\n") {
                            let event_block = buffer[..pos].to_string();
                            buffer = buffer[pos + 2..].to_string();

                            // Parse lines within the event block
                            let mut data_line = String::new();
                            for line in event_block.lines() {
                                if let Some(ev) = line.strip_prefix("event: ") {
                                    current_event_type = ev.trim().to_string();
                                } else if let Some(d) = line.strip_prefix("data: ") {
                                    data_line = d.to_string();
                                }
                            }

                            if !current_event_type.is_empty() && !data_line.is_empty() {
                                if let Some(event) = parse_sse_data(&current_event_type, &data_line)
                                {
                                    if tx.send(event).await.is_err() {
                                        return; // receiver dropped
                                    }
                                }
                            } else if !current_event_type.is_empty() {
                                // Events like content_block_stop may have no data payload
                                if let Some(event) = parse_sse_data(&current_event_type, "{}") {
                                    if tx.send(event).await.is_err() {
                                        return;
                                    }
                                }
                            }

                            current_event_type.clear();
                        }
                    }
                    Ok(None) => return, // stream ended
                    Err(e) => {
                        let _ = tx
                            .send(Err(ProviderError::RequestFailed(e.to_string())))
                            .await;
                        return;
                    }
                }
            }
        });

        Ok(Box::pin(tokio_stream::wrappers::ReceiverStream::new(rx)))
    }
}

// ──────────────────────────────────────────────────────────────────────────────
// API response deserialization types (internal)
// ──────────────────────────────────────────────────────────────────────────────

/// Anthropic Messages API response (subset of fields we care about).
#[derive(Debug, Deserialize)]
struct ApiResponse {
    content: Vec<ApiContentBlock>,
    stop_reason: String,
}

/// A single content block in the API response.
#[derive(Debug, Deserialize)]
struct ApiContentBlock {
    r#type: String,
    /// Present for "text" blocks.
    text: Option<String>,
    /// Present for "tool_use" blocks.
    id: Option<String>,
    /// Present for "tool_use" blocks.
    name: Option<String>,
    /// Present for "tool_use" blocks.
    input: Option<serde_json::Value>,
}

/// Anthropic API error response body.
#[allow(dead_code)]
#[derive(Debug, Deserialize)]
struct ApiErrorResponse {
    error: ApiErrorDetail,
}

#[allow(dead_code)]
#[derive(Debug, Deserialize)]
struct ApiErrorDetail {
    r#type: String,
    message: String,
}

// ──────────────────────────────────────────────────────────────────────────────
// Tests
// ──────────────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    // ── Message conversion tests ─────────────────────────────────────────

    #[test]
    fn test_build_messages_user() {
        let messages = vec![Message::user("Hello")];
        let result = AnthropicLoopProvider::build_messages(&messages);
        assert_eq!(result.len(), 1);
        assert_eq!(result[0]["role"], "user");
        assert_eq!(result[0]["content"], "Hello");
    }

    #[test]
    fn test_build_messages_assistant_text() {
        let messages = vec![Message::assistant(vec![ContentBlock::text("Hi there")])];
        let result = AnthropicLoopProvider::build_messages(&messages);
        assert_eq!(result.len(), 1);
        assert_eq!(result[0]["role"], "assistant");
        let content = result[0]["content"].as_array().unwrap();
        assert_eq!(content.len(), 1);
        assert_eq!(content[0]["type"], "text");
        assert_eq!(content[0]["text"], "Hi there");
    }

    #[test]
    fn test_build_messages_assistant_tool_use() {
        let messages = vec![Message::assistant(vec![
            ContentBlock::text("Let me check"),
            ContentBlock::tool_use(
                "toolu_1".to_string(),
                "bash".to_string(),
                json!({"command": "ls"}),
            ),
        ])];
        let result = AnthropicLoopProvider::build_messages(&messages);
        let content = result[0]["content"].as_array().unwrap();
        assert_eq!(content.len(), 2);

        assert_eq!(content[0]["type"], "text");
        assert_eq!(content[0]["text"], "Let me check");

        assert_eq!(content[1]["type"], "tool_use");
        assert_eq!(content[1]["id"], "toolu_1");
        assert_eq!(content[1]["name"], "bash");
        assert_eq!(content[1]["input"]["command"], "ls");
    }

    #[test]
    fn test_build_messages_tool_result() {
        let messages = vec![Message::tool_result(
            "toolu_1".to_string(),
            "file1.txt\nfile2.txt",
        )];
        let result = AnthropicLoopProvider::build_messages(&messages);
        assert_eq!(result.len(), 1);
        assert_eq!(result[0]["role"], "user");
        let content = result[0]["content"].as_array().unwrap();
        assert_eq!(content.len(), 1);
        assert_eq!(content[0]["type"], "tool_result");
        assert_eq!(content[0]["tool_use_id"], "toolu_1");
        assert_eq!(content[0]["content"], "file1.txt\nfile2.txt");
    }

    #[test]
    fn test_build_messages_multi_tool_results() {
        let messages = vec![Message::tool_results(vec![
            ("toolu_1".to_string(), "result 1".to_string()),
            ("toolu_2".to_string(), "result 2".to_string()),
        ])];
        let result = AnthropicLoopProvider::build_messages(&messages);
        let content = result[0]["content"].as_array().unwrap();
        assert_eq!(content.len(), 2);
        assert_eq!(content[0]["tool_use_id"], "toolu_1");
        assert_eq!(content[1]["tool_use_id"], "toolu_2");
    }

    #[test]
    fn test_build_messages_full_conversation() {
        let messages = vec![
            Message::user("List files"),
            Message::assistant(vec![
                ContentBlock::text("I'll run ls for you."),
                ContentBlock::tool_use(
                    "toolu_1".to_string(),
                    "bash".to_string(),
                    json!({"command": "ls"}),
                ),
            ]),
            Message::tool_result("toolu_1".to_string(), "README.md\nsrc/"),
            Message::assistant(vec![ContentBlock::text(
                "Here are the files: README.md and src/",
            )]),
        ];

        let result = AnthropicLoopProvider::build_messages(&messages);
        assert_eq!(result.len(), 4);
        assert_eq!(result[0]["role"], "user");
        assert_eq!(result[1]["role"], "assistant");
        assert_eq!(result[2]["role"], "user"); // tool_result becomes user role
        assert_eq!(result[3]["role"], "assistant");
    }

    // ── Request body construction tests ──────────────────────────────────

    #[test]
    fn test_build_request_body_defaults() {
        let provider = AnthropicLoopProvider::new("test-key".to_string());
        let messages = vec![Message::user("Hello")];
        let body = provider.build_request_body(&messages);

        assert_eq!(body["model"], DEFAULT_MODEL);
        assert_eq!(body["max_tokens"], DEFAULT_MAX_TOKENS);
        assert!(body.get("system").is_none());
        assert!(body.get("tools").is_none());
    }

    #[test]
    fn test_build_request_body_with_system_prompt() {
        let provider = AnthropicLoopProvider::new("test-key".to_string())
            .with_system_prompt("You are a helpful assistant.")
            .with_caching(false);
        let messages = vec![Message::user("Hello")];
        let body = provider.build_request_body(&messages);

        assert_eq!(body["system"], "You are a helpful assistant.");
    }

    #[test]
    fn test_build_request_body_with_system_prompt_cached() {
        let provider = AnthropicLoopProvider::new("test-key".to_string())
            .with_system_prompt("You are a helpful assistant.");
        let messages = vec![Message::user("Hello")];
        let body = provider.build_request_body(&messages);

        // When caching is enabled, system should be an array with cache_control
        let system = body["system"].as_array().unwrap();
        assert_eq!(system.len(), 1);
        assert_eq!(system[0]["type"], "text");
        assert_eq!(system[0]["text"], "You are a helpful assistant.");
        assert_eq!(system[0]["cache_control"]["type"], "ephemeral");
    }

    #[test]
    fn test_build_request_body_with_tools() {
        let tools = vec![json!({
            "name": "bash",
            "description": "Execute shell commands",
            "input_schema": {
                "type": "object",
                "properties": {
                    "command": {"type": "string"}
                },
                "required": ["command"]
            }
        })];

        let provider = AnthropicLoopProvider::new("test-key".to_string())
            .with_tools(tools.clone())
            .with_caching(false);
        let messages = vec![Message::user("Hello")];
        let body = provider.build_request_body(&messages);

        assert_eq!(body["tools"], json!(tools));
    }

    #[test]
    fn test_build_request_body_with_tools_cached() {
        let tools = vec![json!({
            "name": "bash",
            "description": "Execute shell commands",
            "input_schema": {
                "type": "object",
                "properties": {
                    "command": {"type": "string"}
                },
                "required": ["command"]
            }
        })];

        let provider = AnthropicLoopProvider::new("test-key".to_string()).with_tools(tools.clone());
        let messages = vec![Message::user("Hello")];
        let body = provider.build_request_body(&messages);

        // When caching is enabled, the last tool should have cache_control
        let result_tools = body["tools"].as_array().unwrap();
        assert_eq!(result_tools.len(), 1);
        assert_eq!(result_tools[0]["name"], "bash");
        assert_eq!(result_tools[0]["cache_control"]["type"], "ephemeral");
    }

    #[test]
    fn test_build_request_body_custom_model() {
        let provider =
            AnthropicLoopProvider::new("test-key".to_string()).with_model("claude-opus-4-20250514");
        let messages = vec![Message::user("Hello")];
        let body = provider.build_request_body(&messages);

        assert_eq!(body["model"], "claude-opus-4-20250514");
    }

    #[test]
    fn test_build_request_body_custom_max_tokens() {
        let provider = AnthropicLoopProvider::new("test-key".to_string()).with_max_tokens(4096);
        let messages = vec![Message::user("Hello")];
        let body = provider.build_request_body(&messages);

        assert_eq!(body["max_tokens"], 4096);
    }

    // ── Response parsing tests ───────────────────────────────────────────

    #[test]
    fn test_parse_response_text_only() {
        let api_response = ApiResponse {
            content: vec![ApiContentBlock {
                r#type: "text".to_string(),
                text: Some("Hello! How can I help?".to_string()),
                id: None,
                name: None,
                input: None,
            }],
            stop_reason: "end_turn".to_string(),
        };

        let result = AnthropicLoopProvider::parse_response(&api_response).unwrap();
        assert_eq!(result.stop_reason, "end_turn");
        assert_eq!(result.content.len(), 1);
        assert!(!result.has_tool_use());
        assert_eq!(
            result.first_text(),
            Some("Hello! How can I help?".to_string())
        );
    }

    #[test]
    fn test_parse_response_tool_use() {
        let api_response = ApiResponse {
            content: vec![
                ApiContentBlock {
                    r#type: "text".to_string(),
                    text: Some("Let me check that.".to_string()),
                    id: None,
                    name: None,
                    input: None,
                },
                ApiContentBlock {
                    r#type: "tool_use".to_string(),
                    text: None,
                    id: Some("toolu_abc123".to_string()),
                    name: Some("bash".to_string()),
                    input: Some(json!({"command": "ls -la"})),
                },
            ],
            stop_reason: "tool_use".to_string(),
        };

        let result = AnthropicLoopProvider::parse_response(&api_response).unwrap();
        assert_eq!(result.stop_reason, "tool_use");
        assert_eq!(result.content.len(), 2);
        assert!(result.has_tool_use());

        let tool_uses = result.tool_uses();
        assert_eq!(tool_uses.len(), 1);
        assert_eq!(tool_uses[0].0, "toolu_abc123");
        assert_eq!(tool_uses[0].1, "bash");
        assert_eq!(tool_uses[0].2["command"], "ls -la");
    }

    #[test]
    fn test_parse_response_multiple_tool_uses() {
        let api_response = ApiResponse {
            content: vec![
                ApiContentBlock {
                    r#type: "tool_use".to_string(),
                    text: None,
                    id: Some("toolu_1".to_string()),
                    name: Some("bash".to_string()),
                    input: Some(json!({"command": "ls"})),
                },
                ApiContentBlock {
                    r#type: "tool_use".to_string(),
                    text: None,
                    id: Some("toolu_2".to_string()),
                    name: Some("read_file".to_string()),
                    input: Some(json!({"path": "README.md"})),
                },
            ],
            stop_reason: "tool_use".to_string(),
        };

        let result = AnthropicLoopProvider::parse_response(&api_response).unwrap();
        assert_eq!(result.tool_uses().len(), 2);
    }

    #[test]
    fn test_parse_response_unknown_block_type() {
        let api_response = ApiResponse {
            content: vec![ApiContentBlock {
                r#type: "unknown_type".to_string(),
                text: None,
                id: None,
                name: None,
                input: None,
            }],
            stop_reason: "end_turn".to_string(),
        };

        let result = AnthropicLoopProvider::parse_response(&api_response);
        assert!(result.is_err());
        match result {
            Err(ProviderError::InvalidResponse(msg)) => {
                assert!(msg.contains("Unknown content block type"));
            }
            _ => panic!("Expected InvalidResponse error"),
        }
    }

    // ── JSON deserialization tests (simulating raw API responses) ─────────

    #[test]
    fn test_deserialize_text_response() {
        let raw = json!({
            "content": [
                {"type": "text", "text": "Hello world"}
            ],
            "stop_reason": "end_turn"
        });
        let api_response: ApiResponse = serde_json::from_value(raw).unwrap();
        let result = AnthropicLoopProvider::parse_response(&api_response).unwrap();
        assert_eq!(result.first_text(), Some("Hello world".to_string()));
        assert_eq!(result.stop_reason, "end_turn");
    }

    #[test]
    fn test_deserialize_tool_use_response() {
        let raw = json!({
            "content": [
                {"type": "text", "text": "I'll list the files."},
                {
                    "type": "tool_use",
                    "id": "toolu_xyz",
                    "name": "bash",
                    "input": {"command": "ls -la"}
                }
            ],
            "stop_reason": "tool_use"
        });
        let api_response: ApiResponse = serde_json::from_value(raw).unwrap();
        let result = AnthropicLoopProvider::parse_response(&api_response).unwrap();

        assert_eq!(result.stop_reason, "tool_use");
        assert!(result.has_tool_use());
        let uses = result.tool_uses();
        assert_eq!(uses[0].1, "bash");
    }

    // ── Builder / constructor tests ──────────────────────────────────────

    #[test]
    fn test_new_defaults() {
        let provider = AnthropicLoopProvider::new("sk-test".to_string());
        assert_eq!(provider.model, DEFAULT_MODEL);
        assert_eq!(provider.max_tokens, DEFAULT_MAX_TOKENS);
        assert!(provider.system_prompt.is_none());
        assert!(provider.tools.is_empty());
        assert_eq!(provider.api_key, "sk-test");
    }

    #[test]
    fn test_builder_chain() {
        let provider = AnthropicLoopProvider::new("sk-test".to_string())
            .with_model("claude-opus-4-20250514")
            .with_system_prompt("Be concise.")
            .with_max_tokens(2048)
            .with_tools(vec![json!({"name": "bash"})]);

        assert_eq!(provider.model, "claude-opus-4-20250514");
        assert_eq!(provider.system_prompt, Some("Be concise.".to_string()));
        assert_eq!(provider.max_tokens, 2048);
        assert_eq!(provider.tools.len(), 1);
    }

    #[test]
    fn test_provider_name() {
        let provider = AnthropicLoopProvider::new("sk-test".to_string());
        assert_eq!(provider.name(), "anthropic");
    }

    // ── HTTP integration test with mock server ───────────────────────────

    #[test]
    fn test_parse_full_api_text_response() {
        // Simulate a full API response with extra fields (id, type, model, usage)
        // that our ApiResponse struct ignores gracefully.
        let mock_body = json!({
            "id": "msg_123",
            "type": "message",
            "role": "assistant",
            "content": [
                {"type": "text", "text": "Hello! I'm Claude."}
            ],
            "model": "claude-sonnet-4-20250514",
            "stop_reason": "end_turn",
            "stop_sequence": null,
            "usage": {"input_tokens": 10, "output_tokens": 20}
        });

        let api_response: ApiResponse = serde_json::from_value(mock_body).unwrap();
        let result = AnthropicLoopProvider::parse_response(&api_response).unwrap();

        assert_eq!(result.stop_reason, "end_turn");
        assert_eq!(result.first_text(), Some("Hello! I'm Claude.".to_string()));
        assert!(!result.has_tool_use());
    }

    #[test]
    fn test_parse_full_api_tool_use_response() {
        let mock_body = json!({
            "id": "msg_456",
            "type": "message",
            "role": "assistant",
            "content": [
                {"type": "text", "text": "Let me read that file for you."},
                {
                    "type": "tool_use",
                    "id": "toolu_abc",
                    "name": "read_file",
                    "input": {"path": "/tmp/test.txt"}
                }
            ],
            "model": "claude-sonnet-4-20250514",
            "stop_reason": "tool_use",
            "stop_sequence": null,
            "usage": {"input_tokens": 15, "output_tokens": 30}
        });

        let api_response: ApiResponse = serde_json::from_value(mock_body).unwrap();
        let result = AnthropicLoopProvider::parse_response(&api_response).unwrap();

        assert_eq!(result.stop_reason, "tool_use");
        assert!(result.has_tool_use());

        let text = result.first_text().unwrap();
        assert_eq!(text, "Let me read that file for you.");

        let tool_uses = result.tool_uses();
        assert_eq!(tool_uses.len(), 1);
        assert_eq!(tool_uses[0].0, "toolu_abc");
        assert_eq!(tool_uses[0].1, "read_file");
        assert_eq!(tool_uses[0].2["path"], "/tmp/test.txt");
    }

    // ── Round-trip: build request -> parse response ──────────────────────

    #[test]
    fn test_full_conversation_roundtrip() {
        let provider = AnthropicLoopProvider::new("sk-test".to_string())
            .with_system_prompt("You are a coding assistant.")
            .with_tools(vec![json!({
                "name": "bash",
                "description": "Run a shell command",
                "input_schema": {
                    "type": "object",
                    "properties": {"command": {"type": "string"}},
                    "required": ["command"]
                }
            })])
            .with_caching(false);

        // Simulate a multi-turn conversation
        let messages = vec![
            Message::user("What files are here?"),
            Message::assistant(vec![
                ContentBlock::text("I'll check with ls."),
                ContentBlock::tool_use(
                    "toolu_1".to_string(),
                    "bash".to_string(),
                    json!({"command": "ls"}),
                ),
            ]),
            Message::tool_result("toolu_1".to_string(), "README.md\nsrc/\nCargo.toml"),
        ];

        let body = provider.build_request_body(&messages);

        // Verify structure
        assert_eq!(body["model"], DEFAULT_MODEL);
        assert_eq!(body["system"], "You are a coding assistant.");
        assert_eq!(body["tools"].as_array().unwrap().len(), 1);

        let api_messages = body["messages"].as_array().unwrap();
        assert_eq!(api_messages.len(), 3);

        // First message: user
        assert_eq!(api_messages[0]["role"], "user");
        assert_eq!(api_messages[0]["content"], "What files are here?");

        // Second message: assistant with tool_use
        assert_eq!(api_messages[1]["role"], "assistant");
        let asst_content = api_messages[1]["content"].as_array().unwrap();
        assert_eq!(asst_content.len(), 2);
        assert_eq!(asst_content[1]["type"], "tool_use");
        assert_eq!(asst_content[1]["name"], "bash");

        // Third message: tool_result as user
        assert_eq!(api_messages[2]["role"], "user");
        let tr_content = api_messages[2]["content"].as_array().unwrap();
        assert_eq!(tr_content[0]["type"], "tool_result");
        assert_eq!(tr_content[0]["tool_use_id"], "toolu_1");
    }

    // ── Streaming request body tests ──────────────────────────────────────

    #[test]
    fn test_build_request_body_stream_flag() {
        let provider = AnthropicLoopProvider::new("test-key".to_string());
        let messages = vec![Message::user("Hello")];
        let body = provider.build_request_body_inner(&messages, true);
        assert_eq!(body["stream"], true);
    }

    #[test]
    fn test_build_request_body_no_stream_flag() {
        let provider = AnthropicLoopProvider::new("test-key".to_string());
        let messages = vec![Message::user("Hello")];
        let body = provider.build_request_body_inner(&messages, false);
        assert!(body.get("stream").is_none());
    }

    // ── SSE parsing tests ─────────────────────────────────────────────────

    #[test]
    fn test_parse_sse_text_delta() {
        let data = r#"{"type":"content_block_delta","index":0,"delta":{"type":"text_delta","text":"Hello"}}"#;
        let result = parse_sse_data("content_block_delta", data);
        assert!(result.is_some());
        match result.unwrap().unwrap() {
            StreamEvent::TextDelta(text) => assert_eq!(text, "Hello"),
            other => panic!("Expected TextDelta, got {:?}", other),
        }
    }

    #[test]
    fn test_parse_sse_tool_use_start() {
        let data = r#"{"type":"content_block_start","index":1,"content_block":{"type":"tool_use","id":"toolu_abc","name":"bash"}}"#;
        let result = parse_sse_data("content_block_start", data);
        assert!(result.is_some());
        match result.unwrap().unwrap() {
            StreamEvent::ToolUseStart { id, name } => {
                assert_eq!(id, "toolu_abc");
                assert_eq!(name, "bash");
            }
            other => panic!("Expected ToolUseStart, got {:?}", other),
        }
    }

    #[test]
    fn test_parse_sse_tool_input_delta() {
        let data = r#"{"type":"content_block_delta","index":1,"delta":{"type":"input_json_delta","partial_json":"{\"command\":"}}"#;
        let result = parse_sse_data("content_block_delta", data);
        assert!(result.is_some());
        match result.unwrap().unwrap() {
            StreamEvent::ToolUseInputDelta(json) => assert_eq!(json, "{\"command\":"),
            other => panic!("Expected ToolUseInputDelta, got {:?}", other),
        }
    }

    #[test]
    fn test_parse_sse_content_block_stop() {
        let result = parse_sse_data("content_block_stop", "{}");
        assert!(result.is_some());
        assert!(matches!(
            result.unwrap().unwrap(),
            StreamEvent::ContentBlockStop
        ));
    }

    #[test]
    fn test_parse_sse_message_delta_stop_reason() {
        let data = r#"{"type":"message_delta","delta":{"stop_reason":"end_turn"}}"#;
        let result = parse_sse_data("message_delta", data);
        assert!(result.is_some());
        match result.unwrap().unwrap() {
            StreamEvent::MessageStop { stop_reason } => assert_eq!(stop_reason, "end_turn"),
            other => panic!("Expected MessageStop, got {:?}", other),
        }
    }

    #[test]
    fn test_parse_sse_message_stop_ignored() {
        let result = parse_sse_data("message_stop", "{}");
        assert!(result.is_none());
    }

    #[test]
    fn test_parse_sse_ping_ignored() {
        let result = parse_sse_data("ping", "{}");
        assert!(result.is_none());
    }

    #[test]
    fn test_parse_sse_message_start_ignored() {
        let result = parse_sse_data("message_start", r#"{"type":"message_start"}"#);
        assert!(result.is_none());
    }

    #[test]
    fn test_parse_sse_text_block_start_ignored() {
        let data =
            r#"{"type":"content_block_start","index":0,"content_block":{"type":"text","text":""}}"#;
        let result = parse_sse_data("content_block_start", data);
        assert!(result.is_none());
    }

    #[test]
    fn test_parse_sse_tool_use_stop_reason() {
        let data = r#"{"type":"message_delta","delta":{"stop_reason":"tool_use"}}"#;
        let result = parse_sse_data("message_delta", data);
        assert!(result.is_some());
        match result.unwrap().unwrap() {
            StreamEvent::MessageStop { stop_reason } => assert_eq!(stop_reason, "tool_use"),
            other => panic!("Expected MessageStop, got {:?}", other),
        }
    }
}
