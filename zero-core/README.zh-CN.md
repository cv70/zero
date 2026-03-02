# zero-core

[![License](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)
[![Rust](https://img.shields.io/badge/rust-1.74+-orange.svg)](https://rust-lang.org)
[![Documentation](https://docs.rs/zero-core/badge.svg)](https://docs.rs/zero-core)

zero 智能体框架的核心抽象和接口定义

## 概述

`zero-core` 提供了零智能体框架的核心特征（Trait）定义和共享数据结构。本 crate 包含所有核心抽象的接口定义，包括：

- **[Agent]** - Agent 工厂特征
- **[Tool]** - 统一工具抽象
- **[GlobalSharedMemory]** - 全局共享内存特征
- **[LLMProvider]** - 大语言模型提供者特征
- **[Channel]** - 消息通道特征
- **[Hook]** - 用于扩展性的钩子特征

所有核心能力都定义为特征，实现了松耦合和易于测试。

## 主要特性

- **特征驱动架构** - 所有核心能力都定义为特征
- **异步优先** - 基于 `tokio` 构建，提供高性能异步执行
- **错误处理** - 使用 `thiserror` 的一致错误类型
- **序列化支持** - 完整的 `serde` 支持消息和工具定义
- **可扩展钩子** - 用于 Agent、Tool、Channel、Provider 和 Memory 操作的钩子特征

## 核心特征

### Agent 特征

```rust
use zero_core::Agent;
use zero_core::error::AgentError;
use zero_core::message::Message;
use async_trait::async_trait;

#[async_trait]
pub trait Agent: Send + Sync {
    fn name(&self) -> &str;
    fn system_prompt(&self) -> &str;
    fn description(&self) -> &str;
    async fn execute(&self, context: &AgentContext) -> Result<AgentResponse, AgentError>;
}
```

### Tool 特征

```rust
use zero_core::Tool;
use zero_core::error::ToolError;
use zero_core::tool::ToolContext;
use async_trait::async_trait;

#[async_trait]
pub trait Tool: Send + Sync {
    fn metadata(&self) -> ToolMetadata;
    async fn execute(&self, input: &str, ctx: &ToolContext) -> Result<ToolOutput, ToolError>;
}
```

### Channel 特征

```rust
use zero_core::Channel;
use zero_core::message::Message;
use zero_core::error::ChannelError;
use async_trait::async_trait;

#[async_trait]
pub trait Channel: Send + Sync {
    fn name(&self) -> &str;
    async fn send(&self, msg: &Message) -> Result<(), ChannelError>;
    async fn receive(&self) -> Result<Option<Message>, ChannelError>;
    async fn connect(&self) -> Result<(), ChannelError>;
    async fn disconnect(&self) -> Result<(), ChannelError>;
}
```

### LLM Provider 特征

```rust
use zero_core::LLMProvider;
use zero_core::error::ProviderError;
use zero_core::provider::{CompleteOpts, ToolCallResult, MediaInput, ToolMetadata;
use async_trait::async_trait;

#[async_trait]
pub trait LLMProvider: Send + Sync {
    fn name(&self) -> &str;
    fn capabilities(&self) -> ModelCapability;
    fn available_models(&self) -> Vec<String>;
    async fn complete(&self, prompt: &str, opts: CompleteOpts) -> Result<String, ProviderError>;
}
```

### Global Shared Memory 特征

```rust
use zero_core::GlobalSharedMemory;
use zero_core::error::MemoryError;
use zero_core::memory::MemoryEntry;
use async_trait::async_trait;

#[async_trait]
pub trait GlobalSharedMemory: Send + Sync {
    async fn store(&self, namespace: &str, key: &str, value: &str) -> Result<(), MemoryError>;
    async fn retrieve(&self, namespace: &str, key: &str) -> -> Result<Option<String>, MemoryError>;
    async fn search(&self, namespace: &str, query: &str, limit: usize) -> -> Result<Vec<MemoryEntry>, MemoryError>;
    async fn delete(&self, namespace: &str, key: &str) -> -> Result<(), MemoryError>;
    async fn list_keys(&self, namespace: &str) -> -> Result<Vec<String>, MemoryError>;
}
```

### Hook 特征

```rust
use zero_core::Hook;
use zero_core::error::*;
use async_trait::async_trait;

#[async_trait]
pub trait Hook: Send + Sync {
    fn name(&self) -> &str;
    fn priority(&self) -> i32 { 0 }
}

#[async_trait]
pub trait AgentHook: Hook {
    async fn on_agent_init(&self, agent_name: &str) -> Result<(), String>;
    async fn on_agent_run(&self, agent_name: &str) -> Result<(), String>;
    async fn on_agent_run_done(&self, agent_name: &str, result: &str) -> Result<(), String>;
    async fn on_agent_error(&self, agent_name: &str, error: &str) -> Result<(), String>;
}
```

## 安装

```toml
[dependencies]
zero-core = { path = "path/to/zero-core" }
tokio = { version = "1.0", features = ["full"] }
async-trait = "0.1"
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
thiserror = "1.0"
tracing = "0.1"
uuid = { version = "1.0", features = ["v4"] }
reqwest = { version = "0.12", features = ["json"] }
futures = "0.3"
anyhow = "1.0"
```

## 示例

查看 [examples](../examples/) 目录中的示例代码：

- **[01-simple-agent]** - 简单问候 agent，演示基本 agent 执行
- **[02-custom-tool]** - 自定义计算器工具，演示工具实现
- **[03-multi-agent]** - 多 agent 协调示例，包含研究、分析和报告 agent

## 项目结构

```
zero-core/
├── src/
│   ├── agent/            # Agent 特征和实现
│   │   ├── trait.rs      # Agent 特征定义
│   │   ├── agent_loop.rs # Agent 执行循环
│   │   └── context.rs    # Agent 上下文
│   ├── tool/            # Tool 特征和内置工具
│   │   ├── trait.rs     # Tool 特征定义
│   │   └── builtins/    # 内置工具（文件操作、bash等）
│   ├── channel/         # Channel 特征和实现
│   │   ├── trait.rs     # Channel 特征定义
│   │   └── mod.rs       # Channel 模块
│   ├── provider/        # LLM provider 特征和实现
│   │   ├── trait.rs     # LLMProvider 特征定义
│   │   └── mod.rs       # Provider 模块
│   ├── memory/          # GlobalSharedMemory 特征和实现
│   │   ├── trait.rs     # GlobalSharedMemory 特征定义
│   │   └── mod.rs       # Memory 模块
│   ├── message.rs       # Message 和 ContentBlock 定义
│   ├── hooks/           # 用于可扩展性的 Hook 特征
│   └── lib.rs           # 库根文件
├── examples/            # 示例实现
│   ├── 01-simple-agent.rs
│   ├── 02-custom-tool.rs
│   └── 03-multi-agent.rs
├── Cargo.toml           # 依赖项和包信息
└── README-CN.md        # 本文件
```

## 许可证

本项目采用 MIT 许可证 - 详情请参阅 [LICENSE](../LICENSE) 文件。

## 贡献

欢迎提交问题和拉取请求！

1. Fork 本仓库
2. 创建你的特性分支 (`git checkout -b feature/AmazingFeature`)
3. 提交你的修改 (`git commit -m 'Add some amazing feature'`)
4. 推送到分支 (`git push origin feature/AmazingFeature`)
5. 打开一个 Pull Request

## 致谢

- [Claude Opus 4.6](https://claude.com/claude-code) - AI 助手
- [Rust](https://www.rust-lang.org/) - Rust 语言
- [OpenAI](https://openai.com/) - GPT 模型
- [Anthropic](https://www.anthropic.com/) - Claude 模型
- [Ollama](https://ollama.com/) - 本地 LLM 推理

---

<div align="center">

**Built with [Claude Code](https://claude.com/claude-code) and [Rust](https://www.rust-lang.org/)

</div>
