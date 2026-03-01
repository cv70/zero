# Trait 驱动架构

> **返回首页**: [README.zh-CN.md](../README.zh-CN.md)

Zero 的整个系统基于 **Trait 优先设计** - 一种理念，所有核心能力都通过 Trait 定义，而不是具体实现。这使得系统具有最大的灵活性、可测试性和可扩展性。

## 目录

- [快速概览](#快速概览)
- [5 个核心 Trait](#5-个核心-trait)
  - [Agent Trait](#agent-trait)
  - [Tool Trait](#tool-trait)
  - [GlobalSharedMemory Trait](#globalsharedmemory-trait)
  - [LLMProvider Trait](#llmprovider-trait)
  - [Channel Trait](#channel-trait)
- [执行流程：Agent 循环](#执行流程agent-循环)
- [数据流架构](#数据流架构)
- [分层架构](#分层架构)
- [扩展机制](#扩展机制)
- [常见设计模式](#常见设计模式)
- [性能考虑](#性能考虑)

---

## 快速概览

以下是 5 个核心 Trait 的快速对比表：

| Trait | 目的 | 关键方法 | 责任 |
|-------|------|--------|------|
| **Agent** | Agent 工厂和执行引擎 | `execute()`, `name()`, `system_prompt()` | 协调 Agent 循环和其他组件的交互 |
| **Tool** | 统一工具抽象 | `execute()`, `metadata()`, `validate_input()` | 提供具体功能（bash、文件操作等） |
| **GlobalSharedMemory** | 跨 Agent 的共享记忆 | `store()`, `retrieve()`, `search()`, `delete()` | 管理跨 Agent 的持久化知识 |
| **LLMProvider** | LLM 提供者抽象 | `complete()`, `complete_with_tools()`, `capabilities()` | 接口与语言模型交互（Anthropic、OpenAI 等） |
| **Channel** | 消息通道抽象 | `send()`, `receive()`, `connect()`, `disconnect()` | 处理通信（CLI、Slack、Discord 等） |

---

## 5 个核心 Trait

### Agent Trait

#### 目的

**Agent Trait** 是 Zero 系统的核心。它定义了可以推理、规划和执行任务的 Agent 的接口。Agent 充当协调器，与提供者、工具、记忆和通道协调。

#### 定义和接口

```rust
#[async_trait]
pub trait Agent: Send + Sync {
    /// Agent 名称
    fn name(&self) -> &str {
        ""
    }

    /// Agent 系统提示词
    fn system_prompt(&self) -> &str {
        ""
    }

    /// Agent 描述
    fn description(&self) -> &str {
        ""
    }

    /// 执行 Agent（异步）
    async fn execute(&self, context: &AgentContext) -> Result<AgentResponse, AgentError>;
}
```

**关键方法：**
- `name()` - 返回 Agent 的标识符
- `system_prompt()` - 返回指导 Agent 行为的系统提示词
- `description()` - 返回人类可读的描述
- `execute()` - 运行 Agent 循环的主执行方法

#### 关键概念

Agent 不直接调用 LLM 或执行工具。相反，它使用：
- **LoopProvider**: 一个用于在循环内进行 LLM 调用的提供者 Trait
- **ToolDispatcher**: 用于执行工具调用的调度器
- **HookManager**: 可选的钩子系统，用于可观测性

#### Agent 内的执行流程

```
Agent.execute() 被调用
    ↓
初始化消息历史
    ↓
循环直到完成：
    ├─ 触发：AgentHook::on_agent_run()
    ├─ 调用：LoopProvider.complete()
    ├─ 解析响应（检查 stop_reason）
    ├─ 如果 "tool_use"：
    │   ├─ 触发：ToolHook::on_tool_execute()
    │   ├─ 通过 ToolDispatcher 执行工具
    │   ├─ 触发：ToolHook::on_tool_execute_done()
    │   └─ 将工具结果附加到消息
    ├─ 如果 "end_turn"：
    │   └─ 中断循环
    ├─ 触发：AgentHook::on_agent_run_done()
    └─ 继续或返回
    ↓
返回 AgentResponse
```

#### 实现示例：基础 Agent

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
        "一个基础的自定义 Agent"
    }

    async fn execute(&self, context: &AgentContext) -> Result<AgentResponse, AgentError> {
        // 实际的循环通常由 AgentLoop 处理
        // 这里是你实现自定义逻辑的地方
        let response = AgentResponse {
            content: "Hello, world!".to_string(),
            tool_calls: vec![],
            metadata: Default::default(),
        };
        Ok(response)
    }
}
```

#### 扩展点

- **自定义系统提示词**: 通过改变系统提示词来修改行为
- **Agent 类型**: 创建专门的 Agent（分析师、开发者、研究员等）
- **状态管理**: Agent 可以在调用之间维护内部状态
- **工具选择**: 不同的 Agent 可以拥有不同的工具集

#### 常见模式

1. **Agent 工厂模式**: 基于配置动态创建 Agent
2. **Agent 专门化**: 不同领域的不同 Agent
3. **Agent 委托**: Agent 为复杂任务创建子 Agent
4. **Agent 组合**: 多个 Agent 在不同方面协同工作

---

### Tool Trait

#### 目的

**Tool Trait** 是所有外部功能的抽象。工具是 Agent 与外部世界交互的方式 - 执行命令、读取文件、调用 API 等。

#### 定义和接口

```rust
#[async_trait]
pub trait Tool: Send + Sync {
    /// 工具元数据
    fn metadata(&self) -> ToolMetadata;

    /// 执行工具
    async fn execute(&self, input: &str, ctx: &ToolContext) -> Result<ToolOutput, ToolError>;

    /// 可选：验证输入
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

**关键方法：**
- `metadata()` - 返回工具信息（名称、描述、JSON 模式）
- `execute()` - 使用给定输入运行工具的主方法
- `validate_input()` - 执行前的可选验证

#### 工具上下文

```rust
pub struct ToolContext {
    pub session_id: String,        // 唯一的会话 ID
    pub working_dir: Option<String>, // 工具的工作目录
}
```

#### 实现示例：自定义工具

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
            description: "执行基本算术计算".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "expression": {
                        "type": "string",
                        "description": "待计算的数学表达式"
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
        // 解析并计算表达式
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

#### 工具注册表

工具由 **ToolRegistry** 管理，按名称存储和检索工具：

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

**使用方式：**
```rust
let registry = Arc::new(ToolRegistry::new());
registry.register(Box::new(CalculatorTool)).await;
```

#### 工具执行流程

```
Agent 请求工具调用
    ↓
触发：ToolHook::on_tool_validate()
    ↓
调用：Tool::validate_input()
    ├─ 有效 → 继续
    └─ 无效 → 返回错误
    ↓
触发：ToolHook::on_tool_execute()
    ↓
调用：Tool::execute(input, context)
    ├─ 成功 → 捕获输出
    └─ 错误 → 捕获错误
    ↓
触发：ToolHook::on_tool_execute_done()
    ↓
返回 ToolOutput 或 ToolError
```

#### 扩展点

- **自定义工具**: 通过实现 `Tool` Trait 实现任何工具
- **工具中间件**: 钩子允许添加日志、缓存、限流等
- **工具验证**: 执行前的自定义验证逻辑
- **工具输出处理**: 不同的输出类型（文本、图像、视频、音频）

#### 常见模式

1. **工具组合**: 将多个简单工具组合成复杂工作流
2. **工具链**: 一个工具的输出作为另一个工具的输入
3. **条件工具选择**: 基于任务类型选择不同工具
4. **工具回退**: 尝试一个工具，如果失败则回退到另一个

---

### GlobalSharedMemory Trait

#### 目的

**GlobalSharedMemory Trait** 提供一个在系统中所有 Agent 间共享的持久化知识存储。它是 Agent 的"长期记忆"，用于存储事实、决策和学习的模式。

#### 定义和接口

```rust
#[async_trait]
pub trait GlobalSharedMemory: Send + Sync {
    /// 存储记忆条目
    async fn store(
        &self,
        namespace: &str,
        key: &str,
        value: &str,
    ) -> Result<(), MemoryError>;

    /// 检索记忆条目
    async fn retrieve(
        &self,
        namespace: &str,
        key: &str,
    ) -> Result<Option<String>, MemoryError>;

    /// 搜索记忆条目
    async fn search(
        &self,
        namespace: &str,
        query: &str,
        limit: usize,
    ) -> Result<Vec<MemoryEntry>, MemoryError>;

    /// 删除记忆条目
    async fn delete(
        &self,
        namespace: &str,
        key: &str,
    ) -> Result<(), MemoryError>;

    /// 列出命名空间中的所有键
    async fn list_keys(&self, namespace: &str) -> Result<Vec<String>, MemoryError>;
}

pub struct MemoryEntry {
    pub key: String,
    pub value: String,
    pub timestamp: i64,
    pub metadata: HashMap<String, String>,
}
```

**关键方法：**
- `store()` - 将键值对持久化到命名空间
- `retrieve()` - 按键获取值
- `search()` - 全文搜索条目
- `delete()` - 删除条目
- `list_keys()` - 列出命名空间中的所有键

#### 分层记忆系统

Zero 支持 **分层记忆架构**：

```
┌─────────────────────────────────┐
│  会话记忆（临时）                │  ← 短期、内存中
├─────────────────────────────────┤
│  对话记忆（当前）                │  ← 当前任务上下文
├─────────────────────────────────┤
│  全局共享记忆（持久化）          │  ← 跨 Agent 知识
└─────────────────────────────────┘
```

**命名空间组织：**
- `agent/{agent_id}/` - Agent 特定的记忆
- `task/{task_id}/` - 任务特定的记忆
- `domain/{domain}/` - 特定领域的事实
- `user/{user_id}/` - 用户偏好和历史

#### 实现示例：文件系统后端

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
        // 实现全文搜索
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

#### 扩展点

- **数据库后端**: 使用 SQLite、PostgreSQL 等实现
- **向量搜索**: 使用嵌入进行语义搜索
- **压缩**: 实现记忆压缩策略
- **复制**: 在多个节点间分布记忆
- **缓存**: 为性能添加缓存层

---

### LLMProvider Trait

#### 目的

**LLMProvider Trait** 抽象语言模型交互。它允许 Zero 与任何 LLM 提供者配合使用，而无需改变核心循环逻辑。

#### 定义和接口

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
    /// 提供者名称
    fn name(&self) -> &str;

    /// 支持的模型能力
    fn capabilities(&self) -> ModelCapability;

    /// 可用模型列表
    fn available_models(&self) -> Vec<String>;

    /// 纯文本补全
    async fn complete(
        &self,
        prompt: &str,
        opts: CompleteOpts,
    ) -> Result<String, ProviderError>;

    /// 多模态补全（可选）
    async fn complete_with_media(
        &self,
        prompt: &str,
        media: &[MediaInput],
        opts: CompleteOpts,
    ) -> Result<String, ProviderError>;

    /// 工具调用补全（可选）
    async fn complete_with_tools(
        &self,
        prompt: &str,
        tools: &[ToolCall],
        opts: CompleteOpts,
    ) -> Result<ToolCallResult, ProviderError>;
}
```

**关键方法：**
- `name()` - 提供者标识符（例如"anthropic"、"openai"）
- `capabilities()` - 返回该提供者可以做什么
- `complete()` - 基础文本补全
- `complete_with_media()` - 带有图像、视频等的多模态补全
- `complete_with_tools()` - 工具调用（结构化输出）

#### 实现示例：Anthropic 提供者

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
        // 调用 Anthropic API
        // 解析响应并返回文本
        Ok("Response from Claude".to_string())
    }

    async fn complete_with_media(
        &self,
        prompt: &str,
        media: &[MediaInput],
        opts: CompleteOpts,
    ) -> Result<String, ProviderError> {
        // Claude 支持多模态 - 实现媒体处理
        Ok("Multimodal response".to_string())
    }

    async fn complete_with_tools(
        &self,
        prompt: &str,
        tools: &[ToolCall],
        opts: CompleteOpts,
    ) -> Result<ToolCallResult, ProviderError> {
        // Claude 支持工具使用 - 实现工具调用
        Ok(ToolCallResult {
            id: "call_123".to_string(),
            result: Ok("Tool result".to_string()),
        })
    }
}
```

#### 多模型支持

Zero 支持多个提供者和模型：

```
┌────────────────────────────────────┐
│     提供者路由器                   │
├────────────────────────────────────┤
│ ├─ Anthropic（Claude 3 系列）     │
│ ├─ OpenAI（GPT-4、GPT-3.5）       │
│ ├─ Ollama（本地模型）             │
│ └─ 自定义提供者                   │
└────────────────────────────────────┘
```

#### 能力路由

Agent 可以请求特定的能力，系统会路由到适当的提供者：

```rust
// Agent 需要多模态能力
if needs_image_analysis {
    // 路由到具有 TextAndImages 或 Multimodal 能力的提供者
    let provider = router.select_by_capability(ModelCapability::Multimodal)?;
}

// Agent 需要工具使用
if needs_tool_use {
    // 使用支持工具调用的提供者
    let provider = router.select_by_feature("tool_calling")?;
}
```

#### 扩展点

- **新提供者**: 为新的 LLM 提供者实现 Trait
- **缓存**: 缓存重复提示的响应
- **限流**: 控制 API 调用速率
- **回退链**: 如果一个提供者失败，尝试多个提供者
- **成本优化**: 在可能的情况下路由到更便宜的模型

---

### Channel Trait

#### 目的

**Channel Trait** 抽象通信通道。它允许 Agent 通过各种平台（CLI、Slack、Discord、Email 等）发送和接收消息，而无需改变核心逻辑。

#### 定义和接口

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
    /// 通道名称
    fn name(&self) -> &str;

    /// 发送消息
    async fn send(&self, msg: &Message) -> Result<(), ChannelError>;

    /// 接收消息（可选）
    async fn receive(&self) -> Result<Option<Message>, ChannelError>;

    /// 连接到通道
    async fn connect(&self) -> Result<(), ChannelError>;

    /// 断开连接
    async fn disconnect(&self) -> Result<(), ChannelError>;
}
```

**关键方法：**
- `name()` - 通道标识符
- `send()` - 向通道发送消息
- `receive()` - 等待并接收消息
- `connect()` - 建立连接
- `disconnect()` - 关闭连接

#### 多通道支持

Zero 支持多个同时进行的通道：

```
┌─────────────────────────────────┐
│     通道注册表                  │
├─────────────────────────────────┤
│ ├─ CLI（stdio）                │
│ ├─ Slack                       │
│ ├─ Discord                     │
│ ├─ 邮件                        │
│ ├─ Telegram                    │
│ ├─ Matrix                      │
│ └─ 自定义通道                  │
└─────────────────────────────────┘
```

#### 消息格式标准化

所有通道使用相同的消息结构：

```rust
let msg = Message {
    id: uuid::Uuid::new_v4().to_string(),
    from: "agent_1".to_string(),
    to: "user_1".to_string(),
    content: "任务完成！".to_string(),
    timestamp: chrono::Utc::now().timestamp(),
    metadata: hashmap! {
        "priority" => "high".to_string(),
    },
    attachments: vec![],
};
```

#### 实现示例：Discord 通道

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
        // 发送到 Discord Webhook
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
        // Discord Webhook 是单向的，所以不支持接收
        Err(ChannelError::NotSupported(
            "Discord webhooks don't support receive".into()
        ))
    }

    async fn connect(&self) -> Result<(), ChannelError> {
        // 验证 Webhook URL
        Ok(())
    }

    async fn disconnect(&self) -> Result<(), ChannelError> {
        Ok(())
    }
}
```

#### 扩展点

- **新通道**: 为任何通信平台实现
- **消息格式化**: 每个通道的自定义格式化
- **限流**: 控制消息频率
- **消息持久化**: 记录所有消息
- **加密**: 添加端到端加密

---

## 执行流程：Agent 循环

Agent 循环是 Zero 的核心。以下是详细的执行流程：

```
┌─────────────────────────────────────────────────────────┐
│                    Agent.execute()                      │
└─────────────────────────────────────────────────────────┘
                           ↓
┌─────────────────────────────────────────────────────────┐
│          初始化：messages = [user_input]                │
└─────────────────────────────────────────────────────────┘
                           ↓
                    ┌──────────────┐
                    │ 循环开始     │
                    └──────────────┘
                           ↓
      ┌────────────────────────────────────────┐
      │ 检查：iteration < max_iterations       │
      └────────────────────────────────────────┘
                           ↓
      ┌────────────────────────────────────────┐
      │ 触发：AgentHook::on_agent_run()       │
      └────────────────────────────────────────┘
                           ↓
      ┌────────────────────────────────────────┐
      │ 调用：Provider.complete(messages,      │
      │       tools, system_prompt)            │
      └────────────────────────────────────────┘
                           ↓
      ┌────────────────────────────────────────┐
      │ 解析响应：                             │
      │  - 提取文本块                          │
      │  - 提取工具调用                        │
      │  - 检查 stop_reason                   │
      └────────────────────────────────────────┘
                           ↓
              ┌────────────────────────┐
              │ stop_reason 检查        │
              └────────────────────────┘
              ↙              ↖
         "tool_use"        "end_turn"
             ↓                  ↓
    ┌──────────────┐    ┌─────────────────┐
    │ 执行工具     │    │ 返回最终响应   │
    └──────────────┘    └─────────────────┘
             ↓                  ↓
    ┌──────────────────────────┴──────────┐
    │ 触发：ToolHook::on_tool_validate()  │
    └──────────────────────────────────────┘
             ↓
    ┌──────────────────────────────────────┐
    │ 调用：Tool.validate_input()          │
    └──────────────────────────────────────┘
             ↓
    ┌──────────────────────────────────────┐
    │ 触发：ToolHook::on_tool_execute()   │
    └──────────────────────────────────────┘
             ↓
    ┌──────────────────────────────────────┐
    │ 调用：ToolDispatcher.execute(tool)   │
    └──────────────────────────────────────┘
             ↓
    ┌──────────────────────────────────────┐
    │ 触发：ToolHook::on_tool_execute_done()│
    └──────────────────────────────────────┘
             ↓
    ┌──────────────────────────────────────┐
    │ 将工具结果附加到消息                 │
    └──────────────────────────────────────┘
             ↓
    ┌──────────────────────────────────────┐
    │ 循环回到 Provider.complete()         │
    └──────────────────────────────────────┘
             ↓
           循环继续...
```

#### 关键循环变量

```rust
let mut iteration = 0;          // 防止无限循环
let mut messages = vec![...];   // 消息历史
let mut final_response = "";    // 累积的响应文本

loop {
    // 检查迭代限制
    if iteration >= config.max_iterations {
        return Err(AgentError::MaxIterationsExceeded(iteration));
    }
    iteration += 1;

    // 从提供者获取响应
    let response = provider.complete(&messages, config).await?;

    // 检查停止原因
    match response.stop_reason.as_str() {
        "tool_use" => {
            // 执行工具、将结果添加到消息、继续循环
        }
        "end_turn" => {
            // 提取最终文本并中断
            final_response = extract_text(&response);
            break;
        }
        _ => return Err(AgentError::UnexpectedStopReason(...)),
    }
}

Ok(final_response)
```

---

## 数据流架构

以下是通过 Zero 系统的完整数据流：

```
┌─────────────────────────────────────────────────────────┐
│                   用户输入                               │
└─────────────────────────────────────────────────────────┘
                           ↓
┌─────────────────────────────────────────────────────────┐
│              通道（CLI/Slack/等）                       │
│            接收消息并路由到 Agent                       │
└─────────────────────────────────────────────────────────┘
                           ↓
┌─────────────────────────────────────────────────────────┐
│                   Agent                                 │
│   - 协调执行                                            │
│   - 管理消息历史                                        │
│   - 协调循环                                            │
└─────────────────────────────────────────────────────────┘
                           ↓
┌─────────────────────────────────────────────────────────┐
│              AgentLoop：DefaultAgentLoop                │
│   - 实现核心推理循环                                   │
│   - 处理工具执行                                       │
│   - 管理迭代                                           │
└─────────────────────────────────────────────────────────┘
          ↙                            ↖
    ┌──────────────┐          ┌─────────────────┐
    │ LLMProvider  │          │ ToolDispatcher  │
    │（推理）      │          │（执行）        │
    └──────────────┘          └─────────────────┘
          ↓                            ↓
    ┌──────────────┐          ┌─────────────────┐
    │ Anthropic    │          │ 工具注册表      │
    │ OpenAI       │          │                 │
    │ Ollama       │          ├─ Bash           │
    │ 自定义       │          ├─ 文件 I/O       │
    └──────────────┘          ├─ HTTP           │
                              └─ 自定义工具     │
                                     ↓
                              ┌─────────────────┐
                              │ GlobalSharedMemory
                              │                 │
                              ├─ 文件系统      │
                              ├─ 数据库        │
                              └─ 自定义        │
                                     ↓
                              ┌─────────────────┐
                              │ 存储的知识      │
                              │ 事实与模式      │
                              └─────────────────┘
                           ↑
                    循环回到提供者
                           ↓
┌─────────────────────────────────────────────────────────┐
│                   最终响应                               │
└─────────────────────────────────────────────────────────┘
                           ↓
┌─────────────────────────────────────────────────────────┐
│                   通道                                   │
│            将响应发送回用户                             │
└─────────────────────────────────────────────────────────┘
                           ↓
┌─────────────────────────────────────────────────────────┐
│                   用户输出                               │
└─────────────────────────────────────────────────────────┘
```

---

## 分层架构

Zero 组织为清晰的层级，每个层级都有独特的责任：

```
┌──────────────────────────────────────────────────────────┐
│ 第 5 层：应用与集成                                      │
│ ├─ CLI 接口                                             │
│ ├─ Web API                                              │
│ ├─ IDE 集成                                             │
│ └─ 自定义应用                                           │
├──────────────────────────────────────────────────────────┤
│ 第 4 层：运行时与协调                                   │
│ ├─ AgentLoop（核心推理循环）                          │
│ ├─ TaskManager（任务持久化 - S7+）                    │
│ ├─ TeamCoordinator（多 Agent - S9+）                  │
│ └─ 调度器                                               │
├──────────────────────────────────────────────────────────┤
│ 第 3 层：核心 Trait 层（基础）                         │
│ ├─ Agent Trait ──────────────┐                        │
│ ├─ Tool Trait ────────┐       │                        │
│ ├─ Provider Trait ────┼───────┼─→ 这 5 个 Trait      │
│ ├─ Channel Trait ─────┤       │   定义整个系统       │
│ └─ Memory Trait ──────┘       │   的契约             │
│                               │                        │
├──────────────────────────────────────────────────────────┤
│ 第 2 层：实现层                                          │
│ ├─ 提供者：Anthropic、OpenAI、Ollama                  │
│ ├─ 工具：Bash、File、HTTP、Calculator                │
│ ├─ 通道：CLI、Slack、Discord、Email、Telegram        │
│ ├─ 记忆后端：Filesystem、SQLite、PostgreSQL          │
│ └─ 钩子：可观测性与扩展点                            │
├──────────────────────────────────────────────────────────┤
│ 第 1 层：基础设施与支持                                │
│ ├─ 错误处理（thiserror）                              │
│ ├─ 配置管理                                            │
│ ├─ 安全与沙箱                                          │
│ ├─ 监控与日志                                          │
│ └─ 异步运行时（Tokio）                              │
└──────────────────────────────────────────────────────────┘
```

**关键设计原则**：每一层只依赖其下面的层级。这使得每层可以独立测试和扩展。

---

## 扩展机制

### 1. 实现新 Trait

要扩展 Zero，实现其中一个核心 Trait：

```rust
// 示例：自定义工具
use zero_core::{Tool, ToolMetadata, ToolOutput, ToolContext, ToolError};
use async_trait::async_trait;

pub struct MyCustomTool;

#[async_trait]
impl Tool for MyCustomTool {
    fn metadata(&self) -> ToolMetadata { /* ... */ }
    async fn execute(&self, input: &str, ctx: &ToolContext) -> Result<ToolOutput, ToolError> { /* ... */ }
}

// 示例：自定义提供者
use zero_core::LLMProvider;

pub struct MyCustomProvider;

#[async_trait]
impl LLMProvider for MyCustomProvider {
    fn name(&self) -> &str { "my_provider" }
    fn capabilities(&self) -> ModelCapability { /* ... */ }
    async fn complete(&self, prompt: &str, opts: CompleteOpts) -> Result<String, ProviderError> { /* ... */ }
}

// 示例：自定义通道
use zero_core::Channel;

pub struct MyCustomChannel;

#[async_trait]
impl Channel for MyCustomChannel {
    fn name(&self) -> &str { "my_channel" }
    async fn send(&self, msg: &Message) -> Result<(), ChannelError> { /* ... */ }
    async fn connect(&self) -> Result<(), ChannelError> { /* ... */ }
}

// 示例：自定义记忆后端
use zero_core::GlobalSharedMemory;

pub struct MyMemoryBackend;

#[async_trait]
impl GlobalSharedMemory for MyMemoryBackend {
    async fn store(&self, namespace: &str, key: &str, value: &str) -> Result<(), MemoryError> { /* ... */ }
    async fn retrieve(&self, namespace: &str, key: &str) -> Result<Option<String>, MemoryError> { /* ... */ }
}
```

### 2. 用于可观测性的钩子系统

钩子允许你在关键点观察和修改行为：

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

### 3. 注册和组合

```rust
// 创建和组合组件
let provider = Arc::new(AnthropicProvider::new(api_key));
let tool_registry = Arc::new(ToolRegistry::new());
let memory = Arc::new(FilesystemMemory::new(path));

// 注册工具
tool_registry.register(Box::new(BashTool)).await;
tool_registry.register(Box::new(FileIOTool)).await;

// 注册钩子
let hook_manager = Arc::new(HookManager::new());
hook_manager.register_agent_hook(Box::new(LoggingHook));

// 创建带钩子的循环
let loop_impl = DefaultAgentLoop::new(provider, tool_registry)
    .with_hooks(hook_manager);
```

---

## 常见设计模式

### 1. 构建器模式

逐步构建复杂配置：

```rust
let config = AgentLoopConfig::builder()
    .with_max_iterations(50)
    .with_temperature(0.7)
    .with_timeout_secs(30)
    .build();
```

### 2. 工厂模式

基于配置创建 Agent：

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

### 3. 策略模式

为不同场景提供不同实现：

```rust
pub async fn execute_task(
    task: &Task,
    strategy: Box<dyn ExecutionStrategy>,
) -> Result<TaskResult> {
    strategy.execute(task).await
}
```

### 4. 中间件模式

通过钩子添加功能：

```rust
pub struct CachingHook {
    cache: Arc<RwLock<HashMap<String, String>>>,
}

#[async_trait]
impl ProviderHook for CachingHook {
    async fn on_provider_call(&self, prompt: &str) -> Result<(), String> {
        if let Some(cached) = self.cache.read().await.get(prompt) {
            // 返回缓存的响应
        }
        Ok(())
    }
}
```

### 5. 依赖注入

通过构造函数注入依赖：

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

## 性能考虑

### 1. 异步优先设计

所有 I/O 操作都使用 `async-trait` 和 `tokio` 进行异步处理：

```rust
// 不要阻塞运行时
async fn execute(&self) -> Result<...> {
    // 对 I/O 操作使用 .await
    let result = external_api.call().await?;
    Ok(result)
}
```

**好处：**
- 单线程可以处理数千个并发操作
- 阻塞 I/O 时零 CPU 浪费
- 对多 Agent 系统的自然支持

### 2. 并发级别

```
级别 1：Agent 循环       - 每个 Agent 单个异步任务
级别 2：工具执行         - 可配置并发工具
级别 3：多 Agent         - 多个 Agent 并行
级别 4：团队协调         - Lead + N 个 Worker Agent
```

### 3. 内存效率

- 消息历史保存在内存中但可以压缩（见压缩策略）
- 工具在需要前不加载
- 记忆按命名空间组织以高效清理
- 未使用的条目可归档到持久存储

### 4. 超时管理

```rust
let result = tokio::time::timeout(
    Duration::from_secs(config.tool_timeout),
    dispatcher.execute(tool_call),
).await?;
```

防止工具阻塞整个循环。

### 5. 提供者优化

- 多个请求的连接池
- 可能时的请求批处理
- 对简单任务回退到更便宜的模型
- 重复查询的缓存

---

## 总结

Zero 的 Trait 驱动架构提供：

1. **灵活性**：无需改变其他组件即可交换任何组件
2. **可测试性**：为任何 Trait 提供 Mock 实现
3. **可扩展性**：最小努力下添加新工具、提供者、通道
4. **清晰性**：Trait 充当可执行的契约
5. **性能**：异步优先设计以实现高并发
6. **可观测性**：在每个关键点提供钩子
7. **可组合性**：从简单 Trait 构建复杂系统

5 个核心 Trait 构成基础，其他一切都是可以扩展、替换或自定义的实现细节。

---

## 下一步

- 查看 **04-examples.md** 了解实际实现示例
- 检查 **05-api-reference.md** 获取详细的 API 文档
- 查阅 **06-hooks-system.md** 了解可观测性
- 探索 **07-contributing.md** 开始扩展 Zero
