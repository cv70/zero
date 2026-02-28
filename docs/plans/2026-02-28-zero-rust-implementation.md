# Zero 通用 Agent 运行时实现计划

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** 构建 zero 通用 Agent 运行时平台核心，包括内核架构、Trait 定义、基础工具系统和记忆系统

**Architecture:** 内核 + 扩展模式。zero-core 提供核心 Trait 和引擎，扩展通过实现 Trait 接入。先实现核心内核和 CLI，建立最小可运行产品。

**Tech Stack:** Rust (2024), tokio, serde, clap, axum, rusqlite, async-trait

---

## 阶段一：项目初始化与内核基础

### Task 1: 创建 Workspace 结构

**Files:**
- Modify: `/home/o/space/zero/zero/Cargo.toml`
- Create: `/home/o/space/zero/zero/CLAUDE.md`

**Step 1: 更新 Workspace 根配置**

```toml
[workspace]
resolver = "2"
members = [
    "zero-core",
    "zero-cli",
    "zero-api",
]

[workspace.package]
version = "0.1.0"
edition = "2024"
rust-version = "1.75"

[workspace.dependencies]
tokio = { version = "1", features = ["full"] }
serde = { version = "1", features = ["derive"] }
serde_json = "1"
thiserror = "1"
anyhow = "1"
tracing = "0.1"
tracing-subscriber = "0.3"
async-trait = "0.1"
uuid = { version = "1", features = ["v4"] }
clap = { version = "4", features = ["derive"] }
axum = "0.7"
reqwest = { version = "0.12", features = ["json"] }
```

**Step 2: 创建 CLAUDE.md**

```markdown
# CLAUDE.md - AI 开发者协议

本项目采用 Trait 驱动架构，所有核心能力通过 Trait 定义。

## 核心 Trait

- `Agent` - Agent 工厂
- `Tool` - 统一工具抽象
- `GlobalSharedMemory` - 全局共享记忆
- `LLMProvider` - 模型提供者
- `Channel` - 消息通道

## 开发原则

1. 使用 Rust 2024 edition
2. 所有 Trait 必须使用 async-trait
3. 错误处理使用 thiserror
4. 异步运行时使用 tokio
5. 提交前运行 cargo build 确保编译通过
```

**Step 3: 验证编译**

Run: `cd /home/o/space/zero/zero && cargo build`
Expected: SUCCESS

**Step 4: 提交**

```bash
cd /home/o/space/zero/zero
git add Cargo.toml CLAUDE.md
git commit -m "chore: init workspace structure"
```

---

### Task 2: 创建 zero-core 基础结构

**Files:**
- Create: `/home/o/space/zero/zero/zero-core/Cargo.toml`
- Create: `/home/o/space/zero/zero/zero-core/src/lib.rs`
- Create: `/home/o/space/zero/zero/zero-core/src/error.rs`

**Step 1: 创建 zero-core/Cargo.toml**

```toml
[package]
name = "zero-core"
version.workspace = true
edition.workspace = true
rust-version.workspace = true

[dependencies]
tokio.workspace = true
serde.workspace = true
serde_json.workspace = true
thiserror.workspace = true
anyhow.workspace = true
tracing.workspace = true
async-trait.workspace = true
uuid.workspace = true

[dev-dependencies]
tokio-test = "0.4"
```

**Step 2: 创建 error.rs 定义核心错误类型**

```rust
use thiserror::Error;

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

#[derive(Error, Debug)]
pub enum AgentError {
    #[error("Execution failed: {0}")]
    ExecutionFailed(String),
    #[error("Context error: {0}")]
    ContextError(String),
}

#[derive(Error, Debug)]
pub enum ToolError {
    #[error("Execution failed: {0}")]
    ExecutionFailed(String),
    #[error("Invalid input: {0}")]
    InvalidInput(String),
    #[error("Not supported: {0}")]
    NotSupported(String),
}

#[derive(Error, Debug)]
pub enum MemoryError {
    #[error("Store failed: {0}")]
    StoreFailed(String),
    #[error("Retrieve failed: {0}")]
    RetrieveFailed(String),
    #[error("Search failed: {0}")]
    SearchFailed(String),
}

#[derive(Error, Debug)]
pub enum ProviderError {
    #[error("Request failed: {0}")]
    RequestFailed(String),
    #[error("Rate limited: {0}")]
    RateLimited(String),
    #[error("Invalid response: {0}")]
    InvalidResponse(String),
}

#[derive(Error, Debug)]
pub enum ChannelError {
    #[error("Send failed: {0}")]
    SendFailed(String),
    #[error("Receive failed: {0}")]
    ReceiveFailed(String),
}
```

