# Zero — 通用 Agent 运行时平台

> **English**: [README.md](./README.md)

## Zero 是什么？

**Zero** 是一个基于 Rust 构建的**通用 Agent 运行时平台**，通过 Trait 驱动架构为构建智能、可扩展的 AI 应用而设计。

### 核心价值

- **Trait 驱动** — 所有核心能力都通过 Rust Trait 实现，完全可替换
- **高性能** — 高效的并发执行，极低的资源占用
- **完全可扩展** — 核心保持最小化，能力来自扩展

## 架构概览

```
┌─────────────────────────────────────────┐
│      应用层 (CLI/Web)                   │
├─────────────────────────────────────────┤
│      核心内核 (Trait 驱动)               │
│  Agent | 工具 | 记忆 | 提供者           │
├─────────────���───────────────────────────┤
│      扩展生态                           │
│  工具 | 提供者 | 通道                    │
└─────────────────────────────────────────┘
```

## 学习路径

根据你的目标选择学习路径：

### 🚀 [快速开始 (5 分钟)](./docs/01-getting-started.zh-CN.md)
5 分钟内让 Zero 运行起来。安装、写第一个 Agent、运行它。

**适合:** 想立即看到效果的任何人

### 🧠 [核心概念 (15 分钟)](./docs/02-core-concepts.zh-CN.md)
理解 Trait 驱动的设计哲学和 5 个核心原则，了解为什么 Zero 如此可扩展。

**适合:** 想理解设计背后的 "为什么" 的开发者

### 🏗️ [Trait 架构详解 (30 分钟)](./docs/03-trait-architecture.zh-CN.md)
深入探讨每个核心 Trait：Agent、Tool、Memory、Provider、Channel。学习它们如何交互以及如何扩展。

**适合:** 贡献者和需要构建自定义实现的高级用户

### 💡 [代码示例 (30 分钟)](./docs/04-examples.zh-CN.md)
从简单的 "Hello Agent" 到多 Agent 协作。真实、可运行的代码带详细解释。

**适合:** 喜欢看代码的学习者

### 📚 [API 参考](./docs/05-api-reference.zh-CN.md)
所有核心模块的完整 API 文档。类型签名、参数、返回值。

**适合:** 使用 Zero 进行开发，查找特定 API

### 🔌 [钩子系统 (20 分钟)](./docs/06-hooks-system.zh-CN.md)
学习 Zero 的插件/扩展系统。6 种钩子类型、使用场景、实现模式。

**适合:** 构建扩展和自定义功能

### 🛠️ [贡献指南](./docs/07-contributing.zh-CN.md)
开发环境配置、编码规范、Git 工作流、测试要求。贡献 Zero 的一切你需要知道的。

**适合:** Zero 的贡献者

## 5 分钟示例

```rust
use zero_core::{Agent, AgentContext};
use async_trait::async_trait;

struct MyAgent;

#[async_trait]
impl Agent for MyAgent {
    async fn execute(&self, context: &AgentContext) -> Result<String> {
        Ok(format!("你好，来自 {}!", context.name))
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    let agent = MyAgent;
    let context = AgentContext::new("MyAgent");
    let result = agent.execute(&context).await?;
    println!("{}", result);
    Ok(())
}
```

## 功能亮点

- **多模型支持** — OpenAI、Anthropic、Ollama 和自定义提供者
- **统一工具系统** — JSON/YAML 工具、MCP 集成、Rust 实现
- **分层记忆系统** — Agent 隔离 + 全局共享记忆
- **通道系统** — CLI、Web、Discord、Email 等
- **钩子系统** — 在每个关键点实现可扩展性
- **生产就绪** — 错误处理、异步/等待、类型安全

## 项目状态

- ✅ **阶段 1**: 核心 Trait 定义和基本执行
- ✅ **阶段 2**: 扩展生态 (工具、记忆、提供者)
- ✅ **阶段 3**: 多模态 UI (Web、Desktop)
- ✅ **阶段 4**: 高级特性 (安全、RAG)

## 安装

```bash
# 从源码构建
cargo build -p zero-core

# 运行 CLI
cargo run -p zero-cli -- --help
```

详细的设置说明，见 [快速开始](./docs/01-getting-started.zh-CN.md)。

## 路线图

- 短期: 文档改进、社区示例
- 中期: 性能优化、高级 RAG 能力
- 长期: 完整的多 Agent 团队协作、自主 Agent

## 参与贡献

Zero 欢迎贡献！见 [贡献指南](./docs/07-contributing.zh-CN.md)：
- 开发环境配置
- 编码规范和约定
- Git 工作流和提交流程
- 测试要求

## 许可证

MIT

---

**有疑问?** 查看 [FAQ 部分](./docs/02-core-concepts.zh-CN.md#常见问题) 或 [在 GitHub 上提 issue](https://github.com/your-org/zero/issues)。
