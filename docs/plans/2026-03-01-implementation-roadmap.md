# Zero Project - Agent System Implementation Roadmap

## 项目愿景

在 Rust 中实现一个完整的 Agent 系统，从基础的 Agent Loop 逐步扩展到支持多 Agent 团队协作、自主任务管理和工作树隔离的生产级系统。

架构基于 Trait 驱动设计，与 learn-claude-code 的 Python 参考实现对应，但采用 Rust 的类型安全和性能优势。

---

## 架构核心层次

### 第 1 层：基础能力（已部分完成）

```
Core Traits
├── Agent         (Agent 工厂和执行)
├── Tool          (统一工具抽象)
├── Provider      (LLM 模型提供)
├── Channel       (消息通道)
├── Memory        (全局共享记忆)
└── Hook System   (事件钩子系统)
```

### 第 2 层：执行引擎

```
Execution Layer
├── Message Loop      (Agent 循环核心)
├── Tool Dispatcher   (工具调度和执行)
├── Response Handler  (响应处理)
└── Error Recovery    (错误恢复)
```

### 第 3 层：业务能力

```
Feature Layer
├── Planning          (任务规划)
├── Task Management   (任务持久化和依赖)
├── Background Jobs   (后台任务)
├── Team Coordination (多 Agent 协作)
└── Worktree Manager  (工作树隔离)
```

---

## 12 阶段实现计划

### **Phase 1: 基础循环 (现在 → 第 2 周)**

#### S1: Agent Loop Core
- ✅ 基础 Trait 定义
- [ ] Message 结构体和序列化
- [ ] 完整的循环实现
- [ ] 测试框架

**关键输出**：
```rust
loop {
    response = provider.complete(messages, tools);
    messages.push(assistant_response);
    if response.stop_reason != "tool_use" { break; }
    results = execute_tools(response);
    messages.push(tool_results);
}
```

#### S2: Tool Dispatch
- [ ] Tool Registry 完成
- [ ] Tool Handler 映射
- [ ] 参数验证
- [ ] 结果格式化

**关键输出**：
```
Tool Registry
├── bash: Bash -> Result
├── read_file: FileOp -> Result
├── write_file: FileOp -> Result
└── edit_file: EditOp -> Result
```

---

### **Phase 2: 规划与知识 (第 3-4 周)**

#### S3: Planning System
- [ ] Todo 数据结构
- [ ] 规划 Agent
- [ ] 步骤验证和追踪
- [ ] 完成度计算

**关键输出**：待办任务列表与验证

#### S4: Subagent System
- [ ] Subagent 上下文隔离
- [ ] 独立消息链
- [ ] 结果聚合
- [ ] 错误传播

**关键输出**：
```
Main Agent
├── Subagent1 (isolated messages[])
├── Subagent2 (isolated messages[])
└── Subagent3 (isolated messages[])
```

#### S5: Skill Loading
- [ ] SKILL.md 格式定义
- [ ] 动态技能加载
- [ ] 技能缓存
- [ ] 版本管理

**关键输出**：技能注入机制

#### S6: Context Compression
- [ ] 3 层压缩策略
  - 层 1：摘要关键信息
  - 层 2：压缩旧消息
  - 层 3：存档和检索
- [ ] 压缩触发条件
- [ ] 恢复机制

**关键输出**：无限会话支持

---

### **Phase 3: 持久化 (第 5-6 周)**

#### S7: Task System
- [ ] Task 数据模型
- [ ] JSONL 存储格式
- [ ] 依赖图管理
- [ ] 任务状态机

**关键输出**：
```
Task {
  id: String,
  status: Pending|Running|Done|Error,
  dependencies: Vec<String>,
  result: Option<TaskResult>,
}
```

#### S8: Background Jobs
- [ ] 后台任务队列
- [ ] 异步执行
- [ ] 完成通知
- [ ] 错误重试

**关键输出**：
```
async fn execute_background_job() {
    notify_queue.push(JobComplete {
        task_id: "...",
        result: "...",
    });
}
```