**Step 3: 创建 lib.rs**

```rust
pub mod error;
pub mod agent;
pub mod tool;
pub mod memory;
pub mod provider;
pub mod channel;

pub use error::{ZeroError, AgentError, ToolError, MemoryError, ProviderError, ChannelError};
```

**Step 4: 验证编译**

Run: `cd /home/o/space/zero/zero && cargo build -p zero-core`
Expected: SUCCESS

**Step 5: 提交**

```bash
cd /home/o/space/zero/zero
git add zero-core/
git commit -m "feat(zero-core): add core error types and module structure"
```

---

### Task 3: 实现 Agent Trait

**Files:**
- Create: `/home/o/space/zero/zero/zero-core/src/agent/mod.rs`
- Create: `/home/o/space/zero/zero/zero-core/src/agent/trait.rs`
- Create: `/home/o/space/zero/zero/zero-core/src/agent/context.rs`

**Step 1: 创建 agent/trait.rs**

```rust
use crate::error::AgentError;
use async_trait::async_trait;
use std::collections::HashMap;

/// Agent 执行结果
#[derive(Debug, Clone)]
pub struct AgentResponse {
    pub content: String,
    pub tool_calls: Vec<ToolCall>,
    pub metadata: HashMap<String, String>,
}

/// Tool 调用请求
#[derive(Debug, Clone)]
pub struct ToolCall {
    pub id: String,
    pub name: String,
    pub arguments: String,
}

/// Agent  Trait - 定义 Agent 的核心接口
#[async_trait]
pub trait Agent: Send + Sync {
    /// Agent 名称
    fn name(&self) -> &str;
    
    /// Agent 系统提示词
    fn system_prompt(&self) -> &str;
    
    /// Agent 描述
    fn description(&self) -> &str;
    
    /// 执行 Agent
    async fn run(&mut self, ctx: &mut AgentContext) -> Result<AgentResponse, AgentError>;
}
```

**Step 2: 创建 agent/context.rs**

```rust
use crate::tool::Tool;

/// Agent 执行上下文
pub struct AgentContext {
    pub session_id: String,
    pub tools: Vec<Box<dyn Tool>>,
    pub history: Vec<HistoryEntry>,
}

impl AgentContext {
    pub fn new(session_id: String) -> Self {
        Self {
            session_id,
            tools: Vec::new(),
            history: Vec::new(),
        }
    }
}

/// 历史条目
#[derive(Debug, Clone)]
pub struct HistoryEntry {
    pub role: String,
    pub content: String,
}
```

**Step 3: 创建 agent/mod.rs**

```rust
pub mod trait;
pub mod context;

pub use trait::{Agent, AgentResponse, ToolCall};
pub use context::{AgentContext, HistoryEntry};
```

**Step 4: 验证编译**

Run: `cd /home/o/space/zero/zero && cargo build -p zero-core`
Expected: SUCCESS

**Step 5: 提交**

```bash
cd /home/o/space/zero/zero
git add zero-core/src/agent/
git commit -m "feat(agent): add Agent trait and context"
```

---

### Task 4: 实现 Tool Trait（统一工具抽象）

**Files:**
- Create: `/home/o/space/zero/zero/zero-core/src/tool/mod.rs`
- Create: `/home/o/space/zero/zero/zero-core/src/tool/trait.rs`
- Create: `/home/o/space/zero/zero/zero-core/src/tool/metadata.rs`
- Create: `/home/o/space/zero/zero/zero-core/src/tool/registry.rs`

**Step 1: 创建 tool/trait.rs**

