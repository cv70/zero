# 代码示例

> **返回首页**: [README.zh-CN.md](../README.zh-CN.md)

欢迎来到 Zero Core 示例！这些示例演示了 Agent 框架的关键概念，从基础 Agent 实现到多 Agent 协调。每个示例都是独立可运行的，并逐步建立在之前概念的基础上。

## 概览

示例按进度组织如下：

1. **01-simple-agent** - 学习基础 Agent trait 实现
2. **02-custom-tool** - 实现和使用自定义 Tool
3. **03-multi-agent** - 协调多个 Agent 一起工作

## 示例 1：简单 Agent

### 描述

这个示例演示了最基础的 Agent 实现。你将学到如何：
- 创建实现 `Agent` trait 的 Agent 结构体
- 定义基础 Agent 元数据（名称、描述、系统提示）
- 实现异步 `execute()` 方法
- 创建 agent 上下文并运行 agent

### 核心概念

- **Agent Trait**：框架中任何自主 agent 的核心抽象
- **Agent Context**：携带会话信息和可用工具的上下文
- **AgentResponse**：包含 agent 响应和元数据的输出类型

### 运行示例

```bash
cargo run --example 01-simple-agent
```

### 预期输出

```
Agent Name: GreetingAgent
Agent Description: A simple agent that greets users
System Prompt: You are a friendly greeting agent that welcomes users.

Executing agent...
Agent Response:
  Content: Hello! I'm the GreetingAgent. I'm here to help you.
  Tool Calls: 0

Agent execution completed successfully!
```

### 代码讲解

该示例创建了一个 `GreetingAgent`，实现三个关键方法：

1. **name()** - 返回 agent 的标识符
2. **description()** - 返回可读的描述
3. **execute()** - 执行 agent 逻辑的主异步函数

```rust
#[async_trait]
impl Agent for GreetingAgent {
    fn name(&self) -> &str {
        "GreetingAgent"
    }

    async fn execute(&self, context: &AgentContext) -> Result<AgentResponse, AgentError> {
        Ok(AgentResponse {
            content: "Hello! I'm the GreetingAgent. I'm here to help you.".to_string(),
            tool_calls: Vec::new(),
            metadata: std::collections::HashMap::new(),
        })
    }
}
```

### 学到的核心概念

- 使用 async-trait 的基本 Agent trait 实现
- Agent 生命周期：创建 → 上下文设置 → 执行
- 返回包含元数据的结构化响应
- 使用 AgentError 进行错误处理

## 示例 2：自定义 Tool

### 描述

这个示例展示了如何创建和使用自定义 Tool 实现。你将学到：
- 为自定义工具实现 `Tool` trait
- 使用 JSON schema 定义工具元数据进行输入验证
- 在执行前实现输入验证
- 处理工具执行和错误处理
- 独立测试自定义工具

### 核心概念

- **Tool Trait**：任何工具/功能的抽象接口
- **ToolMetadata**：定义工具接口的名称、描述和 JSON schema
- **ToolContext**：包含会话和环境信息的执行上下文
- **ToolOutput**：支持多种输出格式的结果类型
- **输入验证**：确保工具接收有效输入

### 运行示例

```bash
cargo run --example 02-custom-tool
```

### 预期输出

```
Tool: calculator
Description: Perform basic arithmetic operations (add, subtract, multiply, divide)
Input Schema: { ... }

Running test cases:

Test: 10 + 5
  Result: 10 + 5 = 15

Test: 10 - 3
  Result: 10 - 3 = 7

Test: 6 * 7
  Result: 6 * 7 = 42

Test: 20 / 4
  Result: 20 / 4 = 5

Testing error case (division by zero):
  Expected error: Execution failed: Division by zero

Custom tool example completed!
```

### 代码讲解

该示例实现了一个 `CalculatorTool`，结构如下：

```rust
#[async_trait]
impl Tool for CalculatorTool {
    fn metadata(&self) -> ToolMetadata {
        // 定义工具接口
        ToolMetadata {
            name: "calculator".to_string(),
            description: "Perform basic arithmetic operations".to_string(),
            input_schema: json!({ /* JSON Schema */ }),
        }
    }

    fn validate_input(&self, input: &str) -> Result<(), ToolError> {
        // 可选的执行前验证
        serde_json::from_str::<CalculatorArgs>(input)
            .map_err(|e| ToolError::InvalidInput(format!("Invalid JSON: {}", e)))?;
        Ok(())
    }

    async fn execute(&self, input: &str, ctx: &ToolContext) -> Result<ToolOutput, ToolError> {
        // 解析、验证、执行
        let args: CalculatorArgs = serde_json::from_str(input)?;

        // 执行操作
        let result = match args.operation.as_str() {
            "add" => args.a + args.b,
            // ... 其他操作
        };

        Ok(ToolOutput::text(result.to_string()))
    }
}
```

### 学到的核心概念

- 完整生命周期的 Tool trait 实现
- 用于工具接口的 JSON schema 定义
- 输入验证和错误处理
- 使用 serde 的结构化序列化
- 独立测试工具

## 示例 3：多 Agent 协调

### 描述

