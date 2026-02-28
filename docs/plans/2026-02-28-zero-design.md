# Zero — 通用 Agent 运行时平台设计

**版本**: 1.0  
**日期**: 2026-02-28  
**状态**: 待用户批准

---

## 1. 项目概述

### 1.1 项目名称

**zero** — 通用 Agent 运行时平台

### 1.2 核心定位

- **通用 Agent 运行时平台**，全栈通吃（开发者 / 企业 / 个人用户）
- **Rust 优先** + **Trait 架构** + **轻量运行时**
- 多模态交互：CLI (TUI) + Web + Desktop + REST/gRPC API

### 1.3 设计原则

1. **YAGNI 原则** — 去除所有不必要功能
2. **Trait 驱动架构** — 一切皆可交换
3. **扩展优先** — 核心最小化，能力通过扩展提供
4. **统一抽象** — 通过统一 Trait 消除不同实现方式的差异

---

## 2. 架构设计

### 2.1 整体架构：内核 + 扩展模式

```
┌─────────────────────────────────────────────────────────────────────────┐
│                           zero (应用层)                                  │
│   ┌─────────────┐  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐  │
│   │  CLI (TUI) │  │  Web UI    │  │  Desktop    │  │  API Server │  │
│   └─────────────┘  └─────────────┘  └─────────────┘  └─────────────┘  │
├─────────────────────────────────────────────────────────────────────────┤
│                        zero-core (内核)                                 │
│   ┌─────────────────────────────────────────────────────────────────┐   │
│   │  Agent Engine                                                     │   │
│   │  - Agent Trait (工厂模式)                                         │   │
│   │  - 多 Agent 协调器                                               │   │
│   │  - 会话 / 线程管理                                               │   │
│   └─────────────────────────────────────────────────────────────────┘   │
│   ┌─────────────────────────────────────────────────────────────────┐   │
│   │  Tool System                                                      │   │
│   │  - UnifiedTool Trait (统一抽象)                                  │   │
│   │  - Tool Adapter Layer (适配器层)                                 │   │
│   │  - 支持：声明式 / MCP / Skills / Rust 实现                       │   │
│   └─────────────────────────────────────────────────────────────────┘   │
│   ┌─────────────────────────────────────────────────────────────────┐   │
│   │  Memory System                                                    │   │
│   │  - Agent 隔离上下文                                              │   │
│   │  - GlobalSharedMemory Trait (全局共享记忆抽象)                  │   │
│   │  - 文件系统风格存储 (默认实现)                                   │   │
│   └─────────────────────────────────────────────────────────────────┘   │
│   ┌─────────────────────────────────────────────────────────────────┐   │
│   │  Model Router                                                     │   │
│   │  - 能力感知路由                                                 │   │
│   │  - Provider Trait (统一模型抽象)                                │   │
│   └─────────────────────────────────────────────────────────────────┘   │
├─────────────────────────────────────────────────────────────────────────┤
│                      zero-ext (扩展层)                                  │
│   ┌──────────────┐  ┌──────────────┐  ┌──────────────┐  ┌──────────┐ │
│   │ tools-*      │  │ memory-*     │  │ providers-* │  │channels-*│ │
│   │ - files      │  │ - filesystem │  │ - openai    │  │- telegram│ │
│   │ - shell      │  │ - sqlite     │  │ - anthropic │  │- discord │ │
│   │ - websearch  │  │ - postgres   │  │ - ollama    │  │- slack   │ │
│   │ - mcp        │  │ - redis      │  │ - local     │  │- email   │ │
│   │ - skills     │  │              │  │             │  │- matrix  │ │
│   └──────────────┘  └──────────────┘  └──────────────┘  └──────────┘ │
└─────────────────────────────────────────────────────────────────────────┘
```

### 2.2 扩展组织方式

按能力分类：`zero-ext-tools-*`, `zero-ext-memory-*`, `zero-ext-providers-*`, `zero-ext-channels-*`

---

## 3. 核心 Trait 定义

### 3.1 Agent Trait（工厂模式）

