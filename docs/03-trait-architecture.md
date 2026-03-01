# Trait-Driven Architecture

Zero's entire system is built on **Trait-First Design** - a philosophy where all core capabilities are defined as traits rather than concrete implementations. This enables maximum flexibility, testability, and extensibility.

## Table of Contents

- [Quick Overview](#quick-overview)
- [The 5 Core Traits](#the-5-core-traits)
  - [Agent Trait](#agent-trait)
  - [Tool Trait](#tool-trait)
  - [GlobalSharedMemory Trait](#globalsharedmemory-trait)
  - [LLMProvider Trait](#llmprovider-trait)
  - [Channel Trait](#channel-trait)
- [Execution Flow: The Agent Loop](#execution-flow-the-agent-loop)
- [Data Flow Architecture](#data-flow-architecture)
- [Layered Architecture](#layered-architecture)
- [Extension Mechanisms](#extension-mechanisms)
- [Common Design Patterns](#common-design-patterns)
- [Performance Considerations](#performance-considerations)

---

## Quick Overview

Here's a quick comparison table of the 5 core traits:

| Trait | Purpose | Key Methods | Responsibility |
|-------|---------|------------|-----------------|
| **Agent** | Agent factory and execution engine | `execute()`, `name()`, `system_prompt()` | Orchestrates the Agent loop and interaction with other components |
| **Tool** | Unified tool abstraction | `execute()`, `metadata()`, `validate_input()` | Provides concrete capabilities (bash, file operations, etc.) |
| **GlobalSharedMemory** | Cross-Agent shared memory | `store()`, `retrieve()`, `search()`, `delete()` | Manages persistent knowledge across agents |
| **LLMProvider** | LLM provider abstraction | `complete()`, `complete_with_tools()`, `capabilities()` | Interfaces with language models (Anthropic, OpenAI, etc.) |
| **Channel** | Message channel abstraction | `send()`, `receive()`, `connect()`, `disconnect()` | Handles communication (CLI, Slack, Discord, etc.) |

---

## The 5 Core Traits

### Agent Trait

#### Purpose

The **Agent Trait** is the core of the Zero system. It defines the interface for agents that can reason, plan, and execute tasks. An Agent acts as an orchestrator that coordinates with providers, tools, memory, and channels.

#### Definition and Interface

```rust
#[async_trait]
pub trait Agent: Send + Sync {
    /// Agent name
    fn name(&self) -> &str {
        ""
    }

    /// Agent system prompt
    fn system_prompt(&self) -> &str {
        ""
    }

    /// Agent description
    fn description(&self) -> &str {
        ""
    }

    /// Execute Agent (async)
    async fn execute(&self, context: &AgentContext) -> Result<AgentResponse, AgentError>;
}
```

**Key Methods:**
- `name()` - Returns the agent's identifier
- `system_prompt()` - Returns the system prompt that guides the agent's behavior
- `description()` - Returns a human-readable description
- `execute()` - The main execution method that runs the agent loop

#### Key Concepts

The Agent doesn't directly call the LLM or execute tools. Instead, it uses:
- **LoopProvider**: A provider trait for making LLM calls within the loop
- **ToolDispatcher**: A dispatcher for executing tool calls
- **HookManager**: Optional hooks for observability

#### Execution Flow Within Agent

```
Agent.execute() called
    ↓
Initialize message history
    ↓
Loop while not finished:
    ├─ Fire: AgentHook::on_agent_run()
    ├─ Call: LoopProvider.complete()
    ├─ Parse response (check stop_reason)
    ├─ If "tool_use":
    │   ├─ Fire: ToolHook::on_tool_execute()
    │   ├─ Execute tools via ToolDispatcher
    │   ├─ Fire: ToolHook::on_tool_execute_done()
    │   └─ Append tool results to messages
    ├─ If "end_turn":
    │   └─ Break loop
    ├─ Fire: AgentHook::on_agent_run_done()
    └─ Continue or return
    ↓
Return AgentResponse
```

#### Implementation Example: Basic Agent

```rust
use zero_core::{Agent, AgentContext, AgentResponse, AgentError};
use async_trait::async_trait;

pub struct MyAgent {
    name: String,
    system_prompt: String,
}

impl MyAgent {
    pub fn new(name: String, system_prompt: String) -> Self {
        Self { name, system_prompt }
    }
}

#[async_trait]
impl Agent for MyAgent {
    fn name(&self) -> &str {
        &self.name
    }

    fn system_prompt(&self) -> &str {
        &self.system_prompt
    }

    fn description(&self) -> &str {
        "A basic custom agent"
    }

    async fn execute(&self, context: &AgentContext) -> Result<AgentResponse, AgentError> {
        // The actual loop is typically handled by AgentLoop
        // This is where you'd implement custom logic
        let response = AgentResponse {
            content: "Hello, world!".to_string(),
            tool_calls: vec![],
            metadata: Default::default(),
        };
        Ok(response)
    }
}
```

#### Extension Points

- **Custom System Prompts**: Modify behavior by changing the system prompt
- **Agent Types**: Create specialized agents (analyst, developer, researcher, etc.)
- **State Management**: Agents can maintain internal state between calls
- **Tool Selection**: Different agents can have different tool sets

#### Common Patterns

1. **Agent Factory Pattern**: Create agents dynamically based on configuration
2. **Agent Specialization**: Different agents for different domains
3. **Agent Delegation**: Agents creating sub-agents for complex tasks
4. **Agent Composition**: Multiple agents working together on different aspects

---

### Tool Trait

#### Purpose

The **Tool Trait** is the abstraction for all external capabilities. Tools are how agents interact with the external world - executing commands, reading files, calling APIs, etc.

#### Definition and Interface

```rust
#[async_trait]
pub trait Tool: Send + Sync {
    /// Tool metadata
    fn metadata(&self) -> ToolMetadata;

    /// Execute Tool
    async fn execute(&self, input: &str, ctx: &ToolContext) -> Result<ToolOutput, ToolError>;

    /// Optional: Validate input
    fn validate_input(&self, _input: &str) -> Result<(), ToolError> {
        Ok(())
    }
}

pub struct ToolMetadata {
    pub name: String,
    pub description: String,
    pub input_schema: Value,
}

pub enum ToolOutput {
    Text(String),
    Image { data: Vec<u8>, mime_type: String },
    Video { data: Vec<u8>, mime_type: String },
    Audio { data: Vec<u8>, mime_type: String },
}
```

**Key Methods:**
- `metadata()` - Returns tool information (name, description, JSON schema)
- `execute()` - The main method that runs the tool with given input
- `validate_input()` - Optional validation before execution

#### Tool Context

```rust
pub struct ToolContext {
    pub session_id: String,        // Unique session ID
    pub working_dir: Option<String>, // Working directory for the tool
}
```

#### Implementation Example: Custom Tool

```rust
use zero_core::{Tool, ToolMetadata, ToolOutput, ToolContext, ToolError};
use async_trait::async_trait;
use serde_json::json;

pub struct CalculatorTool;

#[async_trait]
impl Tool for CalculatorTool {
    fn metadata(&self) -> ToolMetadata {
        ToolMetadata {
            name: "calculator".to_string(),
            description: "Performs basic arithmetic calculations".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "expression": {
                        "type": "string",
                        "description": "Math expression to evaluate"
                    }
                },
                "required": ["expression"]
            }),
        }
    }

    async fn execute(
        &self,
        input: &str,
        _ctx: &ToolContext,
    ) -> Result<ToolOutput, ToolError> {
        // Parse and evaluate the expression
        let result = format!("Result: {}", input);
        Ok(ToolOutput::text(result))
    }

    fn validate_input(&self, input: &str) -> Result<(), ToolError> {
        if input.is_empty() {
            Err(ToolError::ValidationFailed("Empty expression".into()))
        } else {
            Ok(())
        }
    }
}
```

#### Tool Registry

Tools are managed by the **ToolRegistry**, which stores and retrieves tools by name:

```rust
pub struct ToolRegistry {
    tools: Arc<RwLock<HashMap<String, Box<dyn Tool>>>>,
}

impl ToolRegistry {
    pub fn new() -> Self { /* ... */ }

    pub async fn register(&self, tool: Box<dyn Tool>) { /* ... */ }

    pub async fn list(&self) -> Vec<String> { /* ... */ }
}
```

**Usage:**
```rust
let registry = Arc::new(ToolRegistry::new());
registry.register(Box::new(CalculatorTool)).await;
```

#### Tool Execution Flow

```
Tool call requested by Agent
    ↓
Fire: ToolHook::on_tool_validate()
    ↓
Call: Tool::validate_input()
    ├─ Valid → Continue
    └─ Invalid → Return error
    ↓
Fire: ToolHook::on_tool_execute()
    ↓
Call: Tool::execute(input, context)
    ├─ Success → Capture output
    └─ Error → Capture error
    ↓
Fire: ToolHook::on_tool_execute_done()
    ↓
Return ToolOutput or ToolError
```

#### Extension Points

- **Custom Tools**: Implement any tool by implementing the `Tool` trait
- **Tool Middleware**: Hooks allow adding logging, caching, rate limiting
- **Tool Validation**: Custom validation logic before execution
- **Tool Output Processing**: Different output types (text, images, video, audio)

#### Common Patterns

1. **Tool Composition**: Combine multiple simple tools into complex workflows
2. **Tool Chains**: One tool's output feeds into another tool's input
3. **Conditional Tool Selection**: Different tools based on task type
4. **Tool Fallbacks**: Try one tool, fall back to another if it fails

---

### GlobalSharedMemory Trait

#### Purpose

The **GlobalSharedMemory Trait** provides a persistent knowledge store shared across all agents in the system. It's the agent's "long-term memory" for storing facts, decisions, and learned patterns.

#### Definition and Interface

```rust
#[async_trait]
pub trait GlobalSharedMemory: Send + Sync {
    /// Store memory entry
    async fn store(
        &self,
        namespace: &str,
        key: &str,
        value: &str,
    ) -> Result<(), MemoryError>;

    /// Retrieve memory entry
    async fn retrieve(
        &self,
        namespace: &str,
        key: &str,
    ) -> Result<Option<String>, MemoryError>;

    /// Search memory entries
    async fn search(
        &self,
        namespace: &str,
        query: &str,
        limit: usize,
    ) -> Result<Vec<MemoryEntry>, MemoryError>;

    /// Delete memory entry
    async fn delete(
        &self,
        namespace: &str,
        key: &str,
    ) -> Result<(), MemoryError>;

    /// List all keys in namespace
    async fn list_keys(&self, namespace: &str) -> Result<Vec<String>, MemoryError>;
}

pub struct MemoryEntry {
    pub key: String,
    pub value: String,
    pub timestamp: i64,
    pub metadata: HashMap<String, String>,
}
```

**Key Methods:**
- `store()` - Persist a key-value pair to a namespace
- `retrieve()` - Get a value by key
- `search()` - Full-text search across entries
- `delete()` - Remove an entry
- `list_keys()` - List all available keys in a namespace

#### Layered Memory System

Zero supports a **layered memory architecture**:

```
┌─────────────────────────────────┐
│  Session Memory (ephemeral)     │  ← Short-term, in-memory
├─────────────────────────────────┤
│  Conversation Memory (current)   │  ← Current task context
├─────────────────────────────────┤
│  Global Shared Memory (persistent)│ ← Cross-agent knowledge
└─────────────────────────────────┘
```

**Namespace Organization:**
- `agent/{agent_id}/` - Agent-specific memories
- `task/{task_id}/` - Task-specific memories
- `domain/{domain}/` - Domain-specific facts
- `user/{user_id}/` - User preferences and history

#### Implementation Example: Filesystem Backend

```rust
use zero_core::GlobalSharedMemory;
use std::path::PathBuf;

pub struct FilesystemMemory {
    base_path: PathBuf,
}

impl FilesystemMemory {
    pub fn new(base_path: PathBuf) -> Self {
        Self { base_path }
    }

    fn namespace_path(&self, namespace: &str) -> PathBuf {
        self.base_path.join("memory").join(namespace)
    }
}

#[async_trait]
impl GlobalSharedMemory for FilesystemMemory {
    async fn store(
        &self,
        namespace: &str,
        key: &str,
        value: &str,
    ) -> Result<(), MemoryError> {
        let path = self.namespace_path(namespace)
            .join(format!("{}.json", key));
        std::fs::create_dir_all(path.parent().unwrap())?;
        std::fs::write(&path, value)?;
        Ok(())
    }

    async fn retrieve(
        &self,
        namespace: &str,
        key: &str,
    ) -> Result<Option<String>, MemoryError> {
        let path = self.namespace_path(namespace)
            .join(format!("{}.json", key));
        match std::fs::read_to_string(&path) {
            Ok(v) => Ok(Some(v)),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(None),
            Err(e) => Err(MemoryError::RetrieveFailed(e.to_string())),
        }
    }

    async fn search(
        &self,
        _namespace: &str,
        _query: &str,
        _limit: usize,
    ) -> Result<Vec<MemoryEntry>, MemoryError> {
        // Implement full-text search
        Ok(Vec::new())
    }

    async fn delete(
        &self,
        namespace: &str,
        key: &str,
    ) -> Result<(), MemoryError> {
        let path = self.namespace_path(namespace)
            .join(format!("{}.json", key));
        std::fs::remove_file(&path)?;
        Ok(())
    }

    async fn list_keys(&self, namespace: &str) -> Result<Vec<String>, MemoryError> {
        let dir = self.namespace_path(namespace);
        let mut keys = Vec::new();
        let entries = std::fs::read_dir(&dir)?;
        for entry in entries {
            if let Some(name) = entry?.path().file_stem() {
                keys.push(name.to_string_lossy().to_string());
            }
        }
        Ok(keys)
    }
}
```

#### Extension Points

- **Database Backends**: Implement with SQLite, PostgreSQL, etc.
- **Vector Search**: Use embeddings for semantic search
- **Compression**: Implement memory compaction strategies
- **Replication**: Distribute memory across multiple nodes
- **Caching**: Add caching layers for performance

---

### LLMProvider Trait

#### Purpose

The **LLMProvider Trait** abstracts language model interactions. It allows Zero to work with any LLM provider without changing the core loop logic.

#### Definition and Interface

```rust
pub enum ModelCapability {
    TextOnly,
    TextAndImages,
    TextAndVideo,
    Multimodal,
}

pub enum MediaInput {
    Image { url: String, mime_type: String },
    ImageBytes { data: Vec<u8>, mime_type: String },
    Video { url: String, mime_type: String },
    Audio { url: String, mime_type: String },
}

pub struct CompleteOpts {
    pub model: String,
    pub temperature: Option<f32>,
    pub max_tokens: Option<usize>,
    pub tools: Vec<ToolMetadata>,
    pub system_prompt: Option<String>,
}

#[async_trait]
pub trait LLMProvider: Send + Sync {
    /// Provider name
    fn name(&self) -> &str;

    /// Supported model capabilities
    fn capabilities(&self) -> ModelCapability;

    /// Available models list
    fn available_models(&self) -> Vec<String>;

    /// Plain text completion
    async fn complete(
        &self,
        prompt: &str,
        opts: CompleteOpts,
    ) -> Result<String, ProviderError>;

    /// Multimodal completion (optional)
    async fn complete_with_media(
        &self,
        prompt: &str,
        media: &[MediaInput],
        opts: CompleteOpts,
    ) -> Result<String, ProviderError>;

    /// Tool-calling completion (optional)
    async fn complete_with_tools(
        &self,
        prompt: &str,
        tools: &[ToolCall],
        opts: CompleteOpts,
    ) -> Result<ToolCallResult, ProviderError>;
}
```

**Key Methods:**
- `name()` - Provider identifier (e.g., "anthropic", "openai")
- `capabilities()` - Returns what this provider can do
- `complete()` - Basic text completion
- `complete_with_media()` - Multimodal completion with images, video, etc.
- `complete_with_tools()` - Tool-calling (structured output)

#### Implementation Example: Anthropic Provider

```rust
use zero_core::LLMProvider;
use async_trait::async_trait;

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

    fn capabilities(&self) -> ModelCapability {
        ModelCapability::Multimodal
    }

    fn available_models(&self) -> Vec<String> {
        vec![
            "claude-3-opus".to_string(),
            "claude-3-sonnet".to_string(),
            "claude-3-haiku".to_string(),
        ]
    }

    async fn complete(
        &self,
        prompt: &str,
        opts: CompleteOpts,
    ) -> Result<String, ProviderError> {
        // Make API call to Anthropic
        // Parse response and return text
        Ok("Response from Claude".to_string())
    }

    async fn complete_with_media(
        &self,
        prompt: &str,
        media: &[MediaInput],
        opts: CompleteOpts,
    ) -> Result<String, ProviderError> {
        // Claude supports multimodal - implement media handling
        Ok("Multimodal response".to_string())
    }

    async fn complete_with_tools(
        &self,
        prompt: &str,
        tools: &[ToolCall],
        opts: CompleteOpts,
    ) -> Result<ToolCallResult, ProviderError> {
        // Claude supports tool use - implement tool calling
        Ok(ToolCallResult {
            id: "call_123".to_string(),
            result: Ok("Tool result".to_string()),
        })
    }
}
```

#### Multi-Model Support

Zero supports multiple providers and models:

```
┌────────────────────────────────────┐
│     Provider Router                │
├────────────────────────────────────┤
│ ├─ Anthropic (Claude 3 family)    │
│ ├─ OpenAI (GPT-4, GPT-3.5)        │
│ ├─ Ollama (Local models)          │
│ └─ Custom providers               │
└────────────────────────────────────┘
```

#### Capability Routing

Agents can request specific capabilities, and the system routes to appropriate providers:

```rust
// Agent wants multimodal capability
if needs_image_analysis {
    // Route to provider with TextAndImages or Multimodal capability
    let provider = router.select_by_capability(ModelCapability::Multimodal)?;
}

// Agent wants tool calling
if needs_tool_use {
    // Use provider that supports tool calling
    let provider = router.select_by_feature("tool_calling")?;
}
```

#### Extension Points

- **New Providers**: Implement the trait for new LLM providers
- **Caching**: Cache responses for repeated prompts
- **Rate Limiting**: Control API call rates
- **Fallback Chains**: Try multiple providers if one fails
- **Cost Optimization**: Route to cheaper models when possible

---

### Channel Trait

#### Purpose

The **Channel Trait** abstracts communication channels. It allows agents to send and receive messages through various platforms (CLI, Slack, Discord, Email, etc.) without changing core logic.

#### Definition and Interface

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    pub id: String,
    pub from: String,
    pub to: String,
    pub content: String,
    pub timestamp: i64,
    pub metadata: HashMap<String, String>,
    pub attachments: Vec<MediaInput>,
}

#[async_trait]
pub trait Channel: Send + Sync {
    /// Channel name
    fn name(&self) -> &str;

    /// Send message
    async fn send(&self, msg: &Message) -> Result<(), ChannelError>;

    /// Receive message (optional)
    async fn receive(&self) -> Result<Option<Message>, ChannelError>;

    /// Connect to channel
    async fn connect(&self) -> Result<(), ChannelError>;

    /// Disconnect from channel
    async fn disconnect(&self) -> Result<(), ChannelError>;
}
```

**Key Methods:**
- `name()` - Channel identifier
- `send()` - Send a message to the channel
- `receive()` - Wait for and receive a message
- `connect()` - Establish connection
- `disconnect()` - Close connection

#### Multiple Channels

Zero supports multiple simultaneous channels:

```
┌─────────────────────────────────┐
│     Channel Registry            │
├─────────────────────────────────┤
│ ├─ CLI (stdio)                 │
│ ├─ Slack                       │
│ ├─ Discord                     │
│ ├─ Email                       │
│ ├─ Telegram                    │
│ ├─ Matrix                      │
│ └─ Custom channels             │
└─────────────────────────────────┘
```

#### Message Format Standardization

All channels use the same message structure:

```rust
let msg = Message {
    id: uuid::Uuid::new_v4().to_string(),
    from: "agent_1".to_string(),
    to: "user_1".to_string(),
    content: "Task complete!".to_string(),
    timestamp: chrono::Utc::now().timestamp(),
    metadata: hashmap! {
        "priority" => "high".to_string(),
    },
    attachments: vec![],
};
```

#### Implementation Example: Discord Channel

```rust
use zero_core::Channel;
use async_trait::async_trait;

pub struct DiscordChannel {
    webhook_url: String,
}

impl DiscordChannel {
    pub fn new(webhook_url: String) -> Self {
        Self { webhook_url }
    }
}

#[async_trait]
impl Channel for DiscordChannel {
    fn name(&self) -> &str {
        "discord"
    }

    async fn send(&self, msg: &Message) -> Result<(), ChannelError> {
        // Send to Discord webhook
        let payload = serde_json::json!({
            "content": msg.content,
            "username": msg.from,
        });

        let client = reqwest::Client::new();
        client
            .post(&self.webhook_url)
            .json(&payload)
            .send()
            .await?;

        Ok(())
    }

    async fn receive(&self) -> Result<Option<Message>, ChannelError> {
        // Discord webhooks are one-way, so no receive
        Err(ChannelError::NotSupported(
            "Discord webhooks don't support receive".into()
        ))
    }

    async fn connect(&self) -> Result<(), ChannelError> {
        // Validate webhook URL
        Ok(())
    }

    async fn disconnect(&self) -> Result<(), ChannelError> {
        Ok(())
    }
}
```

#### Extension Points

- **New Channels**: Implement for any communication platform
- **Message Formatting**: Custom formatting per channel
- **Rate Limiting**: Control message frequency
- **Message Persistence**: Log all messages
- **Encryption**: Add end-to-end encryption

---

## Execution Flow: The Agent Loop

The Agent Loop is the heart of Zero. Here's the detailed execution flow:

```
┌─────────────────────────────────────────────────────────┐
│                    Agent.execute()                      │
└─────────────────────────────────────────────────────────┘
                           ↓
┌─────────────────────────────────────────────────────────┐
│          Initialize: messages = [user_input]            │
└─────────────────────────────────────────────────────────┘
                           ↓
                    ┌──────────────┐
                    │ LOOP START   │
                    └──────────────┘
                           ↓
      ┌────────────────────────────────────────┐
      │ Check: iteration < max_iterations      │
      └────────────────────────────────────────┘
                           ↓
      ┌────────────────────────────────────────┐
      │ Fire: AgentHook::on_agent_run()        │
      └────────────────────────────────────────┘
                           ↓
      ┌────────────────────────────────────────┐
      │ Call: Provider.complete(messages,      │
      │       tools, system_prompt)            │
      └────────────────────────────────────────┘
                           ↓
      ┌────────────────────────────────────────┐
      │ Parse Response:                        │
      │  - Extract text blocks                 │
      │  - Extract tool calls                  │
      │  - Check stop_reason                   │
      └────────────────────────────────────────┘
                           ↓
              ┌────────────────────────┐
              │ stop_reason check       │
              └────────────────────────┘
              ↙              ↖
         "tool_use"        "end_turn"
             ↓                  ↓
    ┌──────────────┐    ┌─────────────────┐
    │ Execute      │    │ Return Final    │
    │ Tools        │    │ Response        │
    └──────────────┘    └─────────────────┘
             ↓                  ↓
    ┌──────────────────────────┴──────────┐
    │ Fire: ToolHook::on_tool_validate()  │
    └──────────────────────────────────────┘
             ↓
    ┌──────────────────────────────────────┐
    │ Call: Tool.validate_input()          │
    └──────────────────────────────────────┘
             ↓
    ┌──────────────────────────────────────┐
    │ Fire: ToolHook::on_tool_execute()   │
    └──────────────────────────────────────┘
             ↓
    ┌──────────────────────────────────────┐
    │ Call: ToolDispatcher.execute(tool)   │
    └──────────────────────────────────────┘
             ↓
    ┌──────────────────────────────────────┐
    │ Fire: ToolHook::on_tool_execute_done()│
    └──────────────────────────────────────┘
             ↓
    ┌──────────────────────────────────────┐
    │ Append tool results to messages      │
    └──────────────────────────────────────┘
             ↓
    ┌──────────────────────────────────────┐
    │ Loop back to Provider.complete()     │
    └──────────────────────────────────────┘
             ↓
           Loop continues...
```

#### Key Loop Variables

```rust
let mut iteration = 0;          // Prevent infinite loops
let mut messages = vec![...];   // Message history
let mut final_response = "";    // Accumulated response text

loop {
    // Check iteration limit
    if iteration >= config.max_iterations {
        return Err(AgentError::MaxIterationsExceeded(iteration));
    }
    iteration += 1;

    // Get response from provider
    let response = provider.complete(&messages, config).await?;

    // Check stop reason
    match response.stop_reason.as_str() {
        "tool_use" => {
            // Execute tools, add results to messages, continue loop
        }
        "end_turn" => {
            // Extract final text and break
            final_response = extract_text(&response);
            break;
        }
        _ => return Err(AgentError::UnexpectedStopReason(...)),
    }
}

Ok(final_response)
```

---

## Data Flow Architecture

Here's the complete data flow through the Zero system:

```
┌─────────────────────────────────────────────────────────┐
│                   User Input                            │
└─────────────────────────────────────────────────────────┘
                           ↓
┌─────────────────────────────────────────────────────────┐
│                   Channel (CLI/Slack/etc)               │
│         Receives message and routes to Agent            │
└─────────────────────────────────────────────────────────┘
                           ↓
┌─────────────────────────────────────────────────────────┐
│                   Agent                                 │
│   - Coordinates execution                              │
│   - Manages message history                            │
│   - Orchestrates loop                                  │
└─────────────────────────────────────────────────────────┘
                           ↓
┌─────────────────────────────────────────────────────────┐
│              AgentLoop: DefaultAgentLoop                │
│   - Implements core reasoning loop                      │
│   - Handles tool execution                             │
│   - Manages iterations                                 │
└─────────────────────────────────────────────────────────┘
          ↙                            ↖
    ┌──────────────┐          ┌─────────────────┐
    │ LLMProvider  │          │ ToolDispatcher  │
    │ (Reasoning)  │          │ (Execution)     │
    └──────────────┘          └─────────────────┘
          ↓                            ↓
    ┌──────────────┐          ┌─────────────────┐
    │ Anthropic    │          │ Tool Registry   │
    │ OpenAI       │          │                 │
    │ Ollama       │          ├─ Bash           │
    │ Custom       │          ├─ File I/O       │
    └──────────────┘          ├─ HTTP           │
                              └─ Custom Tools   │
                                     ↓
                              ┌─────────────────┐
                              │ GlobalSharedMemory
                              │                 │
                              ├─ Filesystem    │
                              ├─ Database      │
                              └─ Custom        │
                                     ↓
                              ┌─────────────────┐
                              │ Stored Knowledge│
                              │ Facts & Patterns│
                              └─────────────────┘
                           ↑
                    Loop back to Provider
                           ↓
┌─────────────────────────────────────────────────────────┐
│                   Final Response                        │
└─────────────────────────────────────────────────────────┘
                           ↓
┌─────────────────────────────────────────────────────────┐
│                   Channel                               │
│         Sends response back to user                     │
└─────────────────────────────────────────────────────────┘
                           ↓
┌─────────────────────────────────────────────────────────┐
│                   User Output                           │
└─────────────────────────────────────────────────────────┘
```

---

## Layered Architecture

Zero is organized in clear layers, each with distinct responsibilities:

```
┌──────────────────────────────────────────────────────────┐
│ Layer 5: Application & Integration                       │
│ ├─ CLI Interface                                         │
│ ├─ Web API                                              │
│ ├─ IDE Integration                                      │
│ └─ Custom Applications                                  │
├──────────────────────────────────────────────────────────┤
│ Layer 4: Runtime & Coordination                          │
│ ├─ AgentLoop (core reasoning loop)                      │
│ ├─ TaskManager (task persistence - S7+)                │
│ ├─ TeamCoordinator (multi-agent - S9+)                 │
│ └─ Scheduler                                            │
├──────────────────────────────────────────────────────────┤
│ Layer 3: Core Trait Layer (The Foundation)              │
│ ├─ Agent Trait ───────────────┐                        │
│ ├─ Tool Trait ────────┐       │                        │
│ ├─ Provider Trait ────┼───────┼─→ These 5 Traits      │
│ ├─ Channel Trait ─────┤       │   define the entire    │
│ └─ Memory Trait ──────┘       │   system's contract    │
│                               │                        │
├──────────────────────────────────────────────────────────┤
│ Layer 2: Implementation Layer                            │
│ ├─ Providers: Anthropic, OpenAI, Ollama               │
│ ├─ Tools: Bash, File, HTTP, Calculator                │
│ ├─ Channels: CLI, Slack, Discord, Email, Telegram     │
│ ├─ Memory Backends: Filesystem, SQLite, PostgreSQL    │
│ └─ Hooks: Observability & Extension Points            │
├──────────────────────────────────────────────────────────┤
│ Layer 1: Infrastructure & Support                       │
│ ├─ Error Handling (thiserror)                          │
│ ├─ Config Management                                   │
│ ├─ Security & Sandboxing                              │
│ ├─ Monitoring & Logging                               │
│ └─ Async Runtime (Tokio)                              │
└──────────────────────────────────────────────────────────┘
```

**Key Design Principle**: Each layer only depends on layers below it. This enables independent testing and extension of each layer.

---

## Extension Mechanisms

### 1. Implementing New Traits

To extend Zero, implement one of the core traits:

```rust
// Example: Custom Tool
use zero_core::{Tool, ToolMetadata, ToolOutput, ToolContext, ToolError};
use async_trait::async_trait;

pub struct MyCustomTool;

#[async_trait]
impl Tool for MyCustomTool {
    fn metadata(&self) -> ToolMetadata { /* ... */ }
    async fn execute(&self, input: &str, ctx: &ToolContext) -> Result<ToolOutput, ToolError> { /* ... */ }
}

// Example: Custom Provider
use zero_core::LLMProvider;

pub struct MyCustomProvider;

#[async_trait]
impl LLMProvider for MyCustomProvider {
    fn name(&self) -> &str { "my_provider" }
    fn capabilities(&self) -> ModelCapability { /* ... */ }
    async fn complete(&self, prompt: &str, opts: CompleteOpts) -> Result<String, ProviderError> { /* ... */ }
}

// Example: Custom Channel
use zero_core::Channel;

pub struct MyCustomChannel;

#[async_trait]
impl Channel for MyCustomChannel {
    fn name(&self) -> &str { "my_channel" }
    async fn send(&self, msg: &Message) -> Result<(), ChannelError> { /* ... */ }
    async fn connect(&self) -> Result<(), ChannelError> { /* ... */ }
}

// Example: Custom Memory Backend
use zero_core::GlobalSharedMemory;

pub struct MyMemoryBackend;

#[async_trait]
impl GlobalSharedMemory for MyMemoryBackend {
    async fn store(&self, namespace: &str, key: &str, value: &str) -> Result<(), MemoryError> { /* ... */ }
    async fn retrieve(&self, namespace: &str, key: &str) -> Result<Option<String>, MemoryError> { /* ... */ }
}
```

### 2. Hook System for Observability

Hooks allow you to observe and modify behavior at key points:

```rust
use zero_core::hooks::{Hook, AgentHook, ToolHook};
use async_trait::async_trait;

pub struct LoggingHook;

#[async_trait]
impl Hook for LoggingHook {
    fn name(&self) -> &str { "logging" }
}

#[async_trait]
impl AgentHook for LoggingHook {
    async fn on_agent_run(&self, agent_name: &str) -> Result<(), String> {
        println!("Agent {} started", agent_name);
        Ok(())
    }

    async fn on_agent_run_done(&self, agent_name: &str, result: &str) -> Result<(), String> {
        println!("Agent {} finished: {}", agent_name, result);
        Ok(())
    }
}

#[async_trait]
impl ToolHook for LoggingHook {
    async fn on_tool_execute(&self, tool_name: &str, input: &str) -> Result<(), String> {
        println!("Executing tool {} with input: {}", tool_name, input);
        Ok(())
    }
}
```

### 3. Registration and Composition

```rust
// Create and compose components
let provider = Arc::new(AnthropicProvider::new(api_key));
let tool_registry = Arc::new(ToolRegistry::new());
let memory = Arc::new(FilesystemMemory::new(path));

// Register tools
tool_registry.register(Box::new(BashTool)).await;
tool_registry.register(Box::new(FileIOTool)).await;

// Register hooks
let hook_manager = Arc::new(HookManager::new());
hook_manager.register_agent_hook(Box::new(LoggingHook));

// Create loop with hooks
let loop_impl = DefaultAgentLoop::new(provider, tool_registry)
    .with_hooks(hook_manager);
```

---

## Common Design Patterns

### 1. Builder Pattern

Build complex configurations step by step:

```rust
let config = AgentLoopConfig::builder()
    .with_max_iterations(50)
    .with_temperature(0.7)
    .with_timeout_secs(30)
    .build();
```

### 2. Factory Pattern

Create agents based on configuration:

```rust
pub fn create_agent(agent_type: &str) -> Arc<dyn Agent> {
    match agent_type {
        "researcher" => Arc::new(ResearcherAgent::new()),
        "developer" => Arc::new(DeveloperAgent::new()),
        "analyst" => Arc::new(AnalystAgent::new()),
        _ => Arc::new(DefaultAgent::new()),
    }
}
```

### 3. Strategy Pattern

Different implementations for different scenarios:

```rust
pub async fn execute_task(
    task: &Task,
    strategy: Box<dyn ExecutionStrategy>,
) -> Result<TaskResult> {
    strategy.execute(task).await
}
```

### 4. Middleware Pattern

Add functionality through hooks:

```rust
pub struct CachingHook {
    cache: Arc<RwLock<HashMap<String, String>>>,
}

#[async_trait]
impl ProviderHook for CachingHook {
    async fn on_provider_call(&self, prompt: &str) -> Result<(), String> {
        if let Some(cached) = self.cache.read().await.get(prompt) {
            // Return cached response
        }
        Ok(())
    }
}
```

### 5. Dependency Injection

Inject dependencies through constructors:

```rust
pub struct Agent {
    provider: Arc<dyn LLMProvider>,
    tools: Arc<ToolRegistry>,
    memory: Arc<dyn GlobalSharedMemory>,
    channels: Arc<ChannelRegistry>,
}

impl Agent {
    pub fn new(
        provider: Arc<dyn LLMProvider>,
        tools: Arc<ToolRegistry>,
        memory: Arc<dyn GlobalSharedMemory>,
        channels: Arc<ChannelRegistry>,
    ) -> Self {
        Self { provider, tools, memory, channels }
    }
}
```

---

## Performance Considerations

### 1. Async-First Design

All I/O operations are async using `async-trait` and `tokio`:

```rust
// Don't block the runtime
async fn execute(&self) -> Result<...> {
    // Use .await for I/O operations
    let result = external_api.call().await?;
    Ok(result)
}
```

**Benefits:**
- Single thread can handle thousands of concurrent operations
- Zero CPU waste on blocking I/O
- Natural support for multi-agent systems

### 2. Concurrency Levels

```
Level 1: Agent Loop       - Single async task per agent
Level 2: Tool Execution   - Configurable concurrent tools
Level 3: Multi-Agent      - Multiple agents in parallel
Level 4: Team Coordination- Lead + N worker agents
```

### 3. Memory Efficiency

- Message history is kept in memory but can be compressed (see compression strategies)
- Tools don't load until needed
- Memory is organized by namespace for efficient cleanup
- Unused entries can be archived to persistent storage

### 4. Timeout Management

```rust
let result = tokio::time::timeout(
    Duration::from_secs(config.tool_timeout),
    dispatcher.execute(tool_call),
).await?;
```

Prevents tools from blocking the entire loop.

### 5. Provider Optimization

- Connection pooling for multiple requests
- Request batching when possible
- Fallback to cheaper models for simple tasks
- Caching of repeated queries

---

## Summary

Zero's trait-driven architecture provides:

1. **Flexibility**: Swap any component without changing others
2. **Testability**: Mock implementations for any trait
3. **Extensibility**: Add new tools, providers, channels with minimal effort
4. **Clarity**: Traits serve as executable contracts
5. **Performance**: Async-first design for high concurrency
6. **Observability**: Hooks at every key point
7. **Composability**: Build complex systems from simple traits

The 5 core traits form the foundation, and everything else is an implementation detail that can be extended, replaced, or customized.

---

## Next Steps

- See **04-examples.md** for practical implementation examples
- Check **05-api-reference.md** for detailed API documentation
- Review **06-hooks-system.md** to learn about observability
- Explore **07-contributing.md** to start extending Zero
