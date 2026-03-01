/// OpenAI LLM provider implementations
///
/// This module provides two implementations:
/// - `OpenAIProvider`: Legacy `LLMProvider` implementation (simple string-in/string-out)
/// - `OpenAILoopProvider`: Full `LoopProvider` implementation with tool calling support

use crate::error::ProviderError;
use crate::message::{ContentBlock, Message, ToolResultContent};
use crate::provider::loop_provider::{LoopProvider, ProviderResponse};
use crate::provider::LLMProvider;
use async_trait::async_trait;
use serde::Deserialize;

// ──────────────────────────────────────────────────────────────────────────────
// Legacy LLMProvider (kept for backward compat)
// ──────────────────────────────────────────────────────────────────────────────

/// Legacy OpenAI provider implementing the simple `LLMProvider` trait.
pub struct OpenAIProvider {
    api_key: String,
}

impl OpenAIProvider {
    pub fn new(api_key: String) -> Self {
        Self { api_key }
    }
}

#[async_trait]
impl LLMProvider for OpenAIProvider {
    fn name(&self) -> &str {
        "openai"
    }

    fn capabilities(&self) -> crate::provider::ModelCapability {
        crate::provider::ModelCapability::Multimodal
    }

    fn available_models(&self) -> Vec<String> {
        vec!["gpt-4".to_string()]
    }

    async fn complete(
        &self,
        prompt: &str,
        _opts: crate::provider::CompleteOpts,
    ) -> Result<String, ProviderError> {
        // Placeholder
        let _ = &self.api_key;
        Ok(format!("Response to: {}", prompt))
    }
}

// ──────────────────────────────────────────────────────────────────────────────
// Real LoopProvider for Agent Loop
// ──────────────────────────────────────────────────────────────────────────────

const DEFAULT_MODEL: &str = "gpt-4o";
const DEFAULT_MAX_TOKENS: u32 = 4096;
const OPENAI_API_URL: &str = "https://api.openai.com/v1/chat/completions";

/// OpenAI provider implementing `LoopProvider` for the Agent loop.
///
/// This provider calls the OpenAI Chat Completions API via HTTP, handles
/// tool-use round-trips, and converts between the internal `Message` format
/// and the OpenAI wire format.
pub struct OpenAILoopProvider {
    client: reqwest::Client,
    api_key: String,
    model: String,
    system_prompt: Option<String>,
    max_tokens: u32,
    tools: Vec<serde_json::Value>,
}

impl OpenAILoopProvider {
    /// Create a new provider with sensible defaults.
    pub fn new(api_key: String) -> Self {
        Self {
            client: reqwest::Client::new(),
            api_key,
            model: DEFAULT_MODEL.to_string(),
            system_prompt: None,
            max_tokens: DEFAULT_MAX_TOKENS,
            tools: Vec::new(),
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

    // ── Internal helpers ─────────────────────────────────────────────────

    /// Convert internal `Message` slice to the OpenAI messages JSON array.
    ///
    /// Key differences from Anthropic format:
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
                        message["content"] = serde_json::Value::Null;
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
            "max_tokens": self.max_tokens,
            "messages": self.build_messages(messages),
        });

        if !self.tools.is_empty() {
            body["tools"] = serde_json::json!(self.tools);
        }

        body
    }

    /// Map OpenAI finish_reason to our internal stop_reason.
    ///
    /// OpenAI uses: "stop", "tool_calls", "length"
    /// We normalize to: "end_turn", "tool_use", "max_tokens"
    fn map_stop_reason(finish_reason: &str) -> &str {
        match finish_reason {
            "stop" => "end_turn",
            "tool_calls" => "tool_use",
            "length" => "max_tokens",
            other => other,
        }
    }

    /// Parse the API response body into a `ProviderResponse`.
    fn parse_response(body: &ApiResponse) -> Result<ProviderResponse, ProviderError> {
        let choice = body.choices.first().ok_or_else(|| {
            ProviderError::InvalidResponse("No choices in response".to_string())
        })?;

        let mut content_blocks: Vec<ContentBlock> = Vec::new();

        // Add text content if present
        if let Some(ref text) = choice.message.content {
            if !text.is_empty() {
                content_blocks.push(ContentBlock::Text {
                    text: text.clone(),
                });
            }
        }

        // Add tool calls if present
        if let Some(ref tool_calls) = choice.message.tool_calls {
            for tc in tool_calls {
                let input: serde_json::Value =
                    serde_json::from_str(&tc.function.arguments).unwrap_or(
                        serde_json::Value::Null,
                    );
                content_blocks.push(ContentBlock::ToolUse {
                    id: tc.id.clone(),
                    name: tc.function.name.clone(),
                    input,
                });
            }
        }

        let stop_reason = Self::map_stop_reason(&choice.finish_reason);

        Ok(ProviderResponse::new(content_blocks, stop_reason))
    }
}

