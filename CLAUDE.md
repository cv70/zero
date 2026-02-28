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