```rust
/// Agent 错误类型
#[derive(Debug, Error)]
pub enum AgentError {
    #[error("Execution failed: {0}")]
    ExecutionFailed(String),
    #[error("Context error: {0}")]
    ContextError(String),
    #[error("Tool error: {0}")]
    ToolError(String),
}

/// Agent 执行结果
#[derive(Debug, Clone)]
pub struct AgentResponse {
    pub content: String,
    pub tool_calls: Vec<ToolCall>,
    pub metadata: HashMap<String, String>,
}

/// Agent  Trait - 定义 Agent 的核心接口
pub trait Agent: Send + Sync {
    /// Agent 名称
    fn name(&self) -> &str;
    
    /// Agent 系统提示词
    fn system_prompt(&self) -> &str;
    
    /// Agent 描述
    fn description(&self) -> &str;
    
    /// 执行 Agent
    async fn run(&mut self, ctx: &mut AgentContext) -> Result<AgentResponse, AgentError>;
    
    /// 可选：Agent 元数据
    fn metadata(&self) -> HashMap<String, String> {
        HashMap::new()
    }
}
```

### 3.2 统一 Tool Trait

```rust
/// Tool 错误类型
#[derive(Debug, Error)]
pub enum ToolError {
    #[error("Execution failed: {0}")]
    ExecutionFailed(String),
    #[error("Invalid input: {0}")]
    InvalidInput(String),
    #[error("Timeout: {0}")]
    Timeout(String),
    #[error("Not supported: {0}")]
    NotSupported(String),
}

/// Tool 输出类型
#[derive(Debug, Clone)]
pub enum ToolOutput {
    Text(String),
    Image { data: Vec<u8>, mime_type: String },
    Video { data: Vec<u8>, mime_type: String },
    Audio { data: Vec<u8>, mime_type: String },
}

/// Tool 定义元数据
#[derive(Debug, Clone)]
pub struct ToolMetadata {
    pub name: String,
    pub description: String,
    pub input_schema: Value,  // JSON Schema
    pub required_capabilities: Vec<ModelCapability>,
}

/// 统一 Tool Trait
pub trait Tool: Send + Sync {
    /// Tool 元数据
    fn metadata(&self) -> ToolMetadata;
    
    /// 执行 Tool
    async fn execute(&self, input: &str, ctx: &ToolContext) -> Result<ToolOutput, ToolError>;
    
    /// 可选：验证输入
    fn validate_input(&self, input: &str) -> Result<(), ToolError> {
        Ok(())
    }
}
```

### 3.3 Memory Trait

```rust
/// Memory 错误类型
#[derive(Debug, Error)]
pub enum MemoryError {
    #[error("Store failed: {0}")]
    StoreFailed(String),
    #[error("Retrieve failed: {0}")]
    RetrieveFailed(String),
    #[error("Search failed: {0}")]
    SearchFailed(String),
}

/// Memory 条目
#[derive(Debug, Clone)]
pub struct MemoryEntry {
    pub key: String,
    pub value: String,
    pub timestamp: i64,
    pub metadata: HashMap<String, String>,
}

/// 全局共享 Memory Trait
pub trait GlobalSharedMemory: Send + Sync {
    /// 存储记忆
    async fn store(&self, namespace: &str, key: &str, value: &str) -> Result<(), MemoryError>;
    
    /// 检索记忆
    async fn retrieve(&self, namespace: &str, key: &str) -> Result<Option<String>, MemoryError>;
    
    /// 搜索记忆
    async fn search(&self, namespace: &str, query: &str, limit: usize) -> Result<Vec<MemoryEntry>, MemoryError>;
    
    /// 删除记忆
    async fn delete(&self, namespace: &str, key: &str) -> Result<(), MemoryError>;
    
    /// 列出所有 keys
    async fn list_keys(&self, namespace: &str) -> Result<Vec<String>, MemoryError>;
}
```

### 3.4 Model Provider Trait