```rust
use crate::error::ToolError;
use crate::provider::ModelCapability;
use async_trait::async_trait;
use serde_json::Value;

/// Tool 输出类型
#[derive(Debug, Clone)]
pub enum ToolOutput {
    Text(String),
    Image { data: Vec<u8>, mime_type: String },
    Video { data: Vec<u8>, mime_type: String },
    Audio { data: Vec<u8>, mime_type: String },
}

impl ToolOutput {
    pub fn text(s: impl Into<String>) -> Self {
        ToolOutput::Text(s.into())
    }
}

/// Tool 元数据
#[derive(Debug, Clone)]
pub struct ToolMetadata {
    pub name: String,
    pub description: String,
    pub input_schema: Value,
    pub required_capabilities: Vec<ModelCapability>,
}

/// Tool 执行上下文
pub struct ToolContext {
    pub session_id: String,
    pub working_dir: Option<String>,
}

impl ToolContext {
    pub fn new(session_id: String) -> Self {
        Self {
            session_id,
            working_dir: None,
        }
    }
}

/// 统一 Tool Trait
#[async_trait]
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

**Step 2: 创建 tool/metadata.rs**

```rust
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolDefinition {
    pub name: String,
    pub description: String,
    #[serde(default)]
    pub input_schema: serde_json::Value,
}
```

**Step 3: 创建 tool/registry.rs**

```rust
use crate::error::ZeroError;
use crate::tool::Tool;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

pub struct ToolRegistry {
    tools: Arc<RwLock<HashMap<String, Box<dyn Tool>>>>,
}

impl ToolRegistry {
    pub fn new() -> Self {
        Self {
            tools: Arc::new(RwLock::new(HashMap::new())),
        }
    }
    
    pub async fn register(&self, tool: Box<dyn Tool>) {
        let name = tool.metadata().name.clone();
        self.tools.write().await.insert(name, tool);
    }
    
    pub async fn get(&self, name: &str) -> Option<Box<dyn Tool>> {
        self.tools.read().await.get(name).cloned()
    }
    
    pub async fn list(&self) -> Vec<String> {
        self.tools.read().await.keys().cloned().collect()
    }
}
```

**Step 4: 创建 tool/mod.rs**

```rust
pub mod trait;
pub mod metadata;
pub mod registry;

pub use trait::{Tool, ToolOutput, ToolMetadata, ToolContext};
pub use metadata::ToolDefinition;
pub use registry::ToolRegistry;
```

**Step 5: 验证编译**

Run: `cd /home/o/space/zero/zero && cargo build -p zero-core`
Expected: SUCCESS

**Step 6: 提交**

```bash
cd /home/o/space/zero/zero
git add zero-core/src/tool/
git commit -m "feat(tool): add unified Tool trait and registry"
```

---

### Task 5: 实现 Memory Trait

**Files:**
- Create: `/home/o/space/zero/zero/zero-core/src/memory/mod.rs`
- Create: `/home/o/space/zero/zero/zero-core/src/memory/trait.rs`
- Create: `/home/o/space/zero/zero/zero-core/src/memory/backend/filesystem.rs`

**Step 1: 创建 memory/trait.rs**

```rust
use crate::error::MemoryError;
use async_trait::async_trait;
use std::collections::HashMap;

/// Memory 条目
#[derive(Debug, Clone)]
pub struct MemoryEntry {
    pub key: String,
    pub value: String,
    pub timestamp: i64,
    pub metadata: HashMap<String, String>,
}

/// 全局共享 Memory Trait
#[async_trait]
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

**Step 2: 创建 memory/backend/filesystem.rs**

