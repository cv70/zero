# Zero Project - Architecture Design Document

## 项目简介

Zero 是一个用 Rust 实现的完整 Agent 系统，基于 Trait 驱动架构。它分 12 个阶段逐步构建，从基础的 Agent Loop 到支持多 Agent 团队协作、自主任务管理和工作树隔离的生产级系统。

该项目是 learn-claude-code 的 Rust 高性能版本，保留了教学价值，同时获得了 Rust 的类型安全和性能优势。

---

## 核心设计原则

### 1. Trait 驱动（Trait-First Design）

**原则**：所有核心能力通过 Trait 定义，而不是具体实现。

**优势**：
- 高度可插拔：可以轻松替换 LLM 提供商、存储后端、通道实现
- 便于测试：易于创建 Mock 实现
- 清晰的接口契约：Trait 即文档

**核心 Trait**：
```
┌─────────────────────────────────────────┐
│         Core Agent System               │
├─────────────────────────────────────────┤
│ Agent          Provider       Channel   │
│ Tool           Memory                   │
│ ToolDispatcher Coordinator              │
└─────────────────────────────────────────┘
```

### 2. 异步优先（Async-First）

**原则**：所有 I/O 操作都是异步的，使用 `tokio` 运行时和 `async-trait`。

**实践**：
```rust
#[async_trait]
pub trait Agent: Send + Sync {
    async fn execute(&self, ...) -> Result<...>;
}
```

**好处**：
- 单线程内高效并发
- 零成本抽象
- 天然支持多 Agent 并行执行

### 3. 逐层构建（Progressive Layering）

**原则**：每个阶段（S1-S12）只添加一种机制，不修改底层循环。

**架构**：
```
Layer 4: Team Coordination (S9-S12)
    ↓
Layer 3: Task Persistence (S7-S8)
    ↓
Layer 2: Planning & Knowledge (S3-S6)
    ↓
Layer 1: Core Loop (S1-S2)
```

**好处**：
- 可验证的进度
- 出现问题易于定位
- 便于理解和学习

### 4. 错误处理规范

**原则**：使用 `thiserror` 统一错误处理，避免 unwrap 和 panic。

```rust
#[derive(thiserror::Error, Debug)]
pub enum AgentError {
    #[error("Max iterations exceeded: {0}")]
    MaxIterationsExceeded(usize),

    #[error("Provider error: {0}")]
    ProviderError(String),

    #[error("Tool error: {0}")]
    ToolError(#[from] ToolError),

    // ...
}
```

### 5. 钩子系统（Hook System）

**原则**：支持在关键点插入钩子，实现可观测性和可扩展性。

```
Before Provider Call → Execute Tools → After Tool Execution
      ↑                                        ↓
   HookManager                          Fire Hooks
```

**类型**：
- AgentHook - Agent 执行
- ToolHook - 工具调用
- ChannelHook - 消息发送
- ProviderHook - LLM 调用
- MemoryHook - 内存访问

---

## 系统架构

### 顶层架构

```
┌──────────────────────────────────────────────────────┐
│                  Application Layer                   │
│  (User CLI, Web API, IDE Integration)                │
├──────────────────────────────────────────────────────┤
│              Runtime & Coordination Layer             │
│  AgentLoop  TaskManager  TeamCoordinator              │
├──────────────────────────────────────────────────────┤
│                   Core Trait Layer                    │
│  Agent  Tool  Provider  Channel  Memory               │
├──────────────────────────────────────────────────────┤
│           Implementation & Extension Layer            │
│  Anthropic  OpenAI  Slack  Discord  Filesystem       │
├──────────────────────────────────────────────────────┤
│              Infrastructure & Support                │
│  Hooks  Config  Security  Monitoring                 │
└──────────────────────────────────────────────────────┘
```

### 模块依赖关系

```
lib.rs
├── message           (基础消息类型)
├── error             (统一错误类型)
├── agent
│   ├── trait         (Agent Trait)
│   ├── loop          (Agent 循环实现)
│   ├── context       (Agent 上下文)
│   └── hook          (Agent 钩子)
├── tool
│   ├── trait         (Tool Trait)
│   ├── registry      (工具注册表)
│   ├── dispatcher    (工具调度器)
│   └── builtins      (内置工具)
├── provider
│   ├── trait         (Provider Trait)
│   ├── anthropic     (Anthropic 实现)
│   ├── openai        (OpenAI 实现)
│   └── router        (路由和故障转移)
├── channel
│   ├── trait         (Channel Trait)
│   ├── slack         (Slack 实现)
│   └── ...
├── memory
│   ├── trait         (Memory Trait)
│   ├── backend       (存储后端)
│   └── search        (搜索能力)
├── planning          (规划系统 S3-S6)
├── task              (任务系统 S7-S8)
├── team              (团队协调 S9-S12)
├── config            (配置管理)
├── hooks             (钩子管理)
└── security          (安全沙箱)
```