这个示例演示了如何创建和协调多个 Agent 一起工作。你将学到：
- 创建具有不同职责的多个 agent
- 构建 Agent 协调器来管理多个 agent
- 使用共享上下文顺序执行 agent
- 收集和汇总多个 agent 的结果
- 设计多 agent 工作流

### 核心概念

- **Agent 多样性**：不同 agent 专门处理不同任务
- **协调**：管理执行顺序和信息流
- **共享上下文**：在 agent 之间传递信息
- **结果聚合**：收集和汇总 agent 输出

### 运行示例

```bash
cargo run --example 03-multi-agent
```

### 预期输出

```
=== Multi-Agent Coordination Example ===

Registered agents:
  - ResearchAgent: Gathers and researches information on given topics
  - AnalysisAgent: Analyzes information and draws conclusions
  - ReportAgent: Generates comprehensive reports from analyzed data

Executing agents in coordination...

Executing agent: ResearchAgent
  Result: Research complete on topic...
Executing agent: AnalysisAgent
  Result: Analysis complete...
Executing agent: ReportAgent
  Result: Executive Summary...

=== Execution Summary ===
Total agents executed: 3

Agent 1 Response:
  Content: Research complete...
  Tool Calls: 0
  Metadata:
    research_items: 3
    confidence: high
...
```

### 代码讲解

该示例创建了三个专门的 agent：

1. **ResearchAgent** - 收集信息
2. **AnalysisAgent** - 分析发现
3. **ReportAgent** - 生成最终报告

以及一个 `AgentCoordinator` 来管理它们：

```rust
pub struct AgentCoordinator {
    agents: Vec<(String, Box<dyn Agent>)>,
}

impl AgentCoordinator {
    pub async fn execute_all(&self, context: &AgentContext)
        -> Result<Vec<AgentResponse>, AgentError> {
        let mut responses = Vec::new();

        for (name, agent) in &self.agents {
            let response = agent.execute(context).await?;
            responses.push(response);
        }

        Ok(responses)
    }
}
```

### 学到的核心概念

- 创建具有不同特性的多个 agent
- Agent 组合和协调模式
- 顺序 vs 并行执行策略
- 结果聚合和汇总
- 设计可复用的协调器组件

## 运行所有示例

编译所有示例而不运行：

```bash
cargo build --examples
```

运行特定示例：

```bash
cargo run --example 01-simple-agent
cargo run --example 02-custom-tool
cargo run --example 03-multi-agent
```

## 后续步骤

探索完这些示例后，你已准备好：

1. **阅读 API 文档**：查看 `docs/01-getting-started.md` 了解详细 API 信息
2. **探索高级功能**：查看 `docs/02-core-concepts.md` 了解高级 agent 执行模式
3. **实现自己的 Agent**：使用这些示例作为自定义 agent 和工具的模板
4. **构建多 Agent 系统**：组合多个 agent 解决复杂问题

## 扩展示例的建议

### 创建自己的 Agent

```rust
#[derive(Debug, Clone)]
pub struct MyCustomAgent;

#[async_trait]
impl Agent for MyCustomAgent {
    fn name(&self) -> &str { "MyAgent" }

    async fn execute(&self, context: &AgentContext) -> Result<AgentResponse, AgentError> {
        // 你的实现
        Ok(AgentResponse {
            content: "Response".to_string(),
            tool_calls: Vec::new(),
            metadata: HashMap::new(),
        })
    }
}
```

### 创建自己的 Tool

```rust
#[derive(Debug, Clone)]
pub struct MyCustomTool;

#[async_trait]
impl Tool for MyCustomTool {
    fn metadata(&self) -> ToolMetadata {
        ToolMetadata {
            name: "my_tool".to_string(),
            description: "Description".to_string(),
            input_schema: json!({ /* schema */ }),
        }
    }

    async fn execute(&self, input: &str, ctx: &ToolContext) -> Result<ToolOutput, ToolError> {
        // 你的实现
        Ok(ToolOutput::text("result".to_string()))
    }
}
```

## 故障排除

### 示例无法编译
- 确保使用 Rust 1.85 或更高版本（用 `rustc --version` 检查）
- 如果遇到构建问题，运行 `cargo clean` 后重新构建
- 检查是否在项目根目录中

### 示例执行时 Panic
- 检查错误消息以了解所需的设置（环境变量、文件路径）
- 确保所有工作区依赖都已正确配置
- 查看示例代码注释了解上下文要求

### Tool 执行失败
- 验证输入 JSON 与 `metadata()` 中定义的 schema 匹配
- 检查错误消息了解验证失败信息
- 使用 `RUST_LOG=debug` 启用日志获取更多详情

## 其他资源

- **入门指南**：查看 `docs/01-getting-started.md`
- **核心概念**：查看 `docs/02-core-concepts.md`
- **Trait 架构**：查看 `docs/03-trait-architecture.md`
- **项目架构**：查看 `ARCHITECTURE.md`
- **API 文档**：运行 `cargo doc --open` 浏览生成的文档

## 贡献示例

如果你想贡献新的示例：

1. 创建一个新文件，遵循命名约定：`NN-description.rs`
2. 添加内联文档说明示例
3. 确保示例可以成功编译和运行
4. 在本文件中记录示例
5. 提交 pull request

祝你编码愉快！探索这些示例来理解 Zero 框架并构建强大的多 agent 系统。