---

### **Phase 4: 团队协作 (第 7-8 周)**

#### S9: Agent Teams
- [ ] 团队成员注册
- [ ] JSONL 邮箱协议
- [ ] 异步消息队列
- [ ] 成员生命周期

**关键输出**：
```
Team
├── Lead Agent
├── Worker 1 (inbox: JSONL)
├── Worker 2 (inbox: JSONL)
└── Worker 3 (inbox: JSONL)
```

#### S10: Team Protocols
- [ ] 请求/响应协议
- [ ] 消息格式标准化
- [ ] 关闭协议
- [ ] 计划批准 FSM

**关键输出**：统一的通信规范

#### S11: Autonomous Agents
- [ ] 心跳循环
- [ ] 自动任务认领
- [ ] 空闲策略
- [ ] 优雅关闭

**关键输出**：
```
every 30s:
    if idle && has_unclaimed_task:
        claim_and_execute()
```

#### S12: Worktree Isolation
- [ ] 工作树管理器
- [ ] 目录隔离
- [ ] 任务绑定
- [ ] 清理策略

**关键输出**：
```
Task { id, ..., worktree: PathBuf }
Worker { ..., cwd: worktree }
```

---

## 模块布局

```
zero-core/src/
├── lib.rs                      (主入口)
│
├── agent/
│   ├── mod.rs                 (模块聚合)
│   ├── trait.rs               (✅ Agent Trait 定义)
│   ├── loop.rs                (S1: 核心循环)
│   ├── context.rs             (✅ AgentContext)
│   ├── hooked_agent.rs        (✅ 钩子支持)
│   └── hook.rs                (✅ Hook 定义)
│
├── tool/
│   ├── mod.rs                 (模块聚合)
│   ├── trait.rs               (✅ Tool Trait)
│   ├── registry.rs            (✅ 工具注册)
│   ├── dispatcher.rs          (S2: 调度器)
│   ├── handler.rs             (S2: 处理器)
│   ├── metadata.rs            (✅ 元数据)
│   ├── hook.rs                (✅ Hook)
│   └── built_in/              (S1-S2: 内置工具)
│       ├── bash.rs
│       ├── file.rs
│       └── ...
│
├── provider/
│   ├── mod.rs                 (模块聚合)
│   ├── trait.rs               (✅ Provider Trait)
│   ├── anthropic.rs           (✅ Anthropic)
│   ├── openai.rs              (✅ OpenAI)
│   ├── ollama.rs              (✅ Ollama)
│   ├── routing.rs             (✅ 路由)
│   ├── health.rs              (✅ 健康检查)
│   ├── hook.rs                (✅ Hook)
│   └── adapter.rs             (✅ 适配器)
│
├── channel/
│   ├── mod.rs                 (模块聚合)
│   ├── trait.rs               (✅ Channel Trait)
│   ├── slack.rs               (✅ Slack)
│   ├── discord.rs             (✅ Discord)
│   ├── telegram.rs            (✅ Telegram)
│   ├── email.rs               (✅ Email)
│   ├── matrix.rs              (✅ Matrix)
│   ├── webhook.rs             (✅ Webhook)
│   ├── registry.rs            (✅ 注册)
│   ├── hook.rs                (✅ Hook)
│   ├── queue.rs               (S8: 队列)
│   ├── persistence.rs         (S8: 持久化)
│   └── analytics.rs           (✅ 分析)
│
├── memory/
│   ├── mod.rs                 (模块聚合)
│   ├── memory.rs              (✅ 内存)
│   ├── hook.rs                (✅ Hook)
│   ├── search.rs              (✅ 搜索)
│   └── backend/
│       ├── mod.rs
│       └── filesystem.rs      (✅ 文件系统)
│
├── config/
│   ├── mod.rs                 (模块聚合)
│   ├── loader.rs              (✅ 加载器)
│   ├── validator.rs           (✅ 验证)
│   ├── hooks.rs               (✅ Hook)
│   └── models.rs              (配置模型)
│
├── hooks/
│   ├── mod.rs                 (✅ 模块聚合)
│   └── manager.rs             (✅ HookManager)
│
├── planning/                   (S3-S5)
│   ├── mod.rs
│   ├── todo.rs                (TodoList, Todo)
│   ├── planner.rs             (PlanningAgent)
│   ├── skill_loader.rs        (SkillLoader - S5)
│   └── context_compressor.rs  (ContextCompressor - S6)
│
├── task/                       (S7-S8)
│   ├── mod.rs
│   ├── model.rs               (Task 数据模型)
│   ├── manager.rs             (TaskManager)
│   ├── store.rs               (JSONL 存储)
│   ├── dependency.rs          (依赖图)
│   └── scheduler.rs           (✅ 调度)
│
├── team/                       (S9-S12)
│   ├── mod.rs
│   ├── coordinator.rs         (✅ TeamCoordinator)
│   ├── protocol.rs            (S10: 通信协议)
│   ├── mailbox.rs             (S9: JSONL 邮箱)
│   ├── autonomy.rs            (S11: 自主逻辑)
│   └── worktree.rs            (S12: 工作树)
│
├── subagent/                   (S4)
│   ├── mod.rs
│   └── context.rs             (独立上下文)
│
├── runtime/                    (✅ 运行时)
│   └── mod.rs
│
├── integration/                (✅ 集成)
│   ├── mod.rs
│   ├── bus.rs
│   ├── envelope.rs
│   └── adapters.rs
│
├── security/                   (✅ 安全)
│   ├── mod.rs
│   ├── sandbox.rs
│   ├── scanner.rs
│   └── validator.rs
│
├── perf/                       (✅ 性能)
│   ├── mod.rs
│   └── monitor.rs
│
├── scheduler/                  (✅ 调度)
│   ├── mod.rs
│   ├── manager.rs
│   ├── priority.rs
│   └── queue.rs
│
├── pool/                       (✅ 连接池)
│   └── mod.rs
│
├── container/                  (✅ 容器)
│   └── mod.rs
│
└── error.rs                    (✅ 错误处理)
```