---

## 关键数据流

### S1-S2: 基础循环和工具调度

```
User Input
    ↓
AgentLoop.execute()
    ├─ Fire: BeforeProviderCall
    ↓
Provider.complete(messages, tools)
    ├─ LLM API 调用
    ↓
Parse Response (Stop Reason)
    ├─ "tool_use"? → 执行工具
    │   ├─ Fire: BeforeToolExecution
    │   ↓
    │   ToolDispatcher.execute(ToolCall)
    │   ├─ Registry 查找
    │   ├─ 参数验证
    │   ├─ 执行工具
    │   ├─ Fire: AfterToolExecution
    │   ↓
    │   Append ToolResult to messages
    │   ↓
    │   Loop back to Provider.complete()
    │
    └─ "end_turn"? → 返回响应文本
        ↓
    Final Response
```

### S7-S9: 任务和团队

```
New Task
    ↓
TaskManager.create()
    ├─ 分配唯一 ID
    ├─ 存储到 JSONL
    └─ 触发依赖图更新
    ↓
TeamCoordinator.distribute()
    ├─ 遍历 Workers
    ├─ 将任务推送到 Mailbox
    ↓
Worker (Autonomous)
    ├─ Poll Mailbox
    ├─ Claim Task
    ├─ Execute (S1-S2 Loop)
    ├─ Mark Complete
    ├─ Notify Dependents
    ↓
Task Dependencies Resolved
    ├─ Unblock dependent tasks
    ↓
Completion
```

---

## 12 个阶段总体视图

### Phase 1: 基础循环 (S1-S2)

**关键点**：
- 实现 Agent 循环的核心逻辑
- 支持多工具调度和执行
- **核心贡献**：可以通过工具执行任务的 Agent

### Phase 2: 规划与知识 (S3-S6)

**关键点**：
- 引入规划系统：Agent 先列出步骤再执行
- 子 Agent：为复杂任务创建专门的子 Agent
- 技能加载：动态注入背景知识
- 上下文压缩：支持无限会话
- **核心贡献**：智能的、可以自我规划和学习的 Agent

### Phase 3: 持久化 (S7-S8)

**关键点**：
- 任务系统：将工作分解为可追踪的任务
- 依赖图：任务间的顺序约束
- 后台执行：无需等待的异步任务
- **核心贡献**：可以持久化和恢复的工作流

### Phase 4: 团队协作 (S9-S12)

**关键点**：
- 多 Agent 协调：lead agent + worker agents
- 通信协议：JSONL 邮箱
- 自主执行：agents 自动轮询和认领任务
- 工作树隔离：每个任务独立的工作目录
- **核心贡献**：可扩展的多 Agent 系统

---

## 关键设计决策

### 1. 消息格式选择

**决策**：采用结构化的 Rust 枚举，而不是字符串

```rust
pub enum Message {
    User { content: String },
    Assistant { content: Vec<ContentBlock> },
    ToolResult { tool_use_id: String, content: String },
}

pub enum ContentBlock {
    Text { text: String },
    ToolUse { id: String, name: String, input: Value },
}
```

**原因**：
- 类型安全：编译时检查
- 模式匹配：易于处理
- 可序列化：可存储和传输

### 2. Tool 执行模型

**决策**：异步调度 + 超时控制

```rust
let result = tokio::time::timeout(
    Duration::from_secs(config.tool_timeout),
    dispatcher.execute(tool_call),
).await?;
```

**原因**：
- 防止工具卡住整个循环
- 可配置的超时时间
- 优雅的错误处理

### 3. 持久化存储格式

**决策**：JSONL（换行分隔的 JSON）

```
{"id":"1","status":"done","result":"..."}
{"id":"2","status":"pending","deps":["1"]}
```

**原因**：
- 流式友好：可逐行读取
- 人类可读：调试方便
- 简单解析：无需数据库
- 版本控制：易于 git diff

### 4. 团队通信协议

**决策**：异步邮箱 + 请求/响应模式

```rust
pub enum Message {
    Request { task: Task, from: AgentId },
    Response { result: TaskResult, to: AgentId },
    Notify { event: Event },
}
```

**原因**：
- 解耦合：agents 可独立速率运行
- 容错：邮件可重试
- 可观测：便于审计和监控

---

## 并发和性能模型

### 并发层级

