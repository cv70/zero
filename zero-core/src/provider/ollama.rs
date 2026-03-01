/// Ollama LLM provider implementations
///
/// This module provides two implementations:
/// - `OllamaProvider`: Legacy `LLMProvider` implementation (simple string-in/string-out)
/// - `OllamaLoopProvider`: Full `LoopProvider` implementation with tool calling support

use crate::error::ProviderError;
use crate::message::{ContentBlock, Message, ToolResultContent};
use crate::provider::loop_provider::{LoopProvider, ProviderResponse};
use crate::provider::LLMProvider;
use async_trait::async_trait;
use serde::Deserialize;

// ──────────────────────────────────────────────────────────────────────────────
// Legacy LLMProvider (kept for backward compat)
// ──────────────────────────────────────────────────────────────────────────────

/// Legacy Ollama provider implementing the simple `LLMProvider` trait.
pub struct OllamaProvider {
    endpoint: String,
}

impl OllamaProvider {
    pub fn new(endpoint: String) -> Self {
        Self { endpoint }
    }
}

#[async_trait]
impl LLMProvider for OllamaProvider {
    fn name(&self) -> &str {
        "ollama"
    }

    fn capabilities(&self) -> crate::provider::ModelCapability {
        crate::provider::ModelCapability::TextOnly
    }

    fn available_models(&self) -> Vec<String> {
        vec!["llama2".to_string()]
    }

    async fn complete(
        &self,
        prompt: &str,
        _opts: crate::provider::CompleteOpts,
    ) -> Result<String, ProviderError> {
        // Placeholder
        let _ = &self.endpoint;
        Ok(format!("Response to: {}", prompt))
    }
}

// ──────────────────────────────────────────────────────────────────────────────
// Real LoopProvider for Agent Loop
// ──────────────────────────────────────────────────────────────────────────────

const DEFAULT_MODEL: &str = "llama3.2";
const DEFAULT_ENDPOINT: &str = "http://localhost:11434";

/// Ollama provider implementing `LoopProvider` for the Agent loop.
///
/// This provider calls the Ollama API (which uses an OpenAI-compatible message
/// format), handles tool-use round-trips, and converts between the internal
/// `Message` format and the Ollama wire format.
///
/// Key differences from OpenAI:
/// - Response is flat (no `choices` array) -- the message sits at the top level
/// - Uses `done_reason` instead of `finish_reason`
/// - No API key required (local inference)
/// - Requires `"stream": false` in the request body
pub struct OllamaLoopProvider {
    client: reqwest::Client,
    endpoint: String,
    model: String,
    system_prompt: Option<String>,
    tools: Vec<serde_json::Value>,
}

impl OllamaLoopProvider {
    /// Create a new provider with sensible defaults.
    pub fn new() -> Self {
        Self {
            client: reqwest::Client::new(),
            endpoint: DEFAULT_ENDPOINT.to_string(),
            model: DEFAULT_MODEL.to_string(),
            system_prompt: None,
            tools: Vec::new(),
        }
    }

    /// Set the model name.
    pub fn with_model(mut self, model: impl Into<String>) -> Self {
        self.model = model.into();
        self
    }

