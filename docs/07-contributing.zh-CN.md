# Zero 项目贡献指南

欢迎！我们很高兴你想为 Zero 项目做贡献。本指南将帮助你快速上手开发流程，并做出有意义的贡献。

## 目录

- [开发环境设置](#开发环境设置)
- [代码风格和标准](#代码风格和标准)
- [项目结构](#项目结构)
- [Git 工作流](#git-工作流)
- [测试](#测试)
- [Trait 设计指南](#trait-设计指南)
- [常见陷阱](#常见陷阱)

---

## 开发环境设置

### 前置要求

在开始之前，请确保你已经安装了：

- **Rust 2024 edition**（或更新版本）- 从 [rustup.rs](https://rustup.rs/) 安装
- **Git** - 用于版本控制
- **文本编辑器或 IDE** - 推荐使用 VS Code 配合 rust-analyzer 扩展

### 设置开发环境

1. **克隆仓库**:
   ```bash
   git clone https://github.com/yourusername/zero.git
   cd zero
   ```

2. **安装 Rust 和依赖**:
   ```bash
   rustup update
   rustup component add rustfmt clippy
   ```

3. **构建项目**:
   ```bash
   cargo build
   ```

4. **运行测试验证设置**:
   ```bash
   cargo test
   ```

### 从源代码构建

项目是一个包含多个 crate 的 Rust workspace：

```bash
# 构建所有 crate
cargo build --release

# 构建特定 crate
cargo build -p zero-core
cargo build -p zero-cli

# 生成文档
cargo doc --open
```

### 运行测试

```bash
# 运行所有测试
cargo test

# 运行特定模块的测试
cargo test agent::

# 运行特定测试并显示输出
cargo test agent::loop_tests::test_simple_loop -- --nocapture

# 运行代码覆盖率测试（需要安装 tarpaulin）
cargo tarpaulin --out Html
```

---

## 代码风格和标准

### 命名约定

遵循 Rust 命名约定，详见 [Rust API Guidelines](https://rust-lang.github.io/api-guidelines/)：

- **类型、Trait、Enum**：`PascalCase`（例如 `AgentLoop`、`ToolDispatcher`）
- **函数、变量、参数**：`snake_case`（例如 `execute_agent`、`tool_name`）
- **常量**：`UPPER_SNAKE_CASE`（例如 `MAX_ITERATIONS`）
- **私有项**：如需要可以用下划线前缀（例如 `_internal_method`）

### async-trait 使用要求

所有包含异步方法的 Trait **必须** 使用 `async-trait` 属性：

```rust
use async_trait::async_trait;

#[async_trait]
pub trait Agent: Send + Sync {
    async fn execute(&self, context: &AgentContext) -> Result<AgentResponse, AgentError>;

    fn name(&self) -> &str;
}
```

**关键要求**：
- 所有异步 Trait 方法都必须有 `async-trait` 属性
- Trait 必须实现 `Send + Sync` 约束
- 返回类型应该使用 `Result<T, E>` 来处理错误

### 错误处理

使用 `thiserror` 进行错误处理。**生产代码中绝对不要使用 `unwrap()` 或 `panic!()`**：

**正确做法 - 使用 thiserror**：
```rust
use thiserror::Error;

#[derive(Error, Debug)]
pub enum AgentError {
    #[error("超过最大迭代次数: {0}")]
    MaxIterationsExceeded(usize),

    #[error("工具执行失败: {0}")]
    ToolError(#[from] ToolError),

    #[error("提供者错误: {0}")]
    ProviderError(String),
}

// 使用方式
pub async fn execute(&self) -> Result<String, AgentError> {
    if iterations > max {
        return Err(AgentError::MaxIterationsExceeded(iterations));
    }
    Ok(result)
}
```

**错误做法 - 使用 unwrap**：
```rust
// 不要这样做
let result = some_operation().unwrap();  // 可能会 panic!
let value = option_value.expect("value");  // 可能会 panic!
```

### 代码格式化和检查

提交前，确保代码通过所有检查：

```bash
# 格式化代码
cargo fmt

# 检查常见错误和改进建议
cargo clippy

# 运行 clippy 的严格模式
cargo clippy -- -W clippy::all -W clippy::pedantic

# 自动修复 clippy 警告
cargo clippy --fix
```

### 文档注释

所有公开项都应该有文档注释：

```rust
/// 执行 Agent 循环直到满足停止条件。
///
/// # 参数
/// * `context` - Agent 执行上下文
/// * `max_iterations` - 最大迭代次数
///
/// # 返回值
/// 返回最终的 Agent 响应，或者一个错误
///
/// # 示例
/// ```
/// let response = agent.execute(&context).await?;
/// ```
pub async fn execute(
    &self,
    context: &AgentContext,
    max_iterations: usize,
) -> Result<AgentResponse, AgentError> {
    // 实现
}
```

---

## 项目结构

### Workspace 组织

项目使用 Rust workspace，包含多个 crate：

```
zero/
├── docs/                          # 文档
│   ├── 01-getting-started.md      # 快速开始指南
│   ├── 03-trait-architecture.md   # 架构详解
│   ├── 07-contributing.md         # 本文件
│   └── plans/                     # 实现路线图
│
├── zero-core/                     # 核心库（主要实现）
│   ├── Cargo.toml
│   └── src/
│       ├── lib.rs                 # 库导出
│       ├── message.rs             # 消息类型
│       ├── error.rs               # 错误定义
│       ├── agent/                 # Agent trait 和实现
│       │   ├── trait.rs
│       │   ├── agent_loop.rs
│       │   └── mod.rs
│       ├── tool/                  # Tool trait 和实现
│       │   ├── trait.rs
│       │   ├── dispatcher.rs
│       │   ├── builtins/          # 内置工具（bash、read、write）
│       │   └── mod.rs
│       ├── provider/              # LLM 提供者抽象
│       │   ├── trait.rs
│       │   ├── anthropic.rs       # Anthropic 提供者
│       │   ├── openai.rs          # OpenAI 提供者
│       │   └── mod.rs
│       ├── channel/               # 消息通道 trait
│       │   ├── trait.rs
│       │   └── mod.rs
│       ├── memory/                # 内存系统
│       │   ├── trait.rs
│       │   └── mod.rs
│       ├── hooks/                 # 钩子系统
│       │   ├── trait.rs
│       │   └── manager.rs
│       ├── config/                # 配置管理
│       │   └── mod.rs
│       └── security/              # 安全沙箱
│           └── mod.rs
│
├── zero-cli/                      # 命令行工具
│   ├── Cargo.toml
│   └── src/
│       └── main.rs
│
├── zero-api/                      # REST API 服务器
│   ├── Cargo.toml
│   └── src/
│       └── main.rs
│
├── CLAUDE.md                      # AI 开发者协议
├── DEVELOPER_GUIDE.md             # 快速开始指南
└── README.md                      # 项目说明
```

### 模块组织原则

1. **保持模块专注**：每个模块应该只有一个责任
2. **使用 mod.rs 组织**：在模块文件中分组相关类型
3. **导出公开 API**：使用 `pub use` 在 mod.rs 中控制导出
4. **默认私有**：除非需要公开，否则保持项目私有

示例模块结构：

```rust
// src/agent/mod.rs
pub mod trait;
pub mod agent_loop;
pub mod context;

pub use trait::Agent;
pub use agent_loop::DefaultAgentLoop;
pub use context::AgentContext;
```

### 在哪里添加新功能

- **新 Trait**：在相应的模块目录中创建（例如 `src/agent/new_trait.rs`）
- **新实现**：在子模块中添加（例如 `src/tool/builtins/newtool.rs`）
- **新模块**：创建目录，包含 `mod.rs` 并从父模块导出
- **测试**：保持与代码相邻，使用 `#[cfg(test)]` 模块

---

## Git 工作流

### 创建功能分支

使用描述性的分支名，遵循模式：`<类型>/<功能描述>`

```bash
# 功能分支
git checkout -b feat/agent-loop-implementation

# 修复分支
git checkout -b fix/memory-leak-in-dispatcher

# 文档
git checkout -b docs/update-contributing-guide

# 重构
git checkout -b refactor/tool-trait-simplification
```

### Commit 消息约定

遵循 Conventional Commits 格式，保持清晰的提交历史：

```
<类型>(<范围>): <主题>

<内容>

<页脚>
```

**类型**：
- `feat`: 新功能
- `fix`: 缺陷修复
- `docs`: 文档变更
- `style`: 代码风格变更（格式化、缺少分号等）
- `refactor`: 代码重构（不改变功能）
- `test`: 添加或更新测试
- `chore`: 构建流程、依赖项等变更

**示例**：

```bash
# 简单 commit
git commit -m "feat(agent): 实现 agent 循环执行

- 添加 AgentLoop 结构体和主循环逻辑
- 实现提供者交互
- 添加工具执行集成
- 添加循环行为的单元测试"

# 修复 commit 并引用问题
git commit -m "fix(provider): 正确处理超时错误

之前超时错误没有正确传播。
现在它们被捕获并作为 ProviderError 返回。

Fixes: #123"

# 文档更新
git commit -m "docs: 添加贡献指南

添加全面的贡献文档，包括：
- 开发环境设置
- 代码风格标准
- 测试指南
- Git 工作流"
```

### Pull Request 流程

1. **在提交前确保代码质量**：
   ```bash
   cargo build --release
   cargo clippy
   cargo fmt
   cargo test
   ```

2. **推送你的分支**：
   ```bash
   git push origin feat/your-feature
   ```

3. **创建 pull request** 并提供清晰的描述：
   - 引用任何相关的问题
   - 解释变更的动机
   - 描述测试内容

4. **代码审查流程**：
   - 解决所有审查意见
   - 如需要，推送额外的 commit
   - 修改后重新请求审查

5. **合并要求**：
   - 所有测试必须通过
   - 需要代码审查批准
   - 分支必须与主分支同步

---

## 测试

### 运行测试套件

```bash
# 运行所有测试
cargo test

# 运行并显示输出（用于调试）
cargo test -- --nocapture

# 运行特定模块测试
cargo test agent::
cargo test tool::dispatcher::

# 运行单个测试
cargo test agent::loop_tests::test_basic_execution

# 以发布模式运行测试（更快）
cargo test --release
```

### 编写测试

遵循 Rust 测试约定：

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_agent_execution() {
        // 准备
        let agent = TestAgent::new();
        let context = AgentContext::default();

        // 执行
        let result = agent.execute(&context).await;

        // 断言
        assert!(result.is_ok());
        let response = result.unwrap();
        assert_eq!(response.content, "expected");
    }

    #[test]
    fn test_sync_operation() {
        let value = compute_something();
        assert_eq!(value, 42);
    }
}
```

**关键点**：
- 异步测试使用 `#[tokio::test]`
- 同步测试使用 `#[test]`
- 遵循 Arrange-Act-Assert 模式
- 使用描述性的测试名称
- 测试成功和失败情况

### 覆盖率期望

目标覆盖率：
- **核心 Trait**：80% 以上
- **实现**：70% 以上
- **工具函数**：60% 以上

运行覆盖率：
```bash
# 生成覆盖率报告（需要安装）
cargo tarpaulin --out Html
```

### 测试组织

逻辑地组织测试：

```rust
#[cfg(test)]
mod tests {
    mod agent_loop {
        mod basic {
            #[test]
            fn test_single_iteration() { }
        }

        mod error_handling {
            #[test]
            fn test_max_iterations_exceeded() { }
        }
    }
}
```

---

## Trait 设计指南

### 何时创建新 Trait

在以下情况下创建新 Trait：
- 多种类型需要实现相同接口
- 你想允许不同的测试实现
- 行为可以被替换或扩展

**不好的做法** - 过于具体：
```rust
pub trait SpecificAgent { }  // 只用于一个用例
```

**好的做法** - 可重用：
```rust
pub trait LLMProvider: Send + Sync {
    async fn complete(&self, messages: &[Message]) -> Result<Response>;
}
```

### Trait 组合模式

通过组合更简单的 Trait 来创建可组合的 Trait：

```rust
// 基础 Trait
pub trait Named {
    fn name(&self) -> &str;
}

// 扩展 Trait
#[async_trait]
pub trait Agent: Named + Send + Sync {
    async fn execute(&self) -> Result<AgentResponse>;
}

// 组合 Trait
pub struct MyAgent;

impl Named for MyAgent {
    fn name(&self) -> &str { "MyAgent" }
}

#[async_trait]
impl Agent for MyAgent {
    async fn execute(&self) -> Result<AgentResponse> {
        // 实现
    }
}
```

### 使 Trait 可组合和可扩展

有效地使用 Trait 对象和约束：

```rust
#[async_trait]
pub trait Tool: Send + Sync {
    fn name(&self) -> &str;
    async fn execute(&self, input: &str) -> Result<String>;
}

// 组合 Trait
pub struct ToolDispatcher {
    tools: Vec<Arc<dyn Tool>>,
}

impl ToolDispatcher {
    pub fn register(&mut self, tool: Arc<dyn Tool>) {
        self.tools.push(tool);
    }
}
```

### 自定义 Trait 的文档

始终为 Trait 编写包含示例的文档：

```rust
/// 表示 LLM 提供者接口。
///
/// 实现应该处理 API 调用、重试和错误处理。
///
/// # 示例
/// ```
/// use zero_core::provider::LLMProvider;
///
/// struct MyProvider;
///
/// #[async_trait]
/// impl LLMProvider for MyProvider {
///     async fn complete(&self, messages: &[Message]) -> Result<Response> {
///         // 实现
///         todo!()
///     }
/// }
/// ```
#[async_trait]
pub trait LLMProvider: Send + Sync {
    async fn complete(&self, messages: &[Message]) -> Result<Response>;
}
```

---

## 常见陷阱

### 1. 在生产代码中使用 unwrap()

**问题**：`unwrap()` 在值为 `None` 或错误时会 panic：
```rust
// 不要这样做
let result = operation().unwrap();  // 错误时会 panic!
```

**解决方案**：使用 `Result` 和 `?` 操作符：
```rust
// 正确做法
let result = operation()?;  // 优雅地传播错误
```

### 2. 忘记 async-trait 属性

**问题**：Trait 中的异步方法需要 `async-trait`：
```rust
// 无法编译
pub trait Agent {
    async fn execute(&self) -> Result<()>;  // 错误!
}
```

**解决方案**：添加属性：
```rust
use async_trait::async_trait;

#[async_trait]
pub trait Agent {
    async fn execute(&self) -> Result<()>;  // 编译成功!
}
```

### 3. 缺少 Send + Sync 约束

**问题**：跨线程使用的 Trait 需要这些约束：
```rust
pub trait Agent {  // 异步操作会出错!
    async fn execute(&self) -> Result<()>;
}
```

**解决方案**：添加约束：
```rust
pub trait Agent: Send + Sync {  // 正确!
    async fn execute(&self) -> Result<()>;
}
```

### 4. 性能问题 - 频繁克隆

**问题**：重复克隆昂贵的对象：
```rust
let tool = expensive_tool.clone();  // 在循环中昂贵!
```

**解决方案**：使用引用或 Arc：
```rust
let tool = Arc::new(expensive_tool);
let tool_ref = Arc::clone(&tool);  // 便宜的引用克隆
```

### 5. 在异步代码中阻塞

**问题**：在异步上下文中使用阻塞操作：
```rust
#[tokio::main]
async fn main() {
    let data = std::fs::read_to_string("file.txt").unwrap();  // 阻塞!
}
```

**解决方案**：使用异步 I/O：
```rust
#[tokio::main]
async fn main() {
    let data = tokio::fs::read_to_string("file.txt").await.unwrap();
}
```

### 6. 错误类型不一致

**问题**：不同的错误类型使处理困难：
```rust
fn operation1() -> Result<String, String> { }
fn operation2() -> Result<String, io::Error> { }
// 不能在它们之间使用 ? 操作符!
```

**解决方案**：使用统一的错误枚举：
```rust
#[derive(Error, Debug)]
pub enum AgentError {
    #[error("IO error: {0}")]
    IoError(#[from] io::Error),
    #[error("Custom error: {0}")]
    Custom(String),
}

fn operation1() -> Result<String, AgentError> { }
fn operation2() -> Result<String, AgentError> { }
```

### 7. 提交前没有运行 Cargo Check

**问题**：提交损坏的代码：
```bash
git commit -m "更新代码"  # 可能无法编译!
```

**解决方案**：提交前总是验证：
```bash
cargo build --release  # 必须通过
cargo clippy          # 无警告
cargo fmt             # 代码格式化
cargo test            # 所有测试通过
git commit -m "feat: 更新代码"
```

---

## 最佳实践总结

提交 PR 前，确保：

✓ 代码可以编译：`cargo build --release`
✓ 无 clippy 警告：`cargo clippy`
✓ 已格式化：`cargo fmt`
✓ 所有测试通过：`cargo test`
✓ 文档完整：`cargo doc --open`
✓ Commit 消息清晰
✓ 生产代码中无 unwrap/panic
✓ 使用 thiserror 正确处理错误
✓ 异步 Trait 使用 async-trait
✓ 需要的地方有 Send + Sync 约束

---

## 获取帮助

- **项目问题**：参见 [01-getting-started.zh-CN.md](./01-getting-started.zh-CN.md)
- **架构详解**：参见 [03-trait-architecture.zh-CN.md](./03-trait-architecture.zh-CN.md)
- **API 参考**：参见 [05-api-reference.zh-CN.md](./05-api-reference.zh-CN.md)
- **实现路线图**：参见 `docs/plans/` 目录
- **代码示例**：参见 `examples/` 目录

感谢你为 Zero 项目做出的贡献！