---

## 关键接口签名

### S1: 核心循环

```rust
#[async_trait]
pub trait AgentLoop: Send + Sync {
    async fn execute(
        &self,
        messages: &mut Vec<Message>,
        max_iterations: usize,
    ) -> Result<String, AgentError>;
}

// 消息类型
#[derive(Debug, Clone)]
pub enum Message {
    User { content: String },
    Assistant { content: Vec<ContentBlock> },
    ToolResult { tool_use_id: String, content: String },
}

pub enum ContentBlock {
    Text(String),
    ToolUse { id: String, name: String, input: serde_json::Value },
}
```

### S2: 工具调度

```rust
#[async_trait]
pub trait ToolDispatcher: Send + Sync {
    async fn execute(
        &self,
        tool_call: ToolCall,
    ) -> Result<String, ToolError>;
}

pub struct ToolRegistry {
    handlers: Arc<RwLock<HashMap<String, Arc<dyn Tool>>>>,
}
```

### S3: 规划系统

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TodoList {
    pub items: Vec<Todo>,
    pub current_index: usize,
}

pub struct PlanningAgent {
    base_agent: Arc<dyn Agent>,
}
```

### S7: 任务系统

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Task {
    pub id: String,
    pub status: TaskStatus,
    pub dependencies: Vec<String>,
    pub result: Option<TaskResult>,
}

#[async_trait]
pub trait TaskManager: Send + Sync {
    async fn create(&self, task: Task) -> Result<String, TaskError>;
    async fn list_pending(&self) -> Result<Vec<Task>, TaskError>;
    async fn mark_complete(&self, id: String, result: TaskResult) -> Result<(), TaskError>;
}
```

### S9: 团队协调