```rust
/// 模型能力枚举
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ModelCapability {
    TextOnly,          // 纯文字输入
    TextAndImages,     // 文字 + 图片
    TextAndVideo,      // 文字 + 视频  
    Multimodal,        // 文字 + 图片 + 视频 + 音频
}

/// 媒体输入类型
#[derive(Debug, Clone)]
pub enum MediaInput {
    Image { url: String, mime_type: String },
    ImageBytes { data: Vec<u8>, mime_type: String },
    Video { url: String, mime_type: String },
    Audio { url: String, mime_type: String },
}

/// 补全选项
#[derive(Debug, Clone)]
pub struct CompleteOpts {
    pub model: String,
    pub temperature: f32,
    pub max_tokens: Option<usize>,
    pub tools: Vec<ToolMetadata>,
    pub system_prompt: Option<String>,
}

/// Tool 调用请求
#[derive(Debug, Clone)]
pub struct ToolCall {
    pub id: String,
    pub name: String,
    pub arguments: String,
}

/// Tool 调用结果
#[derive(Debug, Clone)]
pub struct ToolCallResult {
    pub id: String,
    pub result: Result<String, String>,
}

/// Provider 错误
#[derive(Debug, Error)]
pub enum ProviderError {
    #[error("Request failed: {0}")]
    RequestFailed(String),
    #[error("Rate limited: {0}")]
    RateLimited(String),
    #[error("Invalid response: {0}")]
    InvalidResponse(String),
}

/// LLM Provider Trait
pub trait LLMProvider: Send + Sync {
    /// Provider 名称
    fn name(&self) -> &str;
    
    /// 支持的模型能力
    fn capabilities(&self) -> ModelCapability;
    
    /// 可用模型列表
    fn available_models(&self) -> Vec<String>;
    
    /// 纯文本补全
    async fn complete(&self, prompt: &str, opts: CompleteOpts) -> Result<String, ProviderError>;
    
    /// 多模态补全（可选实现）
    async fn complete_with_media(
        &self, 
        prompt: &str, 
        media: &[MediaInput], 
        opts: CompleteOpts
    ) -> Result<String, ProviderError> {
        Err(ProviderError::NotSupported("Multimodal not supported".into()))
    }
    
    /// Tool 调用补全（可选实现）
    async fn complete_with_tools(
        &self, 
        prompt: &str, 
        tools: &[ToolCall], 
        opts: CompleteOpts
    ) -> Result<ToolCallResult, ProviderError> {
        Err(ProviderError::NotSupported("Tool calling not supported".into()))
    }
}
```

### 3.5 Channel Trait

```rust
/// Channel 错误
#[derive(Debug, Error)]
pub enum ChannelError {
    #[error("Send failed: {0}")]
    SendFailed(String),
    #[error("Receive failed: {0}")]
    ReceiveFailed(String),
    #[error("Connect failed: {0}")]
    ConnectFailed(String),
}

/// 消息结构
#[derive(Debug, Clone)]
pub struct Message {
    pub id: String,
    pub from: String,
    pub to: String,
    pub content: String,
    pub timestamp: i64,
    pub attachments: Vec<MediaInput>,
}

/// Channel Trait
pub trait Channel: Send + Sync {
    /// Channel 名称
    fn name(&self) -> &str;
    
    /// 发送消息
    async fn send(&self, msg: &Message) -> Result<(), ChannelError>;
    
    /// 接收消息（可选）
    async fn receive(&self) -> Result<Option<Message>, ChannelError>;
    
    /// 连接 / 断开
    async fn connect(&self) -> Result<(), ChannelError>;
    async fn disconnect(&self) -> Result<(), ChannelError>;
}
```

---

## 4. Tool 系统设计

### 4.1 统一适配器架构

```
┌─────────────────────────────────────────────┐
│           UnifiedTool (统一入口)             │
└─────────────────┬───────────────────────────┘
                  │
        ┌─────────┼─────────┐
        ▼         ▼         ▼
┌───────────┐ ┌───────────┐ ┌───────────┐
│ Declarative│ │   MCP    │ │  Skills  │
│  Adapter  │ │  Adapter  │ │  Adapter │
└─────┬─────┘ └─────┬─────┘ └─────┬─────┘
      │            │            │
      ▼            ▼            ▼
┌───────────┐ ┌───────────┐ ┌───────────┐
│ JSON/YAML │ │ MCP JSON  │ │  Skill   │
│  Config   │ │ Protocol  │ │  Files   │
└───────────┘ └───────────┘ └───────────┘
```

### 4.2 Tool 定义示例