```rust
use crate::error::MemoryError;
use crate::memory::{GlobalSharedMemory, MemoryEntry};
use async_trait::async_trait;
use std::collections::HashMap;
use std::path::PathBuf;
use tokio::fs;

/// 文件系统内存后端
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
    async fn store(&self, namespace: &str, key: &str, value: &str) -> Result<(), MemoryError> {
        let path = self.namespace_path(namespace).join(format!("{}.json", key));
        fs::create_dir_all(path.parent().unwrap()).await.map_err(|e| MemoryError::StoreFailed(e.to_string()))?;
        fs::write(&path, value).await.map_err(|e| MemoryError::StoreFailed(e.to_string()))?;
        Ok(())
    }
    
    async fn retrieve(&self, namespace: &str, key: &str) -> Result<Option<String>, MemoryError> {
        let path = self.namespace_path(namespace).join(format!("{}.json", key));
        match fs::read_to_string(&path).await {
            Ok(v) => Ok(Some(v)),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(None),
            Err(e) => Err(MemoryError::RetrieveFailed(e.to_string())),
        }
    }
    
    async fn search(&self, _namespace: &str, _query: &str, _limit: usize) -> Result<Vec<MemoryEntry>, MemoryError> {
        // 简化实现：返回空结果
        Ok(Vec::new())
    }
    
    async fn delete(&self, namespace: &str, key: &str) -> Result<(), MemoryError> {
        let path = self.namespace_path(namespace).join(format!("{}.json", key));
        fs::remove_file(&path).await.map_err(|e| MemoryError::StoreFailed(e.to_string()))?;
        Ok(())
    }
    
    async fn list_keys(&self, namespace: &str) -> Result<Vec<String>, MemoryError> {
        let dir = self.namespace_path(namespace);
        let mut keys = Vec::new();
        let mut entries = fs::read_dir(&dir).await.map_err(|e| MemoryError::RetrieveFailed(e.to_string()))?;
        while let Some(entry) = entries.next_entry().await.map_err(|e| MemoryError::RetrieveFailed(e.to_string()))? {
            if let Some(name) = entry.path().file_stem() {
                keys.push(name.to_string_lossy().to_string());
            }
        }
        Ok(keys)
    }
}
```

**Step 3: 创建 memory/mod.rs**

```rust
pub mod trait;
pub mod backend;

pub use trait::{GlobalSharedMemory, MemoryEntry};
pub use backend::filesystem::FilesystemMemory;
```

**Step 4: 验证编译**

Run: `cd /home/o/space/zero/zero && cargo build -p zero-core`
Expected: SUCCESS

**Step 5: 提交**

```bash
cd /home/o/space/zero/zero
git add zero-core/src/memory/
git commit -m "feat(memory): add GlobalSharedMemory trait and filesystem backend"
```

---

### Task 6: 实现 Provider Trait（模型抽象）

**Files:**
- Create: `/home/o/space/zero/zero/zero-core/src/provider/mod.rs`
- Create: `/home/o/space/zero/zero/zero-core/src/provider/trait.rs`

**Step 1: 创建 provider/trait.rs**

```rust
use crate::error::ProviderError;
use crate::tool::ToolMetadata;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// 模型能力枚举
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ModelCapability {
    TextOnly,
    TextAndImages,
    TextAndVideo,
    Multimodal,
}

/// 媒体输入类型
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MediaInput {
    Image { url: String, mime_type: String },
    ImageBytes { data: Vec<u8>, mime_type: String },
    Video { url: String, mime_type: String },
    Audio { url: String, mime_type: String },
}

/// 补全选项
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompleteOpts {
    pub model: String,
    #[serde(default = "default_temperature")]
    pub temperature: f32,
    pub max_tokens: Option<usize>,
    #[serde(default)]
    pub tools: Vec<ToolMetadata>,
    pub system_prompt: Option<String>,
}

fn default_temperature() -> f32 {
    0.7
}

/// Tool 调用请求
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolCall {
    pub id: String,
    pub name: String,
    pub arguments: String,
}

/// Tool 调用结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolCallResult {
    pub id: String,
    pub result: Result<String, String>,
}

/// LLM Provider Trait
#[async_trait]
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
        _prompt: &str, 
        _media: &[MediaInput], 
        _opts: CompleteOpts
    ) -> Result<String, ProviderError> {
        Err(ProviderError::RequestFailed("Multimodal not supported".into()))
    }
    
    /// Tool 调用补全（可选实现）
    async fn complete_with_tools(
        &self, 
        _prompt: &str, 
        _tools: &[ToolCall], 
        _opts: CompleteOpts
    ) -> Result<ToolCallResult, ProviderError> {
        Err(ProviderError::RequestFailed("Tool calling not supported".into()))
    }
}
```

**Step 2: 创建 provider/mod.rs**

```rust
pub mod trait;

pub use trait::{
    LLMProvider, ModelCapability, MediaInput, CompleteOpts, 
    ToolCall, ToolCallResult
};
```

**Step 3: 验证编译**

Run: `cd /home/o/space/zero/zero && cargo build -p zero-core`
Expected: SUCCESS

**Step 4: 提交**

