# Zero Project - Developer Quick Start Guide

## 项目结构速览

```
zero/
├── docs/
│   ├── ARCHITECTURE.md                    # 整体架构设计
│   └── plans/
│       ├── 2026-03-01-implementation-roadmap.md    # 12 阶段路线图
│       ├── 2026-03-01-phase1-detailed-plan.md      # Phase 1 详细计划
│       └── ...
│
├── zero-core/                             # 核心库，包含所有 Traits 和实现
│   ├── Cargo.toml
│   └── src/
│       ├── lib.rs                        # 主导出
│       ├── message.rs                    # 消息类型
│       ├── error.rs                      # 错误类型
│       ├── agent/                        # Agent 系统
│       ├── tool/                         # 工具系统
│       ├── provider/                     # LLM 提供商
│       ├── channel/                      # 消息通道
│       ├── memory/                       # 内存系统
│       ├── planning/                     # 规划系统（S3+）
│       ├── task/                         # 任务系统（S7+）
│       ├── team/                         # 团队系统（S9+）
│       ├── config/                       # 配置管理
│       ├── hooks/                        # 钩子管理
│       └── security/                     # 安全沙箱
│
├── zero-cli/                              # CLI 工具
│   ├── Cargo.toml
│   └── src/
│       └── main.rs
│
├── zero-api/                              # REST API 服务
│   ├── Cargo.toml
│   └── src/
│       └── main.rs
│
├── zero-desktop/                          # 桌面应用（未来）
│
├── benches/                               # 性能基准
│   ├── Cargo.toml
│   └── bench_basic.rs
│
├── CLAUDE.md                              # AI 开发者协议
└── README.md                              # 项目说明

```

---

## 开发流程

### 1. 理解项目

**第一步：阅读核心文档**

1. `CLAUDE.md` - 开发原则
2. `docs/ARCHITECTURE.md` - 架构概览
3. `docs/plans/2026-03-01-implementation-roadmap.md` - 路线图

**第二步：学习参考实现**

进入 `/home/o/space/zero/learn-claude-code` 学习 Python 版本：
- `agents/s01_agent_loop.py` - Agent 循环核心
- `agents/s02_tool_use.py` - 工具调度
- `agents/s07_task_system.py` - 任务系统

**第三步：查看当前代码**

```bash
cd /home/o/space/zero/zero
ls -la zero-core/src/
```

### 2. 设置开发环境

```bash
# 克隆项目
cd /home/o/space/zero/zero

# 安装依赖
cargo build

# 运行所有测试
cargo test

# 特定模块测试
cargo test -p zero-core agent::tests

# 查看警告
cargo clippy
```

### 3. 选择实现的功能

**查看当前状态**：
```bash
git status
```

**当前已完成的部分**（✅）：
- Agent Trait 定义
- Tool Trait 定义
- Provider Trait 定义 (Anthropic, OpenAI, Ollama)
- Channel Trait 定义
- Memory Trait 定义
- Hook 系统基础
- 配置管理
- 错误处理

**需要实现的部分**（Phase 1）：
- [ ] Message 类型和序列化
- [ ] AgentLoop 核心循环
- [ ] ToolDispatcher 工具调度
- [ ] 内置工具（bash, read, write, edit）
- [ ] 完整的单元测试

### 4. 开始开发

**选择一个任务**：

根据 `docs/plans/2026-03-01-phase1-detailed-plan.md`，按这个顺序：

1. **S1.1**: 实现 Message 类型
   ```rust
   // zero-core/src/message.rs (新文件)
   ```

2. **S1.2**: 实现 AgentLoopConfig
   ```rust
   // zero-core/src/agent/loop_config.rs (新文件)
   ```

3. **S1.3**: 实现 AgentLoop 核心
   ```rust
   // zero-core/src/agent/agent_loop.rs (新文件)
   ```