#[async_trait]
impl LoopProvider for OpenAILoopProvider {
    fn name(&self) -> &str {
        "openai"
    }

    async fn complete(&self, messages: &[Message]) -> Result<ProviderResponse, ProviderError> {
        let request_body = self.build_request_body(messages);

        let response = self
            .client
            .post(OPENAI_API_URL)
            .header("Authorization", format!("Bearer {}", self.api_key))
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

/// OpenAI Chat Completions API response (subset of fields we care about).
#[derive(Debug, Deserialize)]
struct ApiResponse {
    choices: Vec<ApiChoice>,
}

/// A single choice in the API response.
#[derive(Debug, Deserialize)]
struct ApiChoice {
    message: ApiMessage,
    finish_reason: String,
}

/// The message within a choice.
#[derive(Debug, Deserialize)]
struct ApiMessage {
    #[allow(dead_code)]
    role: String,
    content: Option<String>,
    tool_calls: Option<Vec<ApiToolCall>>,
}

/// A tool call in the assistant message.
#[derive(Debug, Deserialize)]
struct ApiToolCall {
    id: String,
    #[allow(dead_code)]
    r#type: String,
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
        let provider = OpenAILoopProvider::new("test-key".to_string());
        let messages = vec![Message::user("Hello")];
        let result = provider.build_messages(&messages);
        // No system prompt, so just 1 message
        assert_eq!(result.len(), 1);
        assert_eq!(result[0]["role"], "user");
        assert_eq!(result[0]["content"], "Hello");
    }

    #[test]
    fn test_build_messages_with_system_prompt() {
        let provider = OpenAILoopProvider::new("test-key".to_string())
            .with_system_prompt("You are helpful.");
        let messages = vec![Message::user("Hello")];
        let result = provider.build_messages(&messages);
        // System prompt + user message
        assert_eq!(result.len(), 2);
        assert_eq!(result[0]["role"], "system");
        assert_eq!(result[0]["content"], "You are helpful.");
        assert_eq!(result[1]["role"], "user");
        assert_eq!(result[1]["content"], "Hello");
    }

    #[test]
    fn test_build_messages_assistant_text() {
        let provider = OpenAILoopProvider::new("test-key".to_string());
        let messages = vec![Message::assistant(vec![ContentBlock::text("Hi there")])];
        let result = provider.build_messages(&messages);
        assert_eq!(result.len(), 1);
        assert_eq!(result[0]["role"], "assistant");
        assert_eq!(result[0]["content"], "Hi there");
        // No tool_calls key
        assert!(result[0].get("tool_calls").is_none());
    }

    #[test]
    fn test_build_messages_assistant_tool_use() {
        let provider = OpenAILoopProvider::new("test-key".to_string());
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
        // arguments is a JSON string
        let args: serde_json::Value =
            serde_json::from_str(tool_calls[0]["function"]["arguments"].as_str().unwrap())
                .unwrap();
        assert_eq!(args["command"], "ls");
    }

    #[test]
    fn test_build_messages_tool_result() {
        let provider = OpenAILoopProvider::new("test-key".to_string());
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
        let provider = OpenAILoopProvider::new("test-key".to_string());
        let messages = vec![Message::tool_results(vec![
            ("call_1".to_string(), "result 1".to_string()),
            ("call_2".to_string(), "result 2".to_string()),
        ])];
        let result = provider.build_messages(&messages);
        // Each tool result becomes a separate message
        assert_eq!(result.len(), 2);
        assert_eq!(result[0]["role"], "tool");
        assert_eq!(result[0]["tool_call_id"], "call_1");
        assert_eq!(result[1]["role"], "tool");
        assert_eq!(result[1]["tool_call_id"], "call_2");
    }

    #[test]
    fn test_build_messages_full_conversation() {
        let provider = OpenAILoopProvider::new("test-key".to_string())
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
        let provider = OpenAILoopProvider::new("test-key".to_string());
        let messages = vec![Message::user("Hello")];
        let body = provider.build_request_body(&messages);

        assert_eq!(body["model"], DEFAULT_MODEL);
        assert_eq!(body["max_tokens"], DEFAULT_MAX_TOKENS);
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

        let provider = OpenAILoopProvider::new("test-key".to_string()).with_tools(tools.clone());
        let messages = vec![Message::user("Hello")];
        let body = provider.build_request_body(&messages);

        assert_eq!(body["tools"], json!(tools));
    }

    #[test]
    fn test_build_request_body_custom_model() {
        let provider =
            OpenAILoopProvider::new("test-key".to_string()).with_model("gpt-4-turbo");
        let messages = vec![Message::user("Hello")];
        let body = provider.build_request_body(&messages);

        assert_eq!(body["model"], "gpt-4-turbo");
    }

    #[test]
    fn test_build_request_body_custom_max_tokens() {
        let provider =
            OpenAILoopProvider::new("test-key".to_string()).with_max_tokens(8192);
        let messages = vec![Message::user("Hello")];
        let body = provider.build_request_body(&messages);

        assert_eq!(body["max_tokens"], 8192);
    }

    // ── Response parsing tests ───────────────────────────────────────────

    #[test]
    fn test_parse_response_text_only() {
        let api_response = ApiResponse {
            choices: vec![ApiChoice {
                message: ApiMessage {
                    role: "assistant".to_string(),
                    content: Some("Hello! How can I help?".to_string()),
                    tool_calls: None,
                },
                finish_reason: "stop".to_string(),
            }],
        };

        let result = OpenAILoopProvider::parse_response(&api_response).unwrap();
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
            choices: vec![ApiChoice {
                message: ApiMessage {
                    role: "assistant".to_string(),
                    content: Some("Let me check that.".to_string()),
                    tool_calls: Some(vec![ApiToolCall {
                        id: "call_abc123".to_string(),
                        r#type: "function".to_string(),
                        function: ApiFunction {
                            name: "bash".to_string(),
                            arguments: r#"{"command":"ls -la"}"#.to_string(),
                        },
                    }]),
                },
                finish_reason: "tool_calls".to_string(),
            }],
        };

        let result = OpenAILoopProvider::parse_response(&api_response).unwrap();
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
    fn test_parse_response_multiple_tool_uses() {
        let api_response = ApiResponse {
            choices: vec![ApiChoice {
                message: ApiMessage {
                    role: "assistant".to_string(),
                    content: None,
                    tool_calls: Some(vec![
                        ApiToolCall {
                            id: "call_1".to_string(),
                            r#type: "function".to_string(),
                            function: ApiFunction {
                                name: "bash".to_string(),
                                arguments: r#"{"command":"ls"}"#.to_string(),
                            },
                        },
                        ApiToolCall {
                            id: "call_2".to_string(),
                            r#type: "function".to_string(),
                            function: ApiFunction {
                                name: "read_file".to_string(),
                                arguments: r#"{"path":"README.md"}"#.to_string(),
                            },
                        },
                    ]),
                },
                finish_reason: "tool_calls".to_string(),
            }],
        };

        let result = OpenAILoopProvider::parse_response(&api_response).unwrap();
        assert_eq!(result.tool_uses().len(), 2);
    }

    #[test]
    fn test_parse_response_no_choices() {
        let api_response = ApiResponse {
            choices: vec![],
        };

        let result = OpenAILoopProvider::parse_response(&api_response);
        assert!(result.is_err());
        match result {
            Err(ProviderError::InvalidResponse(msg)) => {
                assert!(msg.contains("No choices"));
            }
            _ => panic!("Expected InvalidResponse error"),
        }
    }

    #[test]
    fn test_parse_response_null_content_with_tool_calls() {
        // OpenAI sometimes sends content: null when there are tool calls
        let api_response = ApiResponse {
            choices: vec![ApiChoice {
                message: ApiMessage {
                    role: "assistant".to_string(),
                    content: None,
                    tool_calls: Some(vec![ApiToolCall {
                        id: "call_1".to_string(),
                        r#type: "function".to_string(),
                        function: ApiFunction {
                            name: "bash".to_string(),
                            arguments: r#"{"command":"ls"}"#.to_string(),
                        },
                    }]),
                },
                finish_reason: "tool_calls".to_string(),
            }],
        };

        let result = OpenAILoopProvider::parse_response(&api_response).unwrap();
        assert_eq!(result.content.len(), 1); // only tool_use, no text
        assert!(result.has_tool_use());
        assert!(result.first_text().is_none());
    }

    // ── Stop reason mapping tests ────────────────────────────────────────

    #[test]
    fn test_map_stop_reason() {
        assert_eq!(OpenAILoopProvider::map_stop_reason("stop"), "end_turn");
        assert_eq!(
            OpenAILoopProvider::map_stop_reason("tool_calls"),
            "tool_use"
        );
        assert_eq!(
            OpenAILoopProvider::map_stop_reason("length"),
            "max_tokens"
        );
        assert_eq!(
            OpenAILoopProvider::map_stop_reason("unknown"),
            "unknown"
        );
    }

    // ── JSON deserialization tests (simulating raw API responses) ─────────

    #[test]
    fn test_deserialize_text_response() {
        let raw = json!({
            "id": "chatcmpl-abc",
            "object": "chat.completion",
            "choices": [{
                "index": 0,
                "message": {
                    "role": "assistant",
                    "content": "Hello world"
                },
                "finish_reason": "stop"
            }],
            "usage": {"prompt_tokens": 10, "completion_tokens": 5}
        });
        let api_response: ApiResponse = serde_json::from_value(raw).unwrap();
        let result = OpenAILoopProvider::parse_response(&api_response).unwrap();
        assert_eq!(result.first_text(), Some("Hello world".to_string()));
        assert_eq!(result.stop_reason, "end_turn");
    }

    #[test]
    fn test_deserialize_tool_use_response() {
        let raw = json!({
            "id": "chatcmpl-xyz",
            "choices": [{
                "index": 0,
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
                "finish_reason": "tool_calls"
            }]
        });
        let api_response: ApiResponse = serde_json::from_value(raw).unwrap();
        let result = OpenAILoopProvider::parse_response(&api_response).unwrap();

        assert_eq!(result.stop_reason, "tool_use");
        assert!(result.has_tool_use());
        let uses = result.tool_uses();
        assert_eq!(uses[0].1, "bash");
    }

    // ── Builder / constructor tests ──────────────────────────────────────

    #[test]
    fn test_new_defaults() {
        let provider = OpenAILoopProvider::new("sk-test".to_string());
        assert_eq!(provider.model, DEFAULT_MODEL);
        assert_eq!(provider.max_tokens, DEFAULT_MAX_TOKENS);
        assert!(provider.system_prompt.is_none());
        assert!(provider.tools.is_empty());
        assert_eq!(provider.api_key, "sk-test");
    }

    #[test]
    fn test_builder_chain() {
        let provider = OpenAILoopProvider::new("sk-test".to_string())
            .with_model("gpt-4-turbo")
            .with_system_prompt("Be concise.")
            .with_max_tokens(2048)
            .with_tools(vec![json!({"type": "function", "function": {"name": "bash"}})]);

        assert_eq!(provider.model, "gpt-4-turbo");
        assert_eq!(provider.system_prompt, Some("Be concise.".to_string()));
        assert_eq!(provider.max_tokens, 2048);
        assert_eq!(provider.tools.len(), 1);
    }

    #[test]
    fn test_provider_name() {
        let provider = OpenAILoopProvider::new("sk-test".to_string());
        assert_eq!(provider.name(), "openai");
    }

    // ── Round-trip: build request -> parse response ──────────────────────

    #[test]
    fn test_full_conversation_roundtrip() {
        let provider = OpenAILoopProvider::new("sk-test".to_string())
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

        // Simulate a multi-turn conversation
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
        assert_eq!(body["tools"].as_array().unwrap().len(), 1);

        let api_messages = body["messages"].as_array().unwrap();
        // system + user + assistant + tool = 4
        assert_eq!(api_messages.len(), 4);

        // First message: system
        assert_eq!(api_messages[0]["role"], "system");
        assert_eq!(api_messages[0]["content"], "You are a coding assistant.");

        // Second message: user
        assert_eq!(api_messages[1]["role"], "user");
        assert_eq!(api_messages[1]["content"], "What files are here?");

        // Third message: assistant with tool_calls
        assert_eq!(api_messages[2]["role"], "assistant");
        assert_eq!(api_messages[2]["content"], "I'll check with ls.");
        let tool_calls = api_messages[2]["tool_calls"].as_array().unwrap();
        assert_eq!(tool_calls.len(), 1);
        assert_eq!(tool_calls[0]["function"]["name"], "bash");

        // Fourth message: tool result
        assert_eq!(api_messages[3]["role"], "tool");
        assert_eq!(api_messages[3]["tool_call_id"], "call_1");
    }

    #[test]
    fn test_assistant_only_tool_use_no_text() {
        // Test assistant message with only tool use, no text
        let provider = OpenAILoopProvider::new("test-key".to_string());
        let messages = vec![Message::assistant(vec![ContentBlock::tool_use(
            "call_1".to_string(),
            "bash".to_string(),
            json!({"command": "ls"}),
        )])];

        let result = provider.build_messages(&messages);
        assert_eq!(result[0]["role"], "assistant");
        // content should be null when there is no text
        assert!(result[0]["content"].is_null());
        assert!(result[0]["tool_calls"].is_array());
    }
}