    /// Set the endpoint URL.
    pub fn with_endpoint(mut self, endpoint: impl Into<String>) -> Self {
        self.endpoint = endpoint.into();
        self
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

    /// Override the HTTP client (useful for testing).
    pub fn with_client(mut self, client: reqwest::Client) -> Self {
        self.client = client;
        self
    }

    // ── Internal helpers ─────────────────────────────────────────────────

    /// Build the API URL for chat completions.
    fn api_url(&self) -> String {
        format!("{}/api/chat", self.endpoint)
    }

    /// Convert internal `Message` slice to the Ollama messages JSON array.
    ///
    /// Ollama uses the same message format as OpenAI:
    /// - System prompt is a separate message with role "system"
    /// - Assistant tool calls go in a "tool_calls" array
    /// - Tool results use role "tool" with "tool_call_id"
    fn build_messages(&self, messages: &[Message]) -> Vec<serde_json::Value> {
        let mut result = Vec::new();

        // System prompt goes as a separate message
        if let Some(ref system) = self.system_prompt {
            result.push(serde_json::json!({
                "role": "system",
                "content": system,
            }));
        }

        for msg in messages {
            match msg {
                Message::User { content } => {
                    result.push(serde_json::json!({
                        "role": "user",
                        "content": content,
                    }));
                }
                Message::Assistant { content } => {
                    let mut message = serde_json::json!({
                        "role": "assistant",
                    });

                    // Extract text content - combine all text blocks
                    let text_parts: Vec<&str> = content
                        .iter()
                        .filter_map(|block| match block {
                            ContentBlock::Text { text } => Some(text.as_str()),
                            _ => None,
                        })
                        .collect();

                    if !text_parts.is_empty() {
                        message["content"] =
                            serde_json::Value::String(text_parts.join(""));
                    } else {
                        message["content"] = serde_json::Value::String(String::new());
                    }

                    // Extract tool calls
                    let tool_calls: Vec<serde_json::Value> = content
                        .iter()
                        .filter_map(|block| match block {
                            ContentBlock::ToolUse { id, name, input } => {
                                Some(serde_json::json!({
                                    "id": id,
                                    "type": "function",
                                    "function": {
                                        "name": name,
                                        "arguments": input.to_string(),
                                    }
                                }))
                            }
                            _ => None,
                        })
                        .collect();

                    if !tool_calls.is_empty() {
                        message["tool_calls"] = serde_json::json!(tool_calls);
                    }

                    result.push(message);
                }
                Message::ToolResult { content } => {
                    // Each tool result becomes a separate message with role "tool"
                    for tr in content {
                        match tr {
                            ToolResultContent::ToolResult {
                                tool_use_id,
                                content,
                            } => {
                                result.push(serde_json::json!({
                                    "role": "tool",
                                    "tool_call_id": tool_use_id,
                                    "content": content,
                                }));
                            }
                        }
                    }
                }
            }
        }

        result
    }

    /// Build the full request body JSON.
    fn build_request_body(&self, messages: &[Message]) -> serde_json::Value {
        let mut body = serde_json::json!({
            "model": self.model,
            "messages": self.build_messages(messages),
            "stream": false,
        });

        if !self.tools.is_empty() {
            body["tools"] = serde_json::json!(self.tools);
        }

        body
    }

    /// Map Ollama done_reason to our internal stop_reason.
    ///
    /// Ollama uses: "stop", "tool_calls", "length" (same as OpenAI)
    /// We normalize to: "end_turn", "tool_use", "max_tokens"
    fn map_stop_reason(done_reason: &str) -> &str {
        match done_reason {
            "stop" => "end_turn",
            "tool_calls" => "tool_use",
            "length" => "max_tokens",
            other => other,
        }
    }

    /// Parse the API response body into a `ProviderResponse`.
    ///
    /// Ollama's response is flat (no choices array):
    /// ```json
    /// {
    ///   "message": { "role": "assistant", "content": "...", "tool_calls": [...] },
    ///   "done_reason": "stop"
    /// }
    /// ```
    fn parse_response(body: &ApiResponse) -> Result<ProviderResponse, ProviderError> {
        let mut content_blocks: Vec<ContentBlock> = Vec::new();

        // Add text content if present and non-empty
        if !body.message.content.is_empty() {
            content_blocks.push(ContentBlock::Text {
                text: body.message.content.clone(),
            });
        }

        // Add tool calls if present
        if let Some(ref tool_calls) = body.message.tool_calls {
            for tc in tool_calls {
                let input: serde_json::Value =
                    serde_json::from_str(&tc.function.arguments).unwrap_or(
                        serde_json::Value::Null,
                    );
                content_blocks.push(ContentBlock::ToolUse {
                    id: tc.id.clone().unwrap_or_default(),
                    name: tc.function.name.clone(),
                    input,
                });
            }
        }

        let stop_reason = body
            .done_reason
            .as_deref()
            .map(Self::map_stop_reason)
            .unwrap_or("end_turn");

        Ok(ProviderResponse::new(content_blocks, stop_reason))
    }
}

impl Default for OllamaLoopProvider {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl LoopProvider for OllamaLoopProvider {
    fn name(&self) -> &str {
        "ollama"
    }