4. **S2.1-S2.3**: 实现工具系统
   ```rust
   // zero-core/src/tool/dispatcher.rs (新文件)
   // zero-core/src/tool/builtins/bash.rs (新文件)
   // zero-core/src/tool/builtins/file.rs (新文件)
   ```

**对于每个任务**：

1. 读取详细计划文档
2. 创建或修改源文件
3. 添加单元测试
4. 验证编译和测试通过
5. 提交 commit

### 5. 代码审查清单

提交前，检查以下项目：

```bash
# ✅ 编译通过，无警告
cargo build --release
cargo clippy

# ✅ 所有测试通过
cargo test

# ✅ 测试覆盖足够
cargo tarpaulin --out Html

# ✅ 格式化代码
cargo fmt

# ✅ 文档注释完整
cargo doc --open
```

### 6. 提交代码

遵循 CLAUDE.md 中的指导：

```bash
# 仅提交相关文件
git add zero-core/src/message.rs zero-core/src/lib.rs

# 提交时使用清晰的消息
git commit -m "feat(zero-core): implement message types and serialization

- Add Message enum with User, Assistant, ToolResult variants
- Add ContentBlock enum with Text and ToolUse variants
- Add Serialize/Deserialize support
- Add unit tests for message serialization

Fixes: S1.1 requirement"

# 推送（如果需要）
git push origin your-branch
```

---

## 关键代码位置

### 理解各个模块

#### Message 类型
**位置**：`zero-core/src/message.rs`（待实现）
**用途**：定义 Agent 通信的消息格式
**学习自**：`learn-claude-code/agents/s01_agent_loop.py` 中的消息结构

#### Agent Trait
**位置**：`zero-core/src/agent/trait.rs` ✅
**用途**：定义 Agent 接口
**查看**：`cargo doc --open` 后搜索 `Agent` trait

#### Tool Trait
**位置**：`zero-core/src/tool/trait.rs` ✅
**用途**：定义工具接口
**实现参考**：`learn-claude-code/agents/s02_tool_use.py`

#### Provider Trait
**位置**：`zero-core/src/provider/trait.rs` ✅
**用途**：定义 LLM 提供商接口
**实现**：`zero-core/src/provider/anthropic.rs` ✅

#### AgentLoop（待实现）
**位置**：`zero-core/src/agent/agent_loop.rs`（S1 实现）
**用途**：核心的 Agent 循环逻辑
**参考流程**：
```
while response.stop_reason == "tool_use":
    response = provider.complete()
    results = dispatcher.execute(tools)
    append(results)
return response
```

---

## 常见任务

### 查看某个 Trait 的定义

```bash
cargo doc --open
# 在浏览器中搜索 Trait 名称
```

### 运行特定测试

```bash
# 运行 agent 模块的所有测试
cargo test agent::

# 运行特定测试
cargo test agent::loop_tests::test_simple_loop_single_response

# 查看测试输出
cargo test -- --nocapture
```

### 添加新的依赖

在 `zero-core/Cargo.toml` 中添加，然后运行：
```bash
cargo build
```

### 创建新工具

1. 在 `zero-core/src/tool/builtins/` 中创建文件
2. 实现 `Tool` trait
3. 在 `zero-core/src/tool/builtins/mod.rs` 中导出
4. 在测试中注册和测试

```rust
pub struct MyTool;

#[async_trait]
impl Tool for MyTool {
    fn name(&self) -> &str { "my_tool" }
    fn schema(&self) -> ToolSchema { /* ... */ }
    async fn execute(&self, arguments: &str) -> Result<String, ToolError> {
        // 实现
    }
}
```

### 使用钩子系统

```rust
use zero_core::hooks::HookManager;

let hooks = Arc::new(HookManager::new());

// 注册钩子（细节待实现）
hooks.on_before_tool_execution(|name, input| {
    println!("Executing: {}", name);
});

// 在 AgentLoop 中使用
let loop_impl = DefaultAgentLoop::new(provider, dispatcher)
    .with_hooks(hooks);
```