```bash
cd /home/o/space/zero/zero
git add zero-core/src/provider/
git commit -m "feat(provider): add LLMProvider trait with capability support"
```

---

### Task 7: 实现 Channel Trait

**Files:**
- Create: `/home/o/space/zero/zero/zero-core/src/channel/mod.rs`
- Create: `/home/o/space/zero/zero/zero-core/src/channel/trait.rs`

**Step 1: 创建 channel/trait.rs**

```rust
use crate::error::ChannelError;
use crate::provider::MediaInput;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};

/// 消息结构
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    pub id: String,
    pub from: String,
    pub to: String,
    pub content: String,
    pub timestamp: i64,
    #[serde(default)]
    pub attachments: Vec<MediaInput>,
}

impl Message {
    pub fn new(from: impl Into<String>, content: impl Into<String>) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            from: from.into(),
            to: String::new(),
            content: content.into(),
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs() as i64,
            attachments: Vec::new(),
        }
    }
}

/// Channel Trait
#[async_trait]
pub trait Channel: Send + Sync {
    /// Channel 名称
    fn name(&self) -> &str;
    
    /// 发送消息
    async fn send(&self, msg: &Message) -> Result<(), ChannelError>;
    
    /// 接收消息（可选）
    async fn receive(&self) -> Result<Option<Message>, ChannelError>;
    
    /// 连接
    async fn connect(&self) -> Result<(), ChannelError>;
    
    /// 断开
    async fn disconnect(&self) -> Result<(), ChannelError>;
}
```

**Step 2: 创建 channel/mod.rs**

```rust
pub mod trait;

pub use trait::{Channel, Message};
```

**Step 3: 验证编译**

Run: `cd /home/o/space/zero/zero && cargo build -p zero-core`
Expected: SUCCESS

**Step 4: 提交**

```bash
cd /home/o/space/zero/zero
git add zero-core/src/channel/
git commit -m "feat(channel): add Channel trait and Message type"
```

---

## 阶段二：CLI 与 API 基础

### Task 8: 创建 zero-cli 基础结构

**Files:**
- Create: `/home/o/space/zero/zero/zero-cli/Cargo.toml`
- Create: `/home/o/space/zero/zero/zero-cli/src/main.rs`

**Step 1: 创建 zero-cli/Cargo.toml**

```toml
[package]
name = "zero-cli"
version.workspace = true
edition.workspace = true
rust-version.workspace = true

[dependencies]
zero-core.workspace = true
tokio.workspace = true
clap.workspace = true
anyhow.workspace = true
tracing.workspace = true
tracing-subscriber.workspace = true
```

**Step 2: 创建 main.rs**

```rust
use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "zero")]
#[command(about = "Zero - 通用 Agent 运行时")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// 启动交互式会话
    Run,
    /// 列出可用 Agent
    ListAgents,
    /// 列出可用工具
    ListTools,
    /// 执行单次命令
    Exec { prompt: String },
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();
    
    let cli = Cli::parse();
    
    match cli.command {
        Commands::Run => {
            println!("Starting interactive session...");
            // TODO: 实现交互式会话
        }
        Commands::ListAgents => {
            println!("Available agents:");
            // TODO: 列出 Agent
        }
        Commands::ListTools => {
            println!("Available tools:");
            // TODO: 列出 Tools
        }
        Commands::Exec { prompt } => {
            println!("Executing: {}", prompt);
            // TODO: 执行单次命令
        }
    }
    
    Ok(())
}
```

**Step 3: 更新 Workspace 配置**

在 `/home/o/space/zero/zero/Cargo.toml` 中添加:

```toml
[workspace]
members = [
    "zero-core",
    "zero-cli",
    "zero-api",
]
```

**Step 4: 验证编译**

Run: `cd /home/o/space/zero/zero && cargo build -p zero-cli`
Expected: SUCCESS

**Step 5: 提交**

```bash
cd /home/o/space/zero/zero
git add zero-cli/
git add Cargo.toml
git commit -m "feat(zero-cli): add basic CLI structure"
```

---

### Task 9: 创建 zero-api 基础结构

**Files:**
- Create: `/home/o/space/zero/zero/zero-api/Cargo.toml`
- Create: `/home/o/space/zero/zero/zero-api/src/main.rs`

