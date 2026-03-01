# API Reference

> **Back to Home**: [README.md](../README.md)

Complete API documentation for the Zero Core framework. This guide covers all trait definitions, methods, type definitions, and error handling.

## Table of Contents

- [Agent API](#agent-api)
- [Tool API](#tool-api)
- [Memory API](#memory-api)
- [Provider API](#provider-api)
- [Channel API](#channel-api)
- [Type Definitions](#type-definitions)
- [Error Types](#error-types)
- [Common Patterns](#common-patterns)

## Agent API

The Agent trait defines the core interface for autonomous agents in the Zero framework.

### Interface Definition

```rust
#[async_trait]
pub trait Agent: Send + Sync {
    fn name(&self) -> &str;
    fn system_prompt(&self) -> &str;
    fn description(&self) -> &str;
    async fn execute(&self, context: &AgentContext) -> Result<AgentResponse, AgentError>;
}
```

### Methods

#### name()

Returns the name of the agent.

- **Signature**: `fn name(&self) -> &str`
- **Returns**: A string slice containing the agent's name
- **Example**:
  ```rust
  let agent_name = agent.name();
  println!("Agent: {}", agent_name);
  ```

#### system_prompt()

Returns the system prompt that defines the agent's behavior and instructions.

- **Signature**: `fn system_prompt(&self) -> &str`
- **Returns**: A string slice containing the system prompt
- **Example**:
  ```rust
  let prompt = agent.system_prompt();
  ```

#### description()

Returns a description of the agent's purpose and capabilities.

- **Signature**: `fn description(&self) -> &str`
- **Returns**: A string slice describing the agent
- **Example**:
  ```rust
  let desc = agent.description();
  println!("About: {}", desc);
  ```

#### execute()

Executes the agent with the given context. This is the main method that runs the agent logic.

- **Signature**: `async fn execute(&self, context: &AgentContext) -> Result<AgentResponse, AgentError>`
- **Parameters**:
  - `context: &AgentContext` - The execution context containing session info, available tools, and conversation history
- **Returns**: `Result<AgentResponse, AgentError>` - The agent's response with content, tool calls, and metadata
- **Errors**:
  - `AgentError::ExecutionFailed` - When agent execution fails
  - `AgentError::ContextError` - When context is invalid
  - `AgentError::ProviderTimeout` - When the provider times out
  - `AgentError::ToolTimeout` - When a tool execution times out
- **Example**:
  ```rust
  let context = AgentContext::new("session-123".to_string());
  match agent.execute(&context).await {
      Ok(response) => println!("Agent response: {}", response.content),
      Err(e) => eprintln!("Execution failed: {}", e),
  }
  ```

## Tool API

The Tool trait provides a unified interface for all executable tools in the framework.

### Interface Definition

```rust
#[async_trait]
pub trait Tool: Send + Sync {
    fn metadata(&self) -> ToolMetadata;
    async fn execute(&self, input: &str, ctx: &ToolContext) -> Result<ToolOutput, ToolError>;
    fn validate_input(&self, _input: &str) -> Result<(), ToolError> {
        Ok(())
    }
}
```

### Methods

#### metadata()

Returns metadata about the tool.

- **Signature**: `fn metadata(&self) -> ToolMetadata`
- **Returns**: A `ToolMetadata` struct with name, description, and input schema
- **Example**:
  ```rust
  let meta = tool.metadata();
  println!("Tool: {}", meta.name);
  println!("Description: {}", meta.description);
  ```

#### execute()

Executes the tool with given input.

- **Signature**: `async fn execute(&self, input: &str, ctx: &ToolContext) -> Result<ToolOutput, ToolError>`
- **Parameters**:
  - `input: &str` - The input string for the tool
  - `ctx: &ToolContext` - The execution context containing session ID and working directory
- **Returns**: `Result<ToolOutput, ToolError>` - The tool execution output (text, image, video, or audio)
- **Errors**:
  - `ToolError::ExecutionFailed` - When tool execution fails
  - `ToolError::InvalidInput` - When input validation fails
  - `ToolError::NotSupported` - When the operation is not supported
- **Example**:
  ```rust
  let ctx = ToolContext::new("session-123".to_string());
  match tool.execute("echo hello", &ctx).await {
      Ok(ToolOutput::Text(result)) => println!("Output: {}", result),
      Err(e) => eprintln!("Tool failed: {}", e),
  }
  ```

#### validate_input()

Validates the input before execution (optional, has default implementation).

- **Signature**: `fn validate_input(&self, _input: &str) -> Result<(), ToolError>`
- **Parameters**: `input: &str` - The input to validate
- **Returns**: `Result<(), ToolError>` - Ok if valid, Err if invalid
- **Example**:
  ```rust
  if let Err(e) = tool.validate_input("input_data") {
      eprintln!("Input validation failed: {}", e);
  }
  ```

## Memory API

The GlobalSharedMemory trait provides persistent storage and retrieval of information across agent executions.

### Interface Definition

```rust
#[async_trait]
pub trait GlobalSharedMemory: Send + Sync {
    async fn store(&self, namespace: &str, key: &str, value: &str) -> Result<(), MemoryError>;
    async fn retrieve(&self, namespace: &str, key: &str) -> Result<Option<String>, MemoryError>;
    async fn search(&self, namespace: &str, query: &str, limit: usize) -> Result<Vec<MemoryEntry>, MemoryError>;
    async fn delete(&self, namespace: &str, key: &str) -> Result<(), MemoryError>;
    async fn list_keys(&self, namespace: &str) -> Result<Vec<String>, MemoryError>;
}
```

### Methods

#### store()

Stores a value in the memory with a key in a specific namespace.

- **Signature**: `async fn store(&self, namespace: &str, key: &str, value: &str) -> Result<(), MemoryError>`
- **Parameters**:
  - `namespace: &str` - The namespace for organizing memory
  - `key: &str` - The key to store the value under
  - `value: &str` - The value to store
- **Returns**: `Result<(), MemoryError>` - Success or error
- **Errors**: `MemoryError::StoreFailed` - When storage operation fails
- **Example**:
  ```rust
  memory.store("user_data", "user_123", "John Doe").await?;
  ```

#### retrieve()

Retrieves a value from memory by key in a specific namespace.

- **Signature**: `async fn retrieve(&self, namespace: &str, key: &str) -> Result<Option<String>, MemoryError>`
- **Parameters**:
  - `namespace: &str` - The namespace to retrieve from
  - `key: &str` - The key to retrieve
- **Returns**: `Result<Option<String>, MemoryError>` - The value if found, None if not found
- **Errors**: `MemoryError::RetrieveFailed` - When retrieval fails
- **Example**:
  ```rust
  if let Ok(Some(value)) = memory.retrieve("user_data", "user_123").await {
      println!("Found: {}", value);
  }
  ```

#### search()

Searches for entries in memory using a query string.

- **Signature**: `async fn search(&self, namespace: &str, query: &str, limit: usize) -> Result<Vec<MemoryEntry>, MemoryError>`
- **Parameters**:
  - `namespace: &str` - The namespace to search in
  - `query: &str` - The search query
  - `limit: usize` - Maximum number of results to return
- **Returns**: `Result<Vec<MemoryEntry>, MemoryError>` - Vector of matching entries
- **Errors**: `MemoryError::SearchFailed` - When search operation fails
- **Example**:
  ```rust
  let results = memory.search("user_data", "John", 10).await?;
  for entry in results {
      println!("Key: {}, Value: {}", entry.key, entry.value);
  }
  ```

#### delete()

Deletes a value from memory by key in a specific namespace.

- **Signature**: `async fn delete(&self, namespace: &str, key: &str) -> Result<(), MemoryError>`
- **Parameters**:
  - `namespace: &str` - The namespace containing the key
  - `key: &str` - The key to delete
- **Returns**: `Result<(), MemoryError>` - Success or error
- **Example**:
  ```rust
  memory.delete("user_data", "user_123").await?;
  ```

#### list_keys()

Lists all keys in a namespace.

- **Signature**: `async fn list_keys(&self, namespace: &str) -> Result<Vec<String>, MemoryError>`
- **Parameters**: `namespace: &str` - The namespace to list keys from
- **Returns**: `Result<Vec<String>, MemoryError>` - Vector of all keys in the namespace
- **Example**:
  ```rust
  let keys = memory.list_keys("user_data").await?;
  for key in keys {
      println!("Key: {}", key);
  }
  ```

## Provider API

The LLMProvider trait defines the interface for language model providers.

### Interface Definition

```rust
#[async_trait]
pub trait LLMProvider: Send + Sync {
    fn name(&self) -> &str;
    fn capabilities(&self) -> ModelCapability;
    fn available_models(&self) -> Vec<String>;
    async fn complete(&self, prompt: &str, opts: CompleteOpts) -> Result<String, ProviderError>;
    async fn complete_with_media(&self, prompt: &str, media: &[MediaInput], opts: CompleteOpts) -> Result<String, ProviderError>;
    async fn complete_with_tools(&self, prompt: &str, tools: &[ToolCall], opts: CompleteOpts) -> Result<ToolCallResult, ProviderError>;
}
```

### Methods

#### name()

Returns the name of the provider.

- **Signature**: `fn name(&self) -> &str`
- **Returns**: A string slice with the provider name
- **Example**:
  ```rust
  println!("Provider: {}", provider.name());
  ```

#### capabilities()

Returns the model capabilities of the provider.

- **Signature**: `fn capabilities(&self) -> ModelCapability`
- **Returns**: A `ModelCapability` enum value (TextOnly, TextAndImages, TextAndVideo, or Multimodal)
- **Example**:
  ```rust
  match provider.capabilities() {
      ModelCapability::Multimodal => println!("Supports all media types"),
      ModelCapability::TextOnly => println!("Text only"),
      _ => println!("Limited media support"),
  }
  ```

#### available_models()

Returns a list of available models.

- **Signature**: `fn available_models(&self) -> Vec<String>`
- **Returns**: A vector of model name strings
- **Example**:
  ```rust
  let models = provider.available_models();
  for model in models {
      println!("Available model: {}", model);
  }
  ```

#### complete()

Performs text completion using the LLM.

- **Signature**: `async fn complete(&self, prompt: &str, opts: CompleteOpts) -> Result<String, ProviderError>`
- **Parameters**:
  - `prompt: &str` - The input prompt
  - `opts: CompleteOpts` - Completion options (model, temperature, max_tokens, etc.)
- **Returns**: `Result<String, ProviderError>` - The completion result
- **Errors**:
  - `ProviderError::RequestFailed` - When the request fails
  - `ProviderError::RateLimited` - When rate limited
  - `ProviderError::InvalidResponse` - When response is invalid
- **Example**:
  ```rust
  let opts = CompleteOpts {
      model: "model-123".to_string(),
      temperature: Some(0.7),
      max_tokens: Some(1000),
      ..Default::default()
  };
  let result = provider.complete("Hello, world!", opts).await?;
  ```

#### complete_with_media()

Performs completion with media input (images, videos, audio). Optional implementation, defaults to error.

- **Signature**: `async fn complete_with_media(&self, prompt: &str, media: &[MediaInput], opts: CompleteOpts) -> Result<String, ProviderError>`
- **Parameters**:
  - `prompt: &str` - The text prompt
  - `media: &[MediaInput]` - Array of media inputs
  - `opts: CompleteOpts` - Completion options
- **Returns**: `Result<String, ProviderError>` - The completion result
- **Example**:
  ```rust
  let media = vec![MediaInput::Image {
      url: "https://example.com/image.jpg".to_string(),
      mime_type: "image/jpeg".to_string(),
  }];
  let result = provider.complete_with_media("Describe this image", &media, opts).await?;
  ```

#### complete_with_tools()

Performs completion with tool calling capability. Optional implementation, defaults to error.

- **Signature**: `async fn complete_with_tools(&self, prompt: &str, tools: &[ToolCall], opts: CompleteOpts) -> Result<ToolCallResult, ProviderError>`
- **Parameters**:
  - `prompt: &str` - The text prompt
  - `tools: &[ToolCall]` - Available tools to call
  - `opts: CompleteOpts` - Completion options
- **Returns**: `Result<ToolCallResult, ProviderError>` - The tool call result
- **Example**:
  ```rust
  let result = provider.complete_with_tools("Use the bash tool", &tools, opts).await?;
  ```

## Channel API

The Channel trait defines the interface for message communication channels.

### Interface Definition

```rust
#[async_trait]
pub trait Channel: Send + Sync {
    fn name(&self) -> &str;
    async fn send(&self, msg: &Message) -> Result<(), ChannelError>;
    async fn receive(&self) -> Result<Option<Message>, ChannelError>;
    async fn connect(&self) -> Result<(), ChannelError>;
    async fn disconnect(&self) -> Result<(), ChannelError>;
}
```

### Methods

#### name()

Returns the name of the channel.

- **Signature**: `fn name(&self) -> &str`
- **Returns**: A string slice with the channel name
- **Example**:
  ```rust
  println!("Channel: {}", channel.name());
  ```

#### send()

Sends a message through the channel.

- **Signature**: `async fn send(&self, msg: &Message) -> Result<(), ChannelError>`
- **Parameters**: `msg: &Message` - The message to send
- **Returns**: `Result<(), ChannelError>` - Success or error
- **Errors**: `ChannelError::SendFailed` - When sending fails
- **Example**:
  ```rust
  let msg = Message {
      id: "123".to_string(),
      from: "user".to_string(),
      to: "agent".to_string(),
      content: "Hello".to_string(),
      timestamp: 0,
      metadata: Default::default(),
      attachments: Vec::new(),
  };
  channel.send(&msg).await?;
  ```

#### receive()

Receives a message from the channel (optional implementation).

- **Signature**: `async fn receive(&self) -> Result<Option<Message>, ChannelError>`
- **Returns**: `Result<Option<Message>, ChannelError>` - A message if available, None if no message
- **Errors**: `ChannelError::ReceiveFailed` - When receiving fails
- **Example**:
  ```rust
  if let Ok(Some(msg)) = channel.receive().await {
      println!("Received from {}: {}", msg.from, msg.content);
  }
  ```

#### connect()

Establishes a connection to the channel.

- **Signature**: `async fn connect(&self) -> Result<(), ChannelError>`
- **Returns**: `Result<(), ChannelError>` - Success or error
- **Example**:
  ```rust
  channel.connect().await?;
  ```

#### disconnect()

Closes the connection to the channel.

- **Signature**: `async fn disconnect(&self) -> Result<(), ChannelError>`
- **Returns**: `Result<(), ChannelError>` - Success or error
- **Example**:
  ```rust
  channel.disconnect().await?;
  ```

## Type Definitions

### AgentContext

The execution context passed to agents.

```rust
pub struct AgentContext {
    pub session_id: String,
    pub tools: Vec<Box<dyn Tool>>,
    pub history: Vec<HistoryEntry>,
}
```

- **Fields**:
  - `session_id: String` - Unique identifier for the execution session
  - `tools: Vec<Box<dyn Tool>>` - Available tools for the agent to use
  - `history: Vec<HistoryEntry>` - Conversation history
- **Methods**:
  - `new(session_id: String) -> Self` - Creates a new context

### HistoryEntry

Represents a single entry in the agent's conversation history.

```rust
pub struct HistoryEntry {
    pub role: String,
    pub content: String,
}
```

- **Fields**:
  - `role: String` - The role of the message sender (e.g., "user", "agent")
  - `content: String` - The message content

### ToolMetadata

Metadata about a tool.

```rust
pub struct ToolMetadata {
    pub name: String,
    pub description: String,
    pub input_schema: Value,
}
```

- **Fields**:
  - `name: String` - The tool name
  - `description: String` - What the tool does
  - `input_schema: Value` - JSON schema for input validation

### ToolContext

Execution context for tools.

```rust
pub struct ToolContext {
    pub session_id: String,
    pub working_dir: Option<String>,
}
```

- **Fields**:
  - `session_id: String` - The session identifier
  - `working_dir: Option<String>` - Optional working directory
- **Methods**:
  - `new(session_id: String) -> Self` - Creates a new context

### ToolOutput

The output produced by a tool.

```rust
pub enum ToolOutput {
    Text(String),
    Image { data: Vec<u8>, mime_type: String },
    Video { data: Vec<u8>, mime_type: String },
    Audio { data: Vec<u8>, mime_type: String },
}
```

- **Variants**:
  - `Text(String)` - Text output
  - `Image` - Image data with MIME type
  - `Video` - Video data with MIME type
  - `Audio` - Audio data with MIME type
- **Methods**:
  - `text(impl Into<String>) -> Self` - Helper to create text output

### Message

A message in the communication channel.

```rust
pub struct Message {
    pub id: String,
    pub from: String,
    pub to: String,
    pub content: String,
    pub timestamp: i64,
    pub metadata: HashMap<String, String>,
    pub attachments: Vec<MediaInput>,
}
```

- **Fields**:
  - `id: String` - Unique message identifier
  - `from: String` - Sender identifier
  - `to: String` - Recipient identifier
  - `content: String` - Message content
  - `timestamp: i64` - Unix timestamp
  - `metadata: HashMap<String, String>` - Additional metadata
  - `attachments: Vec<MediaInput>` - Optional attachments

### CompleteOpts

Options for LLM completion.

```rust
pub struct CompleteOpts {
    pub model: String,
    pub temperature: Option<f32>,
    pub max_tokens: Option<usize>,
    pub tools: Vec<ToolMetadata>,
    pub system_prompt: Option<String>,
}
```

- **Fields**:
  - `model: String` - The model to use
  - `temperature: Option<f32>` - Temperature for response randomness (0.0-1.0)
  - `max_tokens: Option<usize>` - Maximum tokens in response
  - `tools: Vec<ToolMetadata>` - Available tools
  - `system_prompt: Option<String>` - System prompt to use

### MemoryEntry

An entry stored in memory.

```rust
pub struct MemoryEntry {
    pub key: String,
    pub value: String,
    pub timestamp: i64,
    pub metadata: HashMap<String, String>,
}
```

- **Fields**:
  - `key: String` - The storage key
  - `value: String` - The stored value
  - `timestamp: i64` - When it was stored
  - `metadata: HashMap<String, String>` - Additional metadata

### ModelCapability

Enumeration of model capabilities.

```rust
pub enum ModelCapability {
    TextOnly,
    TextAndImages,
    TextAndVideo,
    Multimodal,
}
```

- **Variants**:
  - `TextOnly` - Text input/output only
  - `TextAndImages` - Text and image support
  - `TextAndVideo` - Text and video support
  - `Multimodal` - All media types supported

## Error Types

### ZeroError

The top-level error type aggregating all framework errors.

```rust
#[derive(Error, Debug)]
pub enum ZeroError {
    #[error("Agent error: {0}")]
    Agent(#[from] AgentError),
    #[error("Tool error: {0}")]
    Tool(#[from] ToolError),
    #[error("Memory error: {0}")]
    Memory(#[from] MemoryError),
    #[error("Provider error: {0}")]
    Provider(#[from] ProviderError),
    #[error("Channel error: {0}")]
    Channel(#[from] ChannelError),
    #[error("Config error: {0}")]
    Config(String),
    #[error("Not found: {0}")]
    NotFound(String),
}
```

### AgentError

Errors specific to agent execution.

```rust
#[derive(Error, Debug)]
pub enum AgentError {
    #[error("Execution failed: {0}")]
    ExecutionFailed(String),
    #[error("Context error: {0}")]
    ContextError(String),
    #[error("Max iterations exceeded: {0}")]
    MaxIterationsExceeded(usize),
    #[error("Provider timeout")]
    ProviderTimeout,
    #[error("Provider error: {0}")]
    ProviderError(String),
    #[error("Serialization error: {0}")]
    SerializationError(String),
    #[error("Tool timeout")]
    ToolTimeout,
}
```

- **Variants**:
  - `ExecutionFailed(String)` - Agent execution encountered an error
  - `ContextError(String)` - Context is invalid or missing required data
  - `MaxIterationsExceeded(usize)` - Agent exceeded maximum iteration count
  - `ProviderTimeout` - LLM provider request timed out
  - `ProviderError(String)` - LLM provider returned an error
  - `SerializationError(String)` - Failed to serialize/deserialize data
  - `ToolTimeout` - Tool execution timed out

### ToolError

Errors specific to tool execution.

```rust
#[derive(Error, Debug)]
pub enum ToolError {
    #[error("Execution failed: {0}")]
    ExecutionFailed(String),
    #[error("Invalid input: {0}")]
    InvalidInput(String),
    #[error("Not supported: {0}")]
    NotSupported(String),
}
```

- **Variants**:
  - `ExecutionFailed(String)` - Tool execution failed
  - `InvalidInput(String)` - Input validation failed
  - `NotSupported(String)` - Operation not supported by this tool

### MemoryError

Errors specific to memory operations.

```rust
#[derive(Error, Debug)]
pub enum MemoryError {
    #[error("Store failed: {0}")]
    StoreFailed(String),
    #[error("Retrieve failed: {0}")]
    RetrieveFailed(String),
    #[error("Search failed: {0}")]
    SearchFailed(String),
}
```

- **Variants**:
  - `StoreFailed(String)` - Failed to store value in memory
  - `RetrieveFailed(String)` - Failed to retrieve value from memory
  - `SearchFailed(String)` - Failed to search memory

### ProviderError

Errors specific to LLM provider operations.

```rust
#[derive(Error, Debug)]
pub enum ProviderError {
    #[error("Request failed: {0}")]
    RequestFailed(String),
    #[error("Rate limited: {0}")]
    RateLimited(String),
    #[error("Invalid response: {0}")]
    InvalidResponse(String),
    #[error("API error: {0}")]
    ApiError(String),
}
```

- **Variants**:
  - `RequestFailed(String)` - The LLM request failed
  - `RateLimited(String)` - Rate limit exceeded
  - `InvalidResponse(String)` - Provider returned invalid response
  - `ApiError(String)` - API-specific error

### ChannelError

Errors specific to channel communication.

```rust
#[derive(Error, Debug)]
pub enum ChannelError {
    #[error("Send failed: {0}")]
    SendFailed(String),
    #[error("Receive failed: {0}")]
    ReceiveFailed(String),
}
```

- **Variants**:
  - `SendFailed(String)` - Message sending failed
  - `ReceiveFailed(String)` - Message receiving failed

## Common Patterns

### Creating and Executing an Agent

```rust
use zero_core::{Agent, AgentContext};

// Implement the Agent trait
struct MyAgent {
    name: String,
    prompt: String,
}

#[async_trait]
impl Agent for MyAgent {
    fn name(&self) -> &str {
        &self.name
    }

    fn system_prompt(&self) -> &str {
        &self.prompt
    }

    async fn execute(&self, context: &AgentContext) -> Result<AgentResponse, AgentError> {
        // Implementation
    }
}

// Use it
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let agent = MyAgent {
        name: "Assistant".to_string(),
        prompt: "You are a helpful assistant.".to_string(),
    };

    let context = AgentContext::new("session-123".to_string());
    let response = agent.execute(&context).await?;

    println!("{}", response.content);
    Ok(())
}
```

### Implementing a Custom Tool

```rust
use zero_core::{Tool, ToolMetadata, ToolOutput, ToolContext, ToolError};

struct EchoTool;

#[async_trait]
impl Tool for EchoTool {
    fn metadata(&self) -> ToolMetadata {
        ToolMetadata {
            name: "echo".to_string(),
            description: "Echoes the input back".to_string(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "text": {"type": "string"}
                }
            }),
        }
    }

    async fn execute(&self, input: &str, _ctx: &ToolContext) -> Result<ToolOutput, ToolError> {
        Ok(ToolOutput::text(input.to_string()))
    }
}
```

### Using Memory for Persistence

```rust
use zero_core::GlobalSharedMemory;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let memory = /* obtain memory instance */;

    // Store information
    memory.store("users", "user-123", "John Doe").await?;

    // Retrieve information
    if let Some(name) = memory.retrieve("users", "user-123").await? {
        println!("User: {}", name);
    }

    // Search information
    let results = memory.search("users", "John", 10).await?;
    for entry in results {
        println!("{}: {}", entry.key, entry.value);
    }

    Ok(())
}
```

### Using Channels for Communication

```rust
use zero_core::Channel;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let channel = /* obtain channel instance */;

    // Connect to channel
    channel.connect().await?;

    // Send a message
    let msg = Message {
        id: "msg-1".to_string(),
        from: "user".to_string(),
        to: "agent".to_string(),
        content: "Hello, Agent!".to_string(),
        timestamp: /* current time */,
        metadata: Default::default(),
        attachments: Vec::new(),
    };
    channel.send(&msg).await?;

    // Receive messages
    while let Ok(Some(msg)) = channel.receive().await {
        println!("Received: {}", msg.content);
    }

    // Disconnect
    channel.disconnect().await?;

    Ok(())
}
```

### Error Handling Pattern

```rust
use zero_core::ZeroError;

#[tokio::main]
async fn main() {
    match execute_agent().await {
        Ok(result) => println!("Success: {}", result),
        Err(ZeroError::Agent(e)) => eprintln!("Agent error: {}", e),
        Err(ZeroError::Tool(e)) => eprintln!("Tool error: {}", e),
        Err(ZeroError::Memory(e)) => eprintln!("Memory error: {}", e),
        Err(ZeroError::Provider(e)) => eprintln!("Provider error: {}", e),
        Err(ZeroError::Channel(e)) => eprintln!("Channel error: {}", e),
        Err(e) => eprintln!("Other error: {}", e),
    }
}

async fn execute_agent() -> Result<String, ZeroError> {
    // Implementation
    Ok("Done".to_string())
}
```