---

## 调试技巧

### 打印消息历史

```rust
fn debug_messages(messages: &[Message]) {
    for (i, msg) in messages.iter().enumerate() {
        println!("Message {}: {:?}", i, msg);
    }
}
```

### 追踪工具执行

启用钩子后，可以看到：
```
Before: executing bash
  Input: {"command": "ls -la"}
After: bash completed
  Output: "total 48\n..."
```

### 查看 LLM 调用

```rust
// 在 Provider 实现中添加日志
println!("LLM Request: {:?}", messages);
println!("LLM Response: {:?}", response);
```

---

## 常见问题

### Q: 编译失败，说 Trait 没有实现某个方法？

**A**: 检查 `CLAUDE.md` 中的要求。通常需要：
- 使用 `#[async_trait]` 标注
- 所有方法返回 `Result`
- `Send + Sync` bounds

### Q: 测试失败，说找不到 mock？

**A**: 确保：
1. `Cargo.toml` 包含 `mockall` dev-dependency
2. 使用 `#[mock]` 属性
3. mock 对象的方法签名与 Trait 一致

### Q: 如何理解 Agent Loop 的逻辑？

**A**: 按这个顺序学习：
1. 读 `learn-claude-code/agents/s01_agent_loop.py`
2. 读详细计划文档中的 S1.3 部分
3. 看伪代码注释
4. 运行测试理解数据流

### Q: 怎样添加新的 LLM 提供商？

**A**:
1. 实现 `Provider` trait
2. 放在 `zero-core/src/provider/yourprovider.rs`
3. 在 `zero-core/src/provider/mod.rs` 中导出
4. 添加集成测试

---

## 下一步

### 完成 Phase 1 后

1. 写一个简单的示例：
   ```rust
   // examples/simple_agent.rs
   ```

2. 创建集成测试：
   ```rust
   // zero-core/tests/integration/agent_loop.rs
   ```

3. 开始 Phase 2：规划系统（S3-S6）

### 学习资源

- **Architecture**: `docs/ARCHITECTURE.md`
- **Roadmap**: `docs/plans/2026-03-01-implementation-roadmap.md`
- **S1-S2**: `docs/plans/2026-03-01-phase1-detailed-plan.md`
- **Reference**: `/home/o/space/zero/learn-claude-code/agents/`

---

## 获取帮助

1. **查看文档**：从 `CLAUDE.md` 和 `ARCHITECTURE.md` 开始
2. **查看代码**：已实现的 Trait 在 `src/*/trait.rs` 中
3. **查看参考**：learn-claude-code 的 Python 实现
4. **运行测试**：`cargo test` 查看预期行为

---

## 贡献流程

1. Fork 项目（如果需要）
2. 创建功能分支：`git checkout -b feat/s1-message-types`
3. 实现功能和测试
4. 提交清晰的 commit
5. 推送并创建 Pull Request

---

## 快速命令参考

```bash
# 编译和测试
cargo build                          # 构建项目
cargo test                          # 运行所有测试
cargo test agent::                  # 运行特定模块测试
cargo clippy                        # 代码检查
cargo fmt                           # 代码格式化

# 文档
cargo doc --open                    # 生成并打开文档
cargo doc --document-private-items  # 包含私有项

# 性能
cargo bench                         # 运行性能基准

# 清理
cargo clean                         # 删除构建产物
```

---

## 提交检查清单

在提交代码前，检查：

- [ ] 代码编译无错误：`cargo build --release`
- [ ] 代码无 clippy 警告：`cargo clippy`
- [ ] 测试全部通过：`cargo test`
- [ ] 代码已格式化：`cargo fmt`
- [ ] 添加了必要的注释和文档
- [ ] Commit 消息清晰明确
- [ ] 遵循了 CLAUDE.md 的开发原则

---

祝开发愉快！如有疑问，查阅 `ARCHITECTURE.md` 或 learn-claude-code 的代码。

