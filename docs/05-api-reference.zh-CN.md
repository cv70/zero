# API 参考

> **返回首页**: [README.zh-CN.md](../README.zh-CN.md)

Zero Core 框架的完整 API 文档。本指南涵盖所有 Trait 定义、方法、类型定义和错误处理。

## 目录

- [Agent API](#agent-api)
- [Tool API](#tool-api)
- [记忆 API](#记忆-api)
- [提供者 API](#提供者-api)
- [通道 API](#通道-api)
- [类型定义](#类型定义)
- [错误类型](#错误类型)
- [常见模式](#常见模式)

## Agent API

Agent Trait 定义了 Zero 框架中自主代理的核心接口。

### 接口定义

```rust
#[async_trait]
pub trait Agent: Send + Sync {
    fn name(&self) -> &str;
    fn system_prompt(&self) -> &str;
    fn description(&self) -> &str;
    async fn execute(&self, context: &AgentContext) -> Result<AgentResponse, AgentError>;
}
```

### 方法

#### name()

返回 Agent 的名称。

- **签名**: `fn name(&self) -> &str`
- **返回值**: 包含 Agent 名称的字符串切片
- **示例**:
  ```rust
  let agent_name = agent.name();
  println!("Agent: {}", agent_name);
  ```

#### system_prompt()

返回定义 Agent 行为和指令的系统提示词。

- **签名**: `fn system_prompt(&self) -> &str`
- **返回值**: 包含系统提示词的字符串切片
- **示例**:
  ```rust
  let prompt = agent.system_prompt();
  ```

#### description()

返回 Agent 用途和能力的描述。

- **签名**: `fn description(&self) -> &str`
- **返回值**: 描述 Agent 的字符串切片
- **示例**:
  ```rust
  let desc = agent.description();
  println!("关于: {}", desc);
  ```

#### execute()

使用给定的上下文执行 Agent。这是运行 Agent 逻辑的主方法。

- **签名**: `async fn execute(&self, context: &AgentContext) -> Result<AgentResponse, AgentError>`
- **参数**:
  - `context: &AgentContext` - 执行上下文，包含会话信息、可用工具和对话历史
- **返回值**: `Result<AgentResponse, AgentError>` - Agent 的响应，包含内容、工具调用和元数据
- **可能的错误**:
  - `AgentError::ExecutionFailed` - Agent 执行失败
  - `AgentError::ContextError` - 上下文无效
  - `AgentError::ProviderTimeout` - 模型提供者超时
  - `AgentError::ToolTimeout` - 工具执行超时
- **示例**:
  ```rust
  let context = AgentContext::new("session-123".to_string());
  match agent.execute(&context).await {
      Ok(response) => println!("Agent 响应: {}", response.content),
      Err(e) => eprintln!("执行失败: {}", e),
  }
  ```

## Tool API

Tool Trait 为框架中所有可执行的工具提供统一接口。

### 接口定义

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

### 方法

#### metadata()

返回关于工具的元数据。

- **签名**: `fn metadata(&self) -> ToolMetadata`
- **返回值**: 包含名称、描述和输入架构的 `ToolMetadata` 结构体
- **示例**:
  ```rust
  let meta = tool.metadata();
  println!("工具: {}", meta.name);
  println!("描述: {}", meta.description);
  ```

#### execute()

使用给定的输入执行工具。

- **签名**: `async fn execute(&self, input: &str, ctx: &ToolContext) -> Result<ToolOutput, ToolError>`
- **参数**:
  - `input: &str` - 工具的输入字符串
  - `ctx: &ToolContext` - 执行上下文，包含会话 ID 和工作目录
- **返回值**: `Result<ToolOutput, ToolError>` - 工具执行的输出（文本、图像、视频或音频）
- **可能的错误**:
  - `ToolError::ExecutionFailed` - 工具执行失败
  - `ToolError::InvalidInput` - 输入验证失败
  - `ToolError::NotSupported` - 不支持的操作
- **示例**:
  ```rust
  let ctx = ToolContext::new("session-123".to_string());
  match tool.execute("echo hello", &ctx).await {
      Ok(ToolOutput::Text(result)) => println!("输出: {}", result),
      Err(e) => eprintln!("工具失败: {}", e),
  }
  ```

#### validate_input()

在执行前验证输入（可选，有默认实现）。

- **签名**: `fn validate_input(&self, _input: &str) -> Result<(), ToolError>`
- **参数**: `input: &str` - 要验证的输入
- **返回值**: `Result<(), ToolError>` - 如果有效返回 Ok，无效返回 Err
- **示例**:
  ```rust
  if let Err(e) = tool.validate_input("input_data") {
      eprintln!("输入验证失败: {}", e);
  }
  ```

## 记忆 API

GlobalSharedMemory Trait 提供跨 Agent 执行的持久化存储和检索功能。

### 接口定义

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

### 方法

#### store()

在特定命名空间中使用键存储值。

- **签名**: `async fn store(&self, namespace: &str, key: &str, value: &str) -> Result<(), MemoryError>`
- **参数**:
  - `namespace: &str` - 用于组织记忆的命名空间
  - `key: &str` - 存储值的键
  - `value: &str` - 要存储的值
- **返回值**: `Result<(), MemoryError>` - 成功或错误
- **可能的错误**: `MemoryError::StoreFailed` - 存储操作失败
- **示例**:
  ```rust
  memory.store("user_data", "user_123", "John Doe").await?;
  ```

#### retrieve()

从特定命名空间中按键检索值。

- **签名**: `async fn retrieve(&self, namespace: &str, key: &str) -> Result<Option<String>, MemoryError>`
- **参数**:
  - `namespace: &str` - 检索的命名空间
  - `key: &str` - 要检索的键
- **返回值**: `Result<Option<String>, MemoryError>` - 如果找到返回值，未找到返回 None
- **可能的错误**: `MemoryError::RetrieveFailed` - 检索失败
- **示例**:
  ```rust
  if let Ok(Some(value)) = memory.retrieve("user_data", "user_123").await {
      println!("找到: {}", value);
  }
  ```

#### search()

使用查询字符串搜索记忆中的条目。

- **签名**: `async fn search(&self, namespace: &str, query: &str, limit: usize) -> Result<Vec<MemoryEntry>, MemoryError>`
- **参数**:
  - `namespace: &str` - 要搜索的命名空间
  - `query: &str` - 搜索查询
  - `limit: usize` - 返回结果的最大数量
- **返回值**: `Result<Vec<MemoryEntry>, MemoryError>` - 匹配条目的向量
- **可能的错误**: `MemoryError::SearchFailed` - 搜索操作失败
- **示例**:
  ```rust
  let results = memory.search("user_data", "John", 10).await?;
  for entry in results {
      println!("键: {}, 值: {}", entry.key, entry.value);
  }
  ```

#### delete()

从特定命名空间中按键删除值。

- **签名**: `async fn delete(&self, namespace: &str, key: &str) -> Result<(), MemoryError>`
- **参数**:
  - `namespace: &str` - 包含键的命名空间
  - `key: &str` - 要删除的键
- **返回值**: `Result<(), MemoryError>` - 成功或错误
- **示例**:
  ```rust
  memory.delete("user_data", "user_123").await?;
  ```

#### list_keys()

列出命名空间中的所有键。

- **签名**: `async fn list_keys(&self, namespace: &str) -> Result<Vec<String>, MemoryError>`
- **参数**: `namespace: &str` - 要列出键的命名空间
- **返回值**: `Result<Vec<String>, MemoryError>` - 命名空间中所有键的向量
- **示例**:
  ```rust
  let keys = memory.list_keys("user_data").await?;
  for key in keys {
      println!("键: {}", key);
  }
  ```

## 提供者 API

LLMProvider Trait 定义语言模型提供者的接口。

### 接口定义

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

### 方法

#### name()

返回提供者的名称。

- **签名**: `fn name(&self) -> &str`
- **返回值**: 包含提供者名称的字符串切片
- **示例**:
  ```rust
  println!("提供者: {}", provider.name());
  ```

#### capabilities()

返回提供者的模型能力。

- **签名**: `fn capabilities(&self) -> ModelCapability`
- **返回值**: `ModelCapability` 枚举值（TextOnly、TextAndImages、TextAndVideo 或 Multimodal）
- **示例**:
  ```rust
  match provider.capabilities() {
      ModelCapability::Multimodal => println!("支持所有媒体类型"),
      ModelCapability::TextOnly => println!("仅文本"),
      _ => println!("有限的媒体支持"),
  }
  ```

#### available_models()

返回可用模型列表。

- **签名**: `fn available_models(&self) -> Vec<String>`
- **返回值**: 模型名称字符串的向量
- **示例**:
  ```rust
  let models = provider.available_models();
  for model in models {
      println!("可用模型: {}", model);
  }
  ```

#### complete()

使用 LLM 执行文本补全。

- **签名**: `async fn complete(&self, prompt: &str, opts: CompleteOpts) -> Result<String, ProviderError>`
- **参数**:
  - `prompt: &str` - 输入提示词
  - `opts: CompleteOpts` - 补全选项（模型、温度、最大标记数等）
- **返回值**: `Result<String, ProviderError>` - 补全结果
- **可能的错误**:
  - `ProviderError::RequestFailed` - 请求失败
  - `ProviderError::RateLimited` - 受限制
  - `ProviderError::InvalidResponse` - 响应无效
- **示例**:
  ```rust
  let opts = CompleteOpts {
      model: "model-123".to_string(),
      temperature: Some(0.7),
      max_tokens: Some(1000),
      ..Default::default()
  };
  let result = provider.complete("你好，世界！", opts).await?;
  ```

#### complete_with_media()

执行带有媒体输入的补全（图像、视频、音频）。可选实现，默认返回错误。

- **签名**: `async fn complete_with_media(&self, prompt: &str, media: &[MediaInput], opts: CompleteOpts) -> Result<String, ProviderError>`
- **参数**:
  - `prompt: &str` - 文本提示词
  - `media: &[MediaInput]` - 媒体输入数组
  - `opts: CompleteOpts` - 补全选项
- **返回值**: `Result<String, ProviderError>` - 补全结果
- **示例**:
  ```rust
  let media = vec![MediaInput::Image {
      url: "https://example.com/image.jpg".to_string(),
      mime_type: "image/jpeg".to_string(),
  }];
  let result = provider.complete_with_media("描述这张图片", &media, opts).await?;
  ```

#### complete_with_tools()

执行带有工具调用能力的补全。可选实现，默认返回错误。

- **签名**: `async fn complete_with_tools(&self, prompt: &str, tools: &[ToolCall], opts: CompleteOpts) -> Result<ToolCallResult, ProviderError>`
- **参数**:
  - `prompt: &str` - 文本提示词
  - `tools: &[ToolCall]` - 可调用的工具
  - `opts: CompleteOpts` - 补全选项
- **返回值**: `Result<ToolCallResult, ProviderError>` - 工具调用结果
- **示例**:
  ```rust
  let result = provider.complete_with_tools("使用 bash 工具", &tools, opts).await?;
  ```

## 通道 API

Channel Trait 定义消息通信通道的接口。

### 接口定义

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

### 方法

#### name()

返回通道的名称。

- **签名**: `fn name(&self) -> &str`
- **返回值**: 包含通道名称的字符串切片
- **示例**:
  ```rust
  println!("通道: {}", channel.name());
  ```

#### send()

通过通道发送消息。

- **签名**: `async fn send(&self, msg: &Message) -> Result<(), ChannelError>`
- **参数**: `msg: &Message` - 要发送的消息
- **返回值**: `Result<(), ChannelError>` - 成功或错误
- **可能的错误**: `ChannelError::SendFailed` - 发送失败
- **示例**:
  ```rust
  let msg = Message {
      id: "123".to_string(),
      from: "user".to_string(),
      to: "agent".to_string(),
      content: "你好".to_string(),
      timestamp: 0,
      metadata: Default::default(),
      attachments: Vec::new(),
  };
  channel.send(&msg).await?;
  ```

#### receive()

从通道接收消息（可选实现）。

- **签名**: `async fn receive(&self) -> Result<Option<Message>, ChannelError>`
- **返回值**: `Result<Option<Message>, ChannelError>` - 如果有可用消息返回，否则返回 None
- **可能的错误**: `ChannelError::ReceiveFailed` - 接收失败
- **示例**:
  ```rust
  if let Ok(Some(msg)) = channel.receive().await {
      println!("从 {} 收到: {}", msg.from, msg.content);
  }
  ```

#### connect()

建立与通道的连接。

- **签名**: `async fn connect(&self) -> Result<(), ChannelError>`
- **返回值**: `Result<(), ChannelError>` - 成功或错误
- **示例**:
  ```rust
  channel.connect().await?;
  ```

#### disconnect()

关闭与通道的连接。

- **签名**: `async fn disconnect(&self) -> Result<(), ChannelError>`
- **返回值**: `Result<(), ChannelError>` - 成功或错误
- **示例**:
  ```rust
  channel.disconnect().await?;
  ```

## 类型定义

### AgentContext

传递给 Agent 的执行上下文。

```rust
pub struct AgentContext {
    pub session_id: String,
    pub tools: Vec<Box<dyn Tool>>,
    pub history: Vec<HistoryEntry>,
}
```

- **字段**:
  - `session_id: String` - 执行会话的唯一标识符
  - `tools: Vec<Box<dyn Tool>>` - Agent 可用的工具
  - `history: Vec<HistoryEntry>` - 对话历史
- **方法**:
  - `new(session_id: String) -> Self` - 创建新上下文

### HistoryEntry

Agent 对话历史中的单个条目。

```rust
pub struct HistoryEntry {
    pub role: String,
    pub content: String,
}
```

- **字段**:
  - `role: String` - 消息发送者的角色（例如 "user"、"agent"）
  - `content: String` - 消息内容

### ToolMetadata

关于工具的元数据。

```rust
pub struct ToolMetadata {
    pub name: String,
    pub description: String,
    pub input_schema: Value,
}
```

- **字段**:
  - `name: String` - 工具名称
  - `description: String` - 工具的功能描述
  - `input_schema: Value` - 用于输入验证的 JSON 架构

### ToolContext

工具的执行上下文。

```rust
pub struct ToolContext {
    pub session_id: String,
    pub working_dir: Option<String>,
}
```

- **字段**:
  - `session_id: String` - 会话标识符
  - `working_dir: Option<String>` - 可选的工作目录
- **方法**:
  - `new(session_id: String) -> Self` - 创建新上下文

### ToolOutput

工具生成的输出。

```rust
pub enum ToolOutput {
    Text(String),
    Image { data: Vec<u8>, mime_type: String },
    Video { data: Vec<u8>, mime_type: String },
    Audio { data: Vec<u8>, mime_type: String },
}
```

- **变体**:
  - `Text(String)` - 文本输出
  - `Image` - 带 MIME 类型的图像数据
  - `Video` - 带 MIME 类型的视频数据
  - `Audio` - 带 MIME 类型的音频数据
- **方法**:
  - `text(impl Into<String>) -> Self` - 创建文本输出的辅助函数

### Message

通信通道中的消息。

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

- **字段**:
  - `id: String` - 唯一的消息标识符
  - `from: String` - 发送者标识符
  - `to: String` - 接收者标识符
  - `content: String` - 消息内容
  - `timestamp: i64` - Unix 时间戳
  - `metadata: HashMap<String, String>` - 附加元数据
  - `attachments: Vec<MediaInput>` - 可选的附件

### CompleteOpts

LLM 补全的选项。

```rust
pub struct CompleteOpts {
    pub model: String,
    pub temperature: Option<f32>,
    pub max_tokens: Option<usize>,
    pub tools: Vec<ToolMetadata>,
    pub system_prompt: Option<String>,
}
```

- **字段**:
  - `model: String` - 要使用的模型
  - `temperature: Option<f32>` - 响应随机性的温度（0.0-1.0）
  - `max_tokens: Option<usize>` - 响应中的最大标记数
  - `tools: Vec<ToolMetadata>` - 可用的工具
  - `system_prompt: Option<String>` - 要使用的系统提示词

### MemoryEntry

存储在记忆中的条目。

```rust
pub struct MemoryEntry {
    pub key: String,
    pub value: String,
    pub timestamp: i64,
    pub metadata: HashMap<String, String>,
}
```

- **字段**:
  - `key: String` - 存储键
  - `value: String` - 存储的值
  - `timestamp: i64` - 存储时间
  - `metadata: HashMap<String, String>` - 附加元数据

### ModelCapability

模型能力的枚举。

```rust
pub enum ModelCapability {
    TextOnly,
    TextAndImages,
    TextAndVideo,
    Multimodal,
}
```

- **变体**:
  - `TextOnly` - 仅支持文本输入/输出
  - `TextAndImages` - 支持文本和图像
  - `TextAndVideo` - 支持文本和视频
  - `Multimodal` - 支持所有媒体类型

## 错误类型

### ZeroError

聚合所有框架错误的顶级错误类型。

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

特定于 Agent 执行的错误。

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

- **变体**:
  - `ExecutionFailed(String)` - Agent 执行遇到错误
  - `ContextError(String)` - 上下文无效或缺少必需数据
  - `MaxIterationsExceeded(usize)` - Agent 超过最大迭代次数
  - `ProviderTimeout` - LLM 提供者请求超时
  - `ProviderError(String)` - LLM 提供者返回错误
  - `SerializationError(String)` - 序列化/反序列化失败
  - `ToolTimeout` - 工具执行超时

### ToolError

特定于工具执行的错误。

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

- **变体**:
  - `ExecutionFailed(String)` - 工具执行失败
  - `InvalidInput(String)` - 输入验证失败
  - `NotSupported(String)` - 此工具不支持的操作

### MemoryError

特定于记忆操作的错误。

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

- **变体**:
  - `StoreFailed(String)` - 在记忆中存储值失败
  - `RetrieveFailed(String)` - 从记忆中检索值失败
  - `SearchFailed(String)` - 搜索记忆失败

### ProviderError

特定于 LLM 提供者操作的错误。

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

- **变体**:
  - `RequestFailed(String)` - LLM 请求失败
  - `RateLimited(String)` - 超过速率限制
  - `InvalidResponse(String)` - 提供者返回的响应无效
  - `ApiError(String)` - API 特定错误

### ChannelError

特定于通道通信的错误。

```rust
#[derive(Error, Debug)]
pub enum ChannelError {
    #[error("Send failed: {0}")]
    SendFailed(String),
    #[error("Receive failed: {0}")]
    ReceiveFailed(String),
}
```

- **变体**:
  - `SendFailed(String)` - 消息发送失败
  - `ReceiveFailed(String)` - 消息接收失败

## 常见模式

### 创建和执行 Agent

```rust
use zero_core::{Agent, AgentContext};

// 实现 Agent Trait
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
        // 实现
    }
}

// 使用它
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let agent = MyAgent {
        name: "助手".to_string(),
        prompt: "你是一个有帮助的助手。".to_string(),
    };

    let context = AgentContext::new("session-123".to_string());
    let response = agent.execute(&context).await?;

    println!("{}", response.content);
    Ok(())
}
```

### 实现自定义工具

```rust
use zero_core::{Tool, ToolMetadata, ToolOutput, ToolContext, ToolError};

struct EchoTool;

#[async_trait]
impl Tool for EchoTool {
    fn metadata(&self) -> ToolMetadata {
        ToolMetadata {
            name: "echo".to_string(),
            description: "回显输入".to_string(),
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

### 使用记忆进行持久化

```rust
use zero_core::GlobalSharedMemory;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let memory = /* 获取记忆实例 */;

    // 存储信息
    memory.store("users", "user-123", "John Doe").await?;

    // 检索信息
    if let Some(name) = memory.retrieve("users", "user-123").await? {
        println!("用户: {}", name);
    }

    // 搜索信息
    let results = memory.search("users", "John", 10).await?;
    for entry in results {
        println!("{}: {}", entry.key, entry.value);
    }

    Ok(())
}
```

### 使用通道进行通信

```rust
use zero_core::Channel;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let channel = /* 获取通道实例 */;

    // 连接到通道
    channel.connect().await?;

    // 发送消息
    let msg = Message {
        id: "msg-1".to_string(),
        from: "user".to_string(),
        to: "agent".to_string(),
        content: "你好，Agent！".to_string(),
        timestamp: /* 当前时间 */,
        metadata: Default::default(),
        attachments: Vec::new(),
    };
    channel.send(&msg).await?;

    // 接收消息
    while let Ok(Some(msg)) = channel.receive().await {
        println!("收到: {}", msg.content);
    }

    // 断开连接
    channel.disconnect().await?;

    Ok(())
}
```

### 错误处理模式

```rust
use zero_core::ZeroError;

#[tokio::main]
async fn main() {
    match execute_agent().await {
        Ok(result) => println!("成功: {}", result),
        Err(ZeroError::Agent(e)) => eprintln!("Agent 错误: {}", e),
        Err(ZeroError::Tool(e)) => eprintln!("工具错误: {}", e),
        Err(ZeroError::Memory(e)) => eprintln!("记忆错误: {}", e),
        Err(ZeroError::Provider(e)) => eprintln!("提供者错误: {}", e),
        Err(ZeroError::Channel(e)) => eprintln!("通道错误: {}", e),
        Err(e) => eprintln!("其他错误: {}", e),
    }
}

async fn execute_agent() -> Result<String, ZeroError> {
    // 实现
    Ok("完成".to_string())
}
```
