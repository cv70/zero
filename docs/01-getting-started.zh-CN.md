# Zero 快速开始

> **返回首页**: [README.zh-CN.md](../README.zh-CN.md)

欢迎使用 Zero 框架！本指南将帮助你在 5 分钟内设置项目并运行第一个程序。

## 目录

- [系统要求](#系统要求)
- [安装步骤](#安装步骤)
- [第一个程序](#第一个程序)
- [快速命令参考](#快速命令参考)
- [常见问题排查](#常见问题排查)
- [下一步](#下一步)

## 系统要求

开始前，请确保系统满足以下要求：

### 操作系统
- **macOS**: 10.13 或更新版本（Intel 或 Apple Silicon）
- **Linux**: Ubuntu 18.04+、Debian 10+ 或同等版本
- **Windows**: Windows 10 或更新版本（推荐使用 WSL2）

### Rust 环境
- **Rust 1.70+**: 支持 2024 edition 所需
  - 通过 [rustup](https://rustup.rs/) 安装：`curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh`
  - 验证安装：`rustc --version`（应显示 1.70 或更新版本）

### 其他工具
- **Cargo**: Rust 自带的包管理工具
- **Git**: 克隆仓库所需
- **4GB+ 内存**: 用于编译
- **2GB+ 磁盘空间**: 用于构建产物

### 验证安装

```bash
# 检查 Rust 版本
rustc --version  # 应输出：rustc 1.70.0 或更新版本

# 检查 Cargo 版本
cargo --version  # 应输出：cargo 1.70.0 或更新版本
```

如果你看到版本号，那就可以继续了！

## 安装步骤

### 第一步：克隆仓库

```bash
git clone https://github.com/yourusername/zero.git
cd zero
```

### 第二步：构建项目

导航到项目主目录并构建：

```bash
cd /path/to/zero
cargo build --release
```

首次构建可能需要几分钟，因为需要编译所有依赖。你应该看到如下输出：

```
   Compiling zero-core v0.1.0
   Compiling zero-cli v0.1.0
   Compiling zero-api v0.1.0
    Finished release [optimized] target(s) in 123.45s
```

### 第三步：验证安装

```bash
# 运行测试套件
cargo test --lib

# 预期输出显示测试通过
running 15 tests
test result: ok. 15 passed

# 生成并查看文档
cargo doc --open
```

如果所有测试都通过且文档生成成功，说明安装完成！

## 第一个程序

让我们创建一个简单的 Agent，展示 Zero 框架的核心概念。

### 创建 Rust 文件

创建 `examples/hello_agent.rs`：

```rust
use async_trait::async_trait;
use serde_json::json;
use std::sync::Arc;

// 定义简单的消息结构
#[derive(Debug, Clone)]
pub struct SimpleMessage {
    pub role: String,
    pub content: String,
}

// 定义最小化的 LLM 提供者
pub struct MockProvider;

#[async_trait]
pub trait LLMProvider: Send + Sync {
    async fn call(&self, messages: Vec<SimpleMessage>) -> Result<String, Box<dyn std::error::Error>>;
}

#[async_trait]
impl LLMProvider for MockProvider {
    async fn call(&self, _messages: Vec<SimpleMessage>) -> Result<String, Box<dyn std::error::Error>> {
        Ok("Hello from Zero! I'm running successfully.".to_string())
    }
}

// 定义简单的 Agent
pub struct SimpleAgent {
    provider: Arc<dyn LLMProvider>,
}

impl SimpleAgent {
    pub fn new(provider: Arc<dyn LLMProvider>) -> Self {
        Self { provider }
    }

    pub async fn run(&self, prompt: &str) -> Result<String, Box<dyn std::error::Error>> {
        let messages = vec![SimpleMessage {
            role: "user".to_string(),
            content: prompt.to_string(),
        }];

        self.provider.call(messages).await
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 创建带有 Mock 提供者的 Agent
    let provider = Arc::new(MockProvider);
    let agent = SimpleAgent::new(provider);

    // 运行 Agent
    let response = agent.run("What can you do?").await?;
    println!("Agent response: {}", response);

    println!("\nCongratulations! Your first Zero agent is running!");
    Ok(())
}
```

### 运行你的第一个程序

```bash
cd /path/to/zero
cargo run --example hello_agent
```

预期输出：
```
Agent response: Hello from Zero! I'm running successfully.

Congratulations! Your first Zero agent is running!
```

### 发生了什么？

1. **Provider（提供者）**: `MockProvider` 模拟一个 LLM（语言模型）
2. **Agent（代理）**: `SimpleAgent` 协调与提供者的通信
3. **Messages（消息）**: 框架在组件间传递结构化消息
4. **Async/Await**: 所有内容使用 `tokio` 异步运行

这演示了驱动 Zero 的核心 Trait 驱动架构！

## 快速命令参考

以下是开发过程中最常用的命令：

### 构建和测试

```bash
# 以调试模式构建（更快，二进制文件较大）
cargo build

# 以发布模式构建（较慢，优化过）
cargo build --release

# 运行所有测试
cargo test --lib

# 运行特定模块的测试
cargo test agent:: --lib
cargo test tool:: --lib
cargo test task:: --lib

# 运行测试并显示输出
cargo test -- --nocapture
```

### 运行代码

```bash
# 运行 hello_agent 示例
cargo run --example hello_agent

# 运行特定二进制文件
cargo run -p zero-cli -- [参数]
cargo run -p zero-api

# 使用发布优化运行
cargo run --release --example hello_agent
```

### 文档和探索

```bash
# 生成并打开 API 文档
cargo doc --open

# 检查编译问题而不构建
cargo check

# 查看警告和最佳实践
cargo clippy

# 将代码格式化为项目风格
cargo fmt
```

### 开发工作流

```bash
# 检查代码是否可以编译
cargo check

# 修复格式问题
cargo fmt --check  # 仅检查
cargo fmt          # 修复

# 提交前
cargo build        # 必须成功
cargo test --lib   # 所有测试必须通过
```

## 常见问题排查

### Rust 版本太旧

**问题**: `error: package requires rustc 1.70 or newer`

**解决方案**:
```bash
# 更新 Rust 到最新稳定版本
rustup update stable

# 设置默认工具链
rustup default stable

# 验证版本
rustc --version
```

### 编译错误

**问题**: `error: could not compile 'zero-core'`

**解决方案**:
```bash
# 清理构建缓存
cargo clean

# 重新构建
cargo build

# 如果仍然失败，检查 Rust 是否完全更新
rustup update
cargo update
```

### 测试失败

**问题**: 安装后某些测试失败

**解决方案**:
```bash
# 以详细输出运行测试
cargo test --lib -- --nocapture

# 运行特定的失败测试
cargo test <test_name> -- --nocapture --test-threads=1
```

### 端口已被使用

**问题**: `error: bind error, address already in use`

**解决方案**:
```bash
# 查找使用该端口的进程（macOS/Linux）
lsof -i :8080

# 杀死该进程
kill -9 <PID>

# 或使用不同的端口
export PORT=8081
cargo run
```

### 构建期间内存不足

**问题**: `error: could not compile due to previous error`

**解决方案**:
```bash
# 使用单线程编译
cargo build -j 1

# 或增加交换空间（Linux）:
# 按照你的操作系统文档添加交换文件
```

### Cargo 未找到

**问题**: `command not found: cargo`

**解决方案**:
```bash
# 确保 Rust 正确安装
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# 设置环境（如需要）
source $HOME/.cargo/env

# 验证安装
cargo --version
```

## 下一步

恭喜！你已经成功设置了 Zero 并运行了第一个 Agent 程序。以下是后续的学习路径：

### 继续学习

1. **核心概念**: 阅读 [02-core-concepts.zh-CN.md](./02-core-concepts.zh-CN.md) 了解 Agent、Tool、Task 和 Planning
2. **架构详情**: 查看 [03-trait-architecture.zh-CN.md](./03-trait-architecture.zh-CN.md) 学习 Trait 驱动设计
3. **示例代码**: 浏览 [04-examples.zh-CN.md](./04-examples.zh-CN.md) 了解实际代码样本
4. **API 参考**: 检查 [05-api-reference.zh-CN.md](./05-api-reference.zh-CN.md) 获取详细 API 文档

### 构建你自己的应用

- 通过实现 `Tool` Trait 创建自定义工具
- 构建具有特定能力的 Agent
- 使用 Hook 扩展框架（见 [06-hooks-system.zh-CN.md](./06-hooks-system.zh-CN.md)）
- 使用 `TeamCoordinator` 协调多个 Agent

### 参与项目

- 查看 [07-contributing.zh-CN.md](./07-contributing.zh-CN.md) 了解贡献指南
- 审查 `ARCHITECTURE.md` 中的架构
- 浏览测试套件了解实现模式

**准备好深入学习了吗？从核心概念指南开始！**