```rust
#[async_trait]
pub trait TeamCoordinator: Send + Sync {
    async fn register_member(&self, agent: Arc<dyn Agent>) -> Result<(), TeamError>;
    async fn distribute_task(&self, task: Task) -> Result<String, TeamError>;
    async fn collect_results(&self) -> Result<Vec<TeamResult>, TeamError>;
}
```

---

## 关键数据流

### S1-S2: 基础循环

```
User Input
    ↓
AgentLoop.execute(messages)
    ↓
Provider.complete(messages, tools)  ← 调用 LLM
    ↓
Parse response.content
    ├─→ ContentBlock::ToolUse
    │   ↓
    │   ToolDispatcher.execute(tool_call)
    │   ↓
    │   ToolRegistry.lookup(name)
    │   ↓
    │   Tool.execute(input)
    │   ↓
    │   ToolResult append to messages
    │
    └─→ ContentBlock::Text
        ↓
        return response_text

(if stop_reason == "tool_use") loop back
```

### S7-S9: 任务和团队

```
Task Created
    ↓
TaskManager.store(task)  ← 持久化
    ↓
TeamCoordinator.distribute(task)
    ↓
Worker Agent claims task
    ↓
Execute in isolated context (S4, S12)
    ↓
TaskManager.mark_complete(task_id, result)
    ↓
Notify dependent tasks
```

---

## 依赖关系

```
S1 (Loop)
  ├─ requires: Message, Provider, ContentBlock parsing
  └─ S2 (Dispatch) requires: S1
      └─ S3 (Planning) requires: S1, S2
      │   └─ S4 (Subagents) requires: S1, S3
      │       └─ S5 (Skills) requires: S4
      │           └─ S6 (Compression) requires: S1-S5
      │
      └─ S7 (Tasks) requires: S1, S2
          ├─ S8 (Background) requires: S7
          │   └─ S9 (Teams) requires: S7, S8
          │       ├─ S10 (Protocols) requires: S9
          │       ├─ S11 (Autonomy) requires: S9, S10
          │       └─ S12 (Worktree) requires: S7, S9, S11
          │
          └─ All phases: Hooks, Config, Security, Monitoring
```

---

## 测试策略

### 单元测试
- 每个 Trait 的 mock 实现
- 工具执行的参数验证
- 错误处理和恢复

### 集成测试
- Agent Loop 完整流程
- Tool 调度和执行
- 多 Agent 协作

### E2E 测试
- 完整的 Agent 任务执行
- 任务依赖和调度
- 团队协作和隔离

---

## 开发优先级

### 第 1 优先级（关键路径）
1. S1: AgentLoop 核心实现
2. S2: ToolDispatcher 完整
3. S7: TaskManager 和存储

### 第 2 优先级（扩展功能）
4. S3: PlanningSystem
5. S4: Subagent 隔离
6. S9: TeamCoordinator

### 第 3 优先级（优化）
7. S5-S6: 知识加载和压缩
8. S8: 后台任务
9. S10-S12: 高级团队功能

---

## 验收标准

### 每个阶段的完成条件
- ✅ 核心接口实现
- ✅ 单元测试覆盖 > 80%
- ✅ 集成测试通过
- ✅ 文档完整
- ✅ `cargo build` 无错误
- ✅ `cargo test` 全部通过

### 代码质量
- 遵循 CLAUDE.md 开发原则
- 使用 `async-trait` for all Traits
- 使用 `thiserror` for error handling
- 完整的类型注解和文档注释

---

## 参考实现

learn-claude-code 中的关键实现：
- `agents/s01_agent_loop.py` - 循环结构
- `agents/s02_tool_use.py` - 工具调度
- `agents/s07_task_system.py` - 任务持久化
- `agents/s09_agent_teams.py` - 多 Agent 通信
- `agents/s12_worktree_task_isolation.py` - 隔离执行

---

## 里程碑时间线

```
Week 1-2: S1-S2   (Loop + Dispatch)
Week 3-4: S3-S6   (Planning + Context)
Week 5-6: S7-S8   (Tasks + Background)
Week 7-8: S9-S12  (Teams + Isolation)
Week 9+: Polish & Production
```

