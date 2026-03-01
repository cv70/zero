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

## 钩子系统 (Hooks)

项目增加了完整的钩子系统，支持对 Agent、Tool、Channel、Provider 和 Memory 操作进行插件化扩展。

### 钩子类型

| 钩子名称 | 功能描述 |
|---------|---------|
| `AgentHook` | Agent 执行钩子 |
| `ToolHook` | Tool 执行钩子 |
| `ChannelHook` | Channel 消息钩子 |
| `ProviderHook` | Provider 调用钩子 |
| `MemoryHook` | Memory 访问钩子 |
| `ConfigHook` | 配置加载/保存钩子 |

### 钩子使用示例

```rust
use tokio;
use async_trait::async_trait;
use zero_core::HookManager;

/// Agent 执行钩子
#[derive(Debug, Clone)]
pub struct AgentExecutionHook {}

impl AgentExecutionHook {
    pub fn new() -> Self {
        Self {}
    }
}

/// Tool 执行钩子
#[derive(Debug, Clone)]
pub struct ToolExecutionHook {}

impl ToolExecutionHook {
    pub fn new() -> Self {
        Self {}
    }
}

/// Channel 消息钩子
#[derive(Debug, Clone)]
pub struct ChannelMessageHook {}

impl ChannelMessageHook {
    pub fn new() Self {
        Self {}
    }
}

/// Provider 调用钩子
#[derive(Debug, Clone)]
pub struct ProviderCallHook {}

impl ProviderCallHook {
    pub fn new() Self {
        Self {}
    }
}

/// Memory 访问钩子
#[derive(Debug, Clone)]
pub struct MemoryAccessHook {}

impl MemoryAccessHook {
    pub fn new() Self {
        Self {}
    }
}

/// 配置加载/保存钩子
#[derive(Debug, Clone)]
pub struct ConfigHook {}

impl ConfigHook {
    pub fn new() Self {
        Self {}
    }
}