**声明式 (JSON)**:
```json
{
  "name": "web_search",
  "description": "Search the web for information",
  "input_schema": {
    "type": "object",
    "properties": {
      "query": { "type": "string" },
      "limit": { "type": "integer", "default": 5 }
    },
    "required": ["query"]
  }
}
```

---

## 5. Memory 系统设计

### 5.1 分层记忆架构

```
┌─────────────────────────────────────────────┐
│              Agent Context                  │
│  ┌─────────────────────────────────────┐   │
│  │  Session Memory (会话级)             │   │
│  │  - 当前会话上下文                    │   │
│  │  - 短期记忆                          │   │
│  └─────────────────────────────────────┘   │
└─────────────────┬───────────────────────────┘
                  │
                  ▼
┌─────────────────────────────────────────────┐
│         GlobalSharedMemory (全局共享)        │
│  ┌─────────────────────────────────────┐   │
│  │  Namespace: agent-{id}               │   │
│  │  - per-agent 记忆                   │   │
│  └─────────────────────────────────────┘   │
│  ┌─────────────────────────────────────┐   │
│  │  Namespace: global                   │   │
│  │  - 共享知识                         │   │
│  │  - 跨 Agent 记忆                    │   │
│  └─────────────────────────────────────┘   │
└─────────────────────────────────────────────┘
```

### 5.2 文件系统风格存储（默认）

```
~/.zero/
├── memory/
│   ├── agent-{agent_id}/
│   │   ├── session-{session_id}/
│   │   │   ├── context.json
│   │   │   └── history/
│   │   │       ├── 001.md
│   │   │       └── 002.md
│   │   └── longterm/
│   │       ├── key-value/
│   │       │   ├── project-a.json
│   │       │   └── user-prefs.json
│   │       └── embeddings/
│   │           └── index.faiss
│   └── global/
│       ├── knowledge/
│       │   └── .index/
│       └── shared/
│           └── team-memory.json
```

---

## 6. Agent 协调设计

### 6.1 多 Agent 协调架构

```
┌─────────────────────────────────────────────┐
│           Agent Coordinator                  │
│  - 消息总线                                 │
│  - 任务分发                                 │
│  - 状态同步                                 │
└─────────────────┬───────────────────────────┘
                  │
    ┌─────────────┼─────────────┐
    ▼             ▼             ▼
┌─────────┐  ┌─────────┐  ┌─────────┐
│ Agent A │  │ Agent B │  │ Agent C │
│ (Build) │  │ (Plan)  │  │ (Test)  │
└─────────┘  └─────────┘  └─────────┘
```

### 6.2 Agent 协作消息格式

```rust
#[derive(Debug, Clone)]
pub struct AgentMessage {
    pub id: String,
    pub from: AgentId,
    pub to: Option<AgentId>,  // None = 广播
    pub content: String,
    pub msg_type: MessageType,
    pub timestamp: i64,
}

#[derive(Debug, Clone)]
pub enum MessageType {
    Request,
    Response,
    Event,
    Broadcast,
}
```

---

## 7. API 架构设计

### 7.1 REST API 端点

```
/api/v1/
├── agents/
│   ├── GET    /list
│   ├── POST   /create
│   ├── GET    /{id}
│   ├── POST   /{id}/run
│   └── DELETE /{id}
├── tools/
│   ├── GET    /list
│   ├── POST   /register
│   └── POST   /{name}/execute
├── memory/
│   ├── GET    /{namespace}/{key}
│   ├── POST   /{namespace}/{key}
│   ├── DELETE /{namespace}/{key}
│   └── GET    /{namespace}/search
├── models/
│   ├── GET    /list
│   └── POST   /complete
├── channels/
│   ├── GET    /list
│   ├── POST   /connect
│   └── POST   /send
└── sessions/
    ├── GET    /list
    ├── POST   /create
    └── GET    /{id}
```

### 7.2 gRPC 服务定义

```protobuf
service AgentService {
  rpc CreateAgent(CreateAgentRequest) returns (Agent);
  rpc RunAgent(RunAgentRequest) returns (stream AgentResponse);
  rpc ListAgents(ListAgentsRequest) returns (ListAgentsResponse);
}

service ToolService {
  rpc RegisterTool(ToolDefinition) returns (Tool);
  rpc ExecuteTool(ExecuteToolRequest) returns (ToolResult);
  rpc ListTools(ListToolsRequest) returns (ListToolsResponse);
}

service MemoryService {
  rpc Store(StoreRequest) returns (StoreResponse);
  rpc Retrieve(RetrieveRequest) returns (RetrieveResponse);
  rpc Search(SearchRequest) returns (SearchResponse);
}
```