    async fn complete(&self, messages: &[Message]) -> Result<ProviderResponse, ProviderError> {
        let request_body = self.build_request_body(messages);

        let response = self
            .client
            .post(&self.api_url())
            .header("Content-Type", "application/json")
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
// API response deserialization types (internal)
// ──────────────────────────────────────────────────────────────────────────────

/// Ollama API chat response (flat format, no choices array).
#[derive(Debug, Deserialize)]
struct ApiResponse {
    message: ApiMessage,
    done_reason: Option<String>,
}

/// The message within the response.
#[derive(Debug, Deserialize)]
struct ApiMessage {
    #[allow(dead_code)]
    role: String,
    #[serde(default)]
    content: String,
    tool_calls: Option<Vec<ApiToolCall>>,
}

/// A tool call in the assistant message.
#[derive(Debug, Deserialize)]
struct ApiToolCall {
    /// Ollama may or may not include an id field for tool calls.
    id: Option<String>,
    #[allow(dead_code)]
    r#type: Option<String>,
    function: ApiFunction,
}

/// The function details within a tool call.
#[derive(Debug, Deserialize)]
struct ApiFunction {
    name: String,
    arguments: String,
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
        let provider = OllamaLoopProvider::new();
        let messages = vec![Message::user("Hello")];
        let result = provider.build_messages(&messages);
        assert_eq!(result.len(), 1);
        assert_eq!(result[0]["role"], "user");
        assert_eq!(result[0]["content"], "Hello");
    }

    #[test]
    fn test_build_messages_with_system_prompt() {
        let provider = OllamaLoopProvider::new()
            .with_system_prompt("You are helpful.");
        let messages = vec![Message::user("Hello")];
        let result = provider.build_messages(&messages);
        assert_eq!(result.len(), 2);
        assert_eq!(result[0]["role"], "system");
        assert_eq!(result[0]["content"], "You are helpful.");
        assert_eq!(result[1]["role"], "user");
    }

    #[test]
    fn test_build_messages_assistant_text() {
        let provider = OllamaLoopProvider::new();
        let messages = vec![Message::assistant(vec![ContentBlock::text("Hi there")])];
        let result = provider.build_messages(&messages);
        assert_eq!(result.len(), 1);
        assert_eq!(result[0]["role"], "assistant");
        assert_eq!(result[0]["content"], "Hi there");
        assert!(result[0].get("tool_calls").is_none());
    }

    #[test]
    fn test_build_messages_assistant_tool_use() {
        let provider = OllamaLoopProvider::new();
        let messages = vec![Message::assistant(vec![
            ContentBlock::text("Let me check"),
            ContentBlock::tool_use(
                "call_1".to_string(),
                "bash".to_string(),
                json!({"command": "ls"}),
            ),
        ])];
        let result = provider.build_messages(&messages);
        assert_eq!(result[0]["role"], "assistant");
        assert_eq!(result[0]["content"], "Let me check");

        let tool_calls = result[0]["tool_calls"].as_array().unwrap();
        assert_eq!(tool_calls.len(), 1);
        assert_eq!(tool_calls[0]["id"], "call_1");
        assert_eq!(tool_calls[0]["type"], "function");
        assert_eq!(tool_calls[0]["function"]["name"], "bash");
    }

    #[test]
    fn test_build_messages_tool_result() {
        let provider = OllamaLoopProvider::new();
        let messages = vec![Message::tool_result(
            "call_1".to_string(),
            "file1.txt\nfile2.txt",
        )];
        let result = provider.build_messages(&messages);
        assert_eq!(result.len(), 1);
        assert_eq!(result[0]["role"], "tool");
        assert_eq!(result[0]["tool_call_id"], "call_1");
        assert_eq!(result[0]["content"], "file1.txt\nfile2.txt");
    }

    #[test]
    fn test_build_messages_multi_tool_results() {
        let provider = OllamaLoopProvider::new();
        let messages = vec![Message::tool_results(vec![
            ("call_1".to_string(), "result 1".to_string()),
            ("call_2".to_string(), "result 2".to_string()),
        ])];
        let result = provider.build_messages(&messages);
        assert_eq!(result.len(), 2);
        assert_eq!(result[0]["role"], "tool");
        assert_eq!(result[0]["tool_call_id"], "call_1");
        assert_eq!(result[1]["role"], "tool");
        assert_eq!(result[1]["tool_call_id"], "call_2");
    }

    #[test]
    fn test_build_messages_full_conversation() {
        let provider = OllamaLoopProvider::new()
            .with_system_prompt("You are a coding assistant.");
        let messages = vec![
            Message::user("List files"),
            Message::assistant(vec![
                ContentBlock::text("I'll run ls for you."),
                ContentBlock::tool_use(
                    "call_1".to_string(),
                    "bash".to_string(),
                    json!({"command": "ls"}),
                ),
            ]),
            Message::tool_result("call_1".to_string(), "README.md\nsrc/"),
            Message::assistant(vec![ContentBlock::text(
                "Here are the files: README.md and src/",
            )]),
        ];

        let result = provider.build_messages(&messages);
        // system + user + assistant + tool + assistant = 5
        assert_eq!(result.len(), 5);
        assert_eq!(result[0]["role"], "system");
        assert_eq!(result[1]["role"], "user");
        assert_eq!(result[2]["role"], "assistant");
        assert_eq!(result[3]["role"], "tool");
        assert_eq!(result[4]["role"], "assistant");
    }

    // ── Request body construction tests ──────────────────────────────────

    #[test]
    fn test_build_request_body_defaults() {
        let provider = OllamaLoopProvider::new();
        let messages = vec![Message::user("Hello")];
        let body = provider.build_request_body(&messages);

        assert_eq!(body["model"], DEFAULT_MODEL);
        assert_eq!(body["stream"], false);
        assert!(body.get("tools").is_none());
    }

    #[test]
    fn test_build_request_body_with_tools() {
        let tools = vec![json!({
            "type": "function",
            "function": {
                "name": "bash",
                "description": "Execute shell commands",
                "parameters": {
                    "type": "object",
                    "properties": {
                        "command": {"type": "string"}
                    },
                    "required": ["command"]
                }
            }
        })];

        let provider = OllamaLoopProvider::new().with_tools(tools.clone());
        let messages = vec![Message::user("Hello")];
        let body = provider.build_request_body(&messages);

        assert_eq!(body["tools"], json!(tools));
    }

    #[test]
    fn test_build_request_body_custom_model() {
        let provider = OllamaLoopProvider::new().with_model("mistral");
        let messages = vec![Message::user("Hello")];
        let body = provider.build_request_body(&messages);

        assert_eq!(body["model"], "mistral");
    }

    #[test]
    fn test_build_request_body_no_max_tokens() {
        // Ollama does not use max_tokens in the request body
        let provider = OllamaLoopProvider::new();
        let messages = vec![Message::user("Hello")];
        let body = provider.build_request_body(&messages);

        assert!(body.get("max_tokens").is_none());
    }

    // ── Response parsing tests ───────────────────────────────────────────

    #[test]
    fn test_parse_response_text_only() {
        let api_response = ApiResponse {
            message: ApiMessage {
                role: "assistant".to_string(),
                content: "Hello! How can I help?".to_string(),
                tool_calls: None,
            },
            done_reason: Some("stop".to_string()),
        };

        let result = OllamaLoopProvider::parse_response(&api_response).unwrap();
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
            message: ApiMessage {
                role: "assistant".to_string(),
                content: "Let me check that.".to_string(),
                tool_calls: Some(vec![ApiToolCall {
                    id: Some("call_abc123".to_string()),
                    r#type: Some("function".to_string()),
                    function: ApiFunction {
                        name: "bash".to_string(),
                        arguments: r#"{"command":"ls -la"}"#.to_string(),
                    },
                }]),
            },
            done_reason: Some("tool_calls".to_string()),
        };

        let result = OllamaLoopProvider::parse_response(&api_response).unwrap();
        assert_eq!(result.stop_reason, "tool_use");
        assert_eq!(result.content.len(), 2); // text + tool_use
        assert!(result.has_tool_use());

        let tool_uses = result.tool_uses();
        assert_eq!(tool_uses.len(), 1);
        assert_eq!(tool_uses[0].0, "call_abc123");
        assert_eq!(tool_uses[0].1, "bash");
        assert_eq!(tool_uses[0].2["command"], "ls -la");
    }