```
Level 1: Agent 循环                (单个 tokio 任务)
    ↓
Level 2: 工具并行执行              (可配置 max_concurrent_tools)
    ↓
Level 3: 多 Agent 并发              (每个 agent = 一个 tokio 任务)
    ↓
Level 4: 团队级别                   (Lead + N Workers)
```

### 性能特点

- **内存高效**：无堆积内存，仅保持活跃消息
- **CPU 高效**：IO 阻塞时不消耗 CPU
- **可扩展**：支持数百个并发 agents
- **可测量**：内置性能监控钩子

---

## 安全考虑

### 1. 工具执行沙箱

```rust
// 黑名单危险命令
let dangerous = ["rm -rf /", "sudo", "shutdown"];
if dangerous.iter().any(|d| command.contains(d)) {
    return Err("Blocked");
}
```

### 2. 路径隔离

```rust
// 确保文件访问不逃逸工作目录
fn safe_path(path: &str) -> Result<PathBuf> {
    let resolved = (root / path).canonicalize()?;
    if !resolved.starts_with(&root) {
        return Err("Path escapes workspace");
    }
    Ok(resolved)
}
```

### 3. LLM 调用限制

```rust
// 限制单个 Agent 的 LLM 调用次数
let max_iterations = 100;
let max_tokens_per_call = 8000;
```

### 4. 权限模型（S10+）

- 任务级权限
- Agent 能力声明
- 审计日志

---

## 测试策略

### 单元测试
- 每个 Trait 的 Mock 实现
- 工具安全检查
- 错误恢复逻辑

### 集成测试
- 完整的循环流程
- 多工具协调
- 错误传播

### E2E 测试
- 真实的 LLM 集成
- 实际工具执行
- 性能基准

---

## 文档和学习路径

### 快速入门
1. 理解 Agent Loop 的核心：`docs/S1-The-Agent-Loop.md`
2. 学习工具系统：`docs/S2-Tool-Use.md`
3. 运行最小示例：`examples/minimal_agent.rs`

### 深度学习
1. 研究参考实现：学习 learn-claude-code 的 Python 版本
2. 跟踪代码变更：每个 PR 对应一个 S 阶段
3. 实现自定义 Tool 和 Channel

### 生产部署
1. 配置多个 Provider（故障转移）
2. 启用所有钩子（可观测性）
3. 设置 Channel 集成（协作）
4. 配置任务持久化（容错）

---

## 下一步（未来规划）

### 短期（3 个月）
- 完成 S1-S6：基础功能
- CLI 工具初版
- 基础文档

### 中期（6 个月）
- 完成 S7-S12：完整系统
- Web UI
- 扩展文档和教程

### 长期（1 年+）
- 企业级功能（权限、审计）
- 分布式支持
- IDE 集成
- 生态工具链

---

## 社区和贡献

### 学习资源
- 完整的代码注释
- 每个阶段的教学文档
- 配套的视频讲解（计划）

### 参与方式
- GitHub Issues：报告 bug 或建议功能
- PR：贡献代码实现
- Discussions：讨论设计和架构

---

## 许可和归属

基于 learn-claude-code 的架构参考。
Zero 采用 MIT 或 Apache 2.0 许可（待定）。

---

## 快速参考

### 核心概念速查表

| 概念 | 说明 | 位置 |
|------|------|------|
| Message | 消息类型，包含 User/Assistant/ToolResult | `src/message.rs` |
| Agent | 执行 Agent 的入口 Trait | `src/agent/trait.rs` |
| Tool | 工具抽象，执行具体操作 | `src/tool/trait.rs` |
| Provider | LLM 提供商，调用模型 API | `src/provider/trait.rs` |
| Channel | 消息通道，发送/接收通知 | `src/channel/trait.rs` |
| AgentLoop | 执行循环逻辑 | `src/agent/agent_loop.rs` |
| ToolDispatcher | 工具调度和执行 | `src/tool/dispatcher.rs` |
| TaskManager | 任务持久化和管理（S7+） | `src/task/manager.rs` |
| TeamCoordinator | 多 Agent 协调（S9+） | `src/team/coordinator.rs` |

### 常见任务

```rust
// 创建一个简单的 Agent
let provider = Arc::new(AnthropicProvider::new(api_key));
let tools = Arc::new(ToolRegistry::new());
let loop_impl = DefaultAgentLoop::new(provider, tools);

// 执行 Agent
let mut messages = vec![Message::user("Hello")];
let response = loop_impl.execute(&mut messages, &config).await?;

// 注册自定义工具
tools.register("my_tool", Arc::new(MyTool::new())).await;
```