---

## 8. 项目结构

```
/home/o/space/zero/zero/
├── Cargo.toml                 # Workspace 根配置
├── README.md
├── CLAUDE.md                  # AI 开发者协议
├── AGENTS.md                  # Agent 工程协议
├── zero-core/                 # 内核 (zero-core)
│   ├── Cargo.toml
│   └── src/
│       ├── lib.rs
│       ├── agent/             # Agent 引擎
│       │   ├── mod.rs
│       │   ├── trait.rs
│       │   ├── context.rs
│       │   └── coordinator.rs
│       ├── tool/              # 统一工具系统
│       │   ├── mod.rs
│       │   ├── trait.rs
│       │   ├── adapter/
│       │   │   ├── mod.rs
│       │   │   ├── declarative.rs
│       │   │   ├── mcp.rs
│       │   │   └── skill.rs
│       │   └── registry.rs
│       ├── memory/            # 记忆系统
│       │   ├── mod.rs
│       │   ├── trait.rs
│       │   └── backend/
│       │       ├── mod.rs
│       │       └── filesystem.rs
│       ├── provider/          # 模型提供者
│       │   ├── mod.rs
│       │   └── trait.rs
│       ├── channel/           # 消息通道
│       │   ├── mod.rs
│       │   └── trait.rs
│       └── runtime/           # 运行时
│           └── mod.rs
├── zero-cli/                  # CLI 应用 (TUI)
│   ├── Cargo.toml
│   └── src/
│       └── main.rs
├── zero-web/                  # Web UI (React)
│   ├── package.json
│   └── src/
├── zero-desktop/              # Desktop (Tauri)
│   ├── Cargo.toml
│   └── src/
├── zero-api/                  # REST/gRPC API
│   ├── Cargo.toml
│   └── src/
│       ├── main.rs
│       ├── api/
│       └── grpc/
└── zero-ext/                  # 扩展 (扩展生态)
    ├── tools/
    │   ├── zero-ext-tool-files/
    │   ├── zero-ext-tool-shell/
    │   ├── zero-ext-tool-websearch/
    │   └── zero-ext-tool-mcp/
    ├── memory/
    │   ├── zero-ext-memory-filesystem/
    │   ├── zero-ext-memory-sqlite/
    │   └── zero-ext-memory-postgres/
    ├── providers/
    │   ├── zero-ext-provider-openai/
    │   ├── zero-ext-provider-anthropic/
    │   └── zero-ext-provider-ollama/
    └── channels/
        ├── zero-ext-channel-telegram/
        ├── zero-ext-channel-discord/
        └── zero-ext-channel-email/
```

---

## 9. 优先级排序

### 9.1 第一阶段：核心验证

1. **zero-core 内核**
   - Agent Trait + 基本执行
   - 统一 Tool Trait + 基本适配器
   - 文件系统 Memory 后端
   
2. **zero-cli (TUI)**
   - 最小可用 CLI

3. **zero-api**
   - 基本 REST API

### 9.2 第二阶段：扩展生态

1. **工具扩展**
   - files, shell, websearch 工具
   - MCP 适配器
   - Skills 适配器

2. **记忆扩展**
   - SQLite 后端

3. **Provider 扩展**
   - OpenAI, Anthropic, Ollama

### 9.3 第三阶段：多模态 UI

1. **zero-web** (Web UI)
2. **zero-desktop** (Desktop)
3. **更多 Channel**

### 9.4 第四阶段：高级特性

1. 安全沙箱
2. 更多 Channel
3. RAG 能力

---

## 10. 待定事项

- [ ] 安全沙箱具体实现方案
- [ ] RAG 能力详细设计
- [ ] 部署模式（Docker/Kubernetes）
- [ ] 监控和可观测性方案

---

## 11. 批准记录

| 日期 | 版本 | 批准人 | 备注 |
|------|------|--------|------|
| 2026-02-28 | 1.0 | - | 初版设计，待批准 |