    #[test]
    fn test_parse_response_no_done_reason() {
        // Ollama may omit done_reason in some cases
        let api_response = ApiResponse {
            message: ApiMessage {
                role: "assistant".to_string(),
                content: "Hello!".to_string(),
                tool_calls: None,
            },
            done_reason: None,
        };

        let result = OllamaLoopProvider::parse_response(&api_response).unwrap();
        assert_eq!(result.stop_reason, "end_turn"); // default
    }

    #[test]
    fn test_parse_response_empty_content_with_tool_calls() {
        let api_response = ApiResponse {
            message: ApiMessage {
                role: "assistant".to_string(),
                content: String::new(),
                tool_calls: Some(vec![ApiToolCall {
                    id: Some("call_1".to_string()),
                    r#type: Some("function".to_string()),
                    function: ApiFunction {
                        name: "bash".to_string(),
                        arguments: r#"{"command":"ls"}"#.to_string(),
                    },
                }]),
            },
            done_reason: Some("tool_calls".to_string()),
        };

        let result = OllamaLoopProvider::parse_response(&api_response).unwrap();
        assert_eq!(result.content.len(), 1); // only tool_use, no text
        assert!(result.has_tool_use());
        assert!(result.first_text().is_none());
    }

    #[test]
    fn test_parse_response_tool_call_no_id() {
        // Ollama may not include id for tool calls
        let api_response = ApiResponse {
            message: ApiMessage {
                role: "assistant".to_string(),
                content: String::new(),
                tool_calls: Some(vec![ApiToolCall {
                    id: None,
                    r#type: None,
                    function: ApiFunction {
                        name: "bash".to_string(),
                        arguments: r#"{"command":"ls"}"#.to_string(),
                    },
                }]),
            },
            done_reason: Some("tool_calls".to_string()),
        };

        let result = OllamaLoopProvider::parse_response(&api_response).unwrap();
        let tool_uses = result.tool_uses();
        assert_eq!(tool_uses.len(), 1);
        assert_eq!(tool_uses[0].0, ""); // defaults to empty string
        assert_eq!(tool_uses[0].1, "bash");
    }