**Step 1: 创建 zero-api/Cargo.toml**

```toml
[package]
name = "zero-api"
version.workspace = true
edition.workspace = true
rust-version.workspace = true

[dependencies]
zero-core.workspace = true
tokio.workspace = true
axum.workspace = true
serde.workspace = true
serde_json.workspace = true
anyhow.workspace = true
tracing.workspace = true
tracing-subscriber.workspace = true
tower.workspace = true
```

**Step 2: 创建 main.rs**

```rust
use axum::{
    routing::{get, post},
    Router,
};
use std::net::SocketAddr;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();
    
    let app = Router::new()
        .route("/", get(root))
        .route("/api/v1/health", get(health));
    
    let addr = SocketAddr::from(([0, 0, 0, 0], 3000));
    tracing::info!("Starting API server on {}", addr);
    
    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

async fn root() -> &'static str {
    "Zero API Server"
}

async fn health() -> &'static str {
    "OK"
}
```

**Step 3: 验证编译**

Run: `cd /home/o/space/zero/zero && cargo build -p zero-api`
Expected: SUCCESS

**Step 4: 提交**

```bash
cd /home/o/space/zero/zero
git add zero-api/
git commit -m "feat(zero-api): add basic API server structure"
```

---

### Task 10: 创建扩展基础结构

**Files:**
- Create: `/home/o/space/zero/zero/zero-ext/Cargo.toml`
- Create: `/home/o/space/zero/zero/zero-ext/tools/zero-ext-tool-files/Cargo.toml`
- Create: `/home/o/space/zero/zero/zero-ext/tools/zero-ext-tool-files/src/lib.rs`

**Step 1: 创建 zero-ext/Cargo.toml**

```toml
[workspace]
resolver = "2"
```

**Step 2: 创建 zero-ext-tool-files/Cargo.toml**

```toml
[package]
name = "zero-ext-tool-files"
version = "0.1.0"
edition.workspace = true

[dependencies]
zero-core.workspace = true
tokio.workspace = true
anyhow.workspace = true
```

**Step 3: 创建简单文件读取工具**

```rust
use zero_core::{tool::*, error::ToolError};
use async_trait::async_trait;
use std::path::PathBuf;

pub struct FileReadTool {
    base_path: PathBuf,
}

impl FileReadTool {
    pub fn new(base_path: PathBuf) -> Self {
        Self { base_path }
    }
}

#[async_trait]
impl Tool for FileReadTool {
    fn metadata(&self) -> ToolMetadata {
        ToolMetadata {
            name: "file_read".to_string(),
            description: "Read file contents".to_string(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "path": { "type": "string" }
                },
                "required": ["path"]
            }),
            required_capabilities: vec![ModelCapability::TextOnly],
        }
    }
    
    async fn execute(&self, input: &str, _ctx: &ToolContext) -> Result<ToolOutput, ToolError> {
        let path: serde_json::Value = serde_json::from_str(input)
            .map_err(|e| ToolError::InvalidInput(e.to_string()))?;
        
        let path = path.get("path")
            .and_then(|v| v.as_str())
            .ok_or_else(|| ToolError::InvalidInput("path required".to_string()))?;
        
        let full_path = self.base_path.join(path);
        let content = tokio::fs::read_to_string(&full_path)
            .await
            .map_err(|e| ToolError::ExecutionFailed(e.to_string()))?;
        
        Ok(ToolOutput::text(content))
    }
}
```

**Step 4: 提交**

```bash
cd /home/o/space/zero/zero
git add zero-ext/
git commit -m "feat(zero-ext): add extension workspace structure"
```

---

## 总结

此计划包含 10 个主要任务，涵盖：

1. **项目初始化** - Workspace 和基础结构
2. **核心 Trait** - Agent, Tool, Memory, Provider, Channel
3. **CLI 应用** - 基础命令行工具
4. **API 服务** - 基础 REST API
5. **扩展生态** - 扩展包组织结构

---

**Plan complete and saved to `docs/plans/2026-02-28-zero-rust-implementation.md`. Two execution options:**

**1. Subagent-Driven (this session)** - I dispatch fresh subagent per task, review between tasks, fast iteration

**2. Parallel Session (separate)** - Open new session with executing-plans, batch execution with checkpoints

**Which approach?**