    // ── Stop reason mapping tests ────────────────────────────────────────

    #[test]
    fn test_map_stop_reason() {
        assert_eq!(OllamaLoopProvider::map_stop_reason("stop"), "end_turn");
        assert_eq!(
            OllamaLoopProvider::map_stop_reason("tool_calls"),
            "tool_use"
        );
        assert_eq!(
            OllamaLoopProvider::map_stop_reason("length"),
            "max_tokens"
        );
        assert_eq!(
            OllamaLoopProvider::map_stop_reason("unknown"),
            "unknown"
        );
    }

    // ── JSON deserialization tests (simulating raw API responses) ─────────

    #[test]
    fn test_deserialize_text_response() {
        let raw = json!({
            "model": "llama3.2",
            "message": {
                "role": "assistant",
                "content": "Hello world"
            },
            "done": true,
            "done_reason": "stop",
            "total_duration": 1234567
        });
        let api_response: ApiResponse = serde_json::from_value(raw).unwrap();
        let result = OllamaLoopProvider::parse_response(&api_response).unwrap();
        assert_eq!(result.first_text(), Some("Hello world".to_string()));
        assert_eq!(result.stop_reason, "end_turn");
    }

    #[test]
    fn test_deserialize_tool_use_response() {
        let raw = json!({
            "model": "llama3.2",
            "message": {
                "role": "assistant",
                "content": "I'll list the files.",
                "tool_calls": [{
                    "id": "call_xyz",
                    "type": "function",
                    "function": {
                        "name": "bash",
                        "arguments": "{\"command\":\"ls -la\"}"
                    }
                }]
            },
            "done": true,
            "done_reason": "tool_calls"
        });
        let api_response: ApiResponse = serde_json::from_value(raw).unwrap();
        let result = OllamaLoopProvider::parse_response(&api_response).unwrap();

        assert_eq!(result.stop_reason, "tool_use");
        assert!(result.has_tool_use());
        let uses = result.tool_uses();
        assert_eq!(uses[0].1, "bash");
    }

    // ── Builder / constructor tests ──────────────────────────────────────

    #[test]
    fn test_new_defaults() {
        let provider = OllamaLoopProvider::new();
        assert_eq!(provider.model, DEFAULT_MODEL);
        assert_eq!(provider.endpoint, DEFAULT_ENDPOINT);
        assert!(provider.system_prompt.is_none());
        assert!(provider.tools.is_empty());
    }

    #[test]
    fn test_builder_chain() {
        let provider = OllamaLoopProvider::new()
            .with_model("mistral")
            .with_endpoint("http://myserver:11434")
            .with_system_prompt("Be concise.")
            .with_tools(vec![json!({"type": "function", "function": {"name": "bash"}})]);

        assert_eq!(provider.model, "mistral");
        assert_eq!(provider.endpoint, "http://myserver:11434");
        assert_eq!(provider.system_prompt, Some("Be concise.".to_string()));
        assert_eq!(provider.tools.len(), 1);
    }

    #[test]
    fn test_provider_name() {
        let provider = OllamaLoopProvider::new();
        assert_eq!(provider.name(), "ollama");
    }

    #[test]
    fn test_api_url() {
        let provider = OllamaLoopProvider::new();
        assert_eq!(provider.api_url(), "http://localhost:11434/api/chat");

        let provider = OllamaLoopProvider::new()
            .with_endpoint("http://myserver:8080");
        assert_eq!(provider.api_url(), "http://myserver:8080/api/chat");
    }

    #[test]
    fn test_default_impl() {
        let provider = OllamaLoopProvider::default();
        assert_eq!(provider.model, DEFAULT_MODEL);
        assert_eq!(provider.endpoint, DEFAULT_ENDPOINT);
    }

    // ── Round-trip: build request -> parse response ──────────────────────

    #[test]
    fn test_full_conversation_roundtrip() {
        let provider = OllamaLoopProvider::new()
            .with_system_prompt("You are a coding assistant.")
            .with_tools(vec![json!({
                "type": "function",
                "function": {
                    "name": "bash",
                    "description": "Run a shell command",
                    "parameters": {
                        "type": "object",
                        "properties": {"command": {"type": "string"}},
                        "required": ["command"]
                    }
                }
            })]);

        let messages = vec![
            Message::user("What files are here?"),
            Message::assistant(vec![
                ContentBlock::text("I'll check with ls."),
                ContentBlock::tool_use(
                    "call_1".to_string(),
                    "bash".to_string(),
                    json!({"command": "ls"}),
                ),
            ]),
            Message::tool_result("call_1".to_string(), "README.md\nsrc/\nCargo.toml"),
        ];

        let body = provider.build_request_body(&messages);

        // Verify structure
        assert_eq!(body["model"], DEFAULT_MODEL);
        assert_eq!(body["stream"], false);
        assert_eq!(body["tools"].as_array().unwrap().len(), 1);

        let api_messages = body["messages"].as_array().unwrap();
        // system + user + assistant + tool = 4
        assert_eq!(api_messages.len(), 4);

        assert_eq!(api_messages[0]["role"], "system");
        assert_eq!(api_messages[0]["content"], "You are a coding assistant.");
        assert_eq!(api_messages[1]["role"], "user");
        assert_eq!(api_messages[2]["role"], "assistant");
        assert_eq!(api_messages[2]["content"], "I'll check with ls.");
        assert_eq!(api_messages[3]["role"], "tool");
        assert_eq!(api_messages[3]["tool_call_id"], "call_1");
    }
}
