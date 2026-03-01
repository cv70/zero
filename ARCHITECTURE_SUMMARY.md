# Zero Project - 架构设计总结

## 📋 设计文档总览

本项目的架构设计包含 4 个核心文档：

### 1. **ARCHITECTURE.md** - 总体架构设计
- **内容**：系统整体架构、核心设计原则、模块依赖关系
- **目标读者**：所有开发者（了解全貌）
- **关键内容**：
  - 5 大设计原则（Trait 驱动、异步优先等）
  - 完整的模块依赖图
  - 12 个阶段的总体视图
  - 关键设计决策的理由

### 2. **docs/plans/2026-03-01-implementation-roadmap.md** - 实现路线图
- **内容**：12 个阶段的全面计划、文件布局、验收标准
- **目标读者**：项目经理、架构师
- **关键内容**：
  - 12 个阶段的详细描述
  - 完整的目录结构规划
  - 每个阶段的关键接口签名
  - 数据流和依赖关系

### 3. **docs/plans/2026-03-01-phase1-detailed-plan.md** - Phase 1 详细计划
- **内容**：S1 和 S2 的具体代码实现、测试、集成点
- **目标读者**：实现者（具体编码）
- **关键内容**：
  - S1: Message 类型、AgentLoop 核心
  - S2: Tool Dispatcher、内置工具
  - 具体的代码框架和签名
  - 单元测试示例
  - 文件清单

### 4. **DEVELOPER_GUIDE.md** - 开发者快速入门
- **内容**：项目结构、开发流程、常见任务、调试技巧
- **目标读者**：新加入的开发者
- **关键内容**：
  - 快速理解项目结构
  - 开发环境设置
  - 常见代码任务
  - 调试和问题解决

---

## 🏗️ 架构三角形

```
        学习路径
        /      \
       /        \
      /          \
    参考实现    当前项目
    (Python)    (Rust)
     \            /
      \          /
       \        /
        设计文档
```

**参考实现**：learn-claude-code 中的 12 个 Python 实现
**当前项目**：zero 中的 Rust 实现
**设计文档**：我们为两者都创建的清晰规划

---

## 📊 系统架构层次

```
┌─────────────────────────────────────┐
│    Application Layer                │
│  (CLI, API, Desktop)                │
├─────────────────────────────────────┤
│  Runtime & Coordination Layer       │
│  AgentLoop  TaskManager  Teams      │
├─────────────────────────────────────┤
│    Core Trait Layer                 │
│  Agent Tool Provider Channel Memory │
├─────────────────────────────────────┤
│  Implementation & Extension Layer   │
│  Anthropic OpenAI Slack Discord...  │
├─────────────────────────────────────┤
│  Infrastructure & Support           │
│  Hooks Config Security Monitoring   │
└─────────────────────────────────────┘
```

---

## 🎯 12 阶段全景

```
Phase 1: 基础循环
├─ S1: Agent Loop       → 核心循环逻辑
└─ S2: Tool Use         → 工具调度执行

Phase 2: 规划与知识
├─ S3: Planning         → 任务规划系统
├─ S4: Subagents        → 子 Agent 隔离
├─ S5: Skill Loading    → 技能动态加载
└─ S6: Context Compact  → 无限会话支持

Phase 3: 持久化
├─ S7: Tasks            → 任务持久化管理
└─ S8: Background Jobs  → 后台异步执行

Phase 4: 团队协作
├─ S9: Agent Teams      → 多 Agent 协调
├─ S10: Protocols       → 通信协议规范
├─ S11: Autonomy        → 自主任务认领
└─ S12: Worktree        → 工作树隔离执行
```

---

## 🔑 核心设计原则

### 1. Trait 驱动（类型安全）
所有核心能力通过 Trait 定义，实现可插拔架构。

### 2. 异步优先（高效并发）
所有 I/O 操作都是异步的，使用 tokio 和 async-trait。

### 3. 逐层构建（可验证进度）
每个阶段只添加一种机制，不修改底层循环。

### 4. 错误处理规范（可靠运行）
使用 thiserror 统一错误处理，避免 panic。

### 5. 钩子系统（可观测可扩展）
在关键点支持插入钩子，实现可观测性和扩展性。

---

## 📁 项目文件结构规划

```
zero-core/src/
├── message.rs                      # S1: 消息类型
├── agent/
│   ├── trait.rs                   # ✅ Agent Trait
│   ├── loop_config.rs             # S1: 循环配置
│   ├── agent_loop.rs              # S1: 循环实现
│   └── hook.rs                    # ✅ Agent 钩子
├── tool/
│   ├── trait.rs                   # ✅ Tool Trait
│   ├── dispatcher.rs              # S2: 工具调度
│   ├── builtins/
│   │   ├── bash.rs                # S2: Bash 工具
│   │   └── file.rs                # S2: 文件工具
│   └── hook.rs                    # ✅ Tool 钩子
├── planning/                       # S3-S6: 规划系统
├── task/                          # S7-S8: 任务系统
└── team/                          # S9-S12: 团队系统
```

---

## 🚀 开发路线

### 第 1 周：Foundation
- [ ] 实现 Message 类型和序列化
- [ ] 设计 AgentLoopConfig
- [ ] 框架化 AgentLoop trait

### 第 2 周：Dispatch
- [ ] 实现 ToolDispatcher
- [ ] 创建内置工具（bash, read, write, edit）
- [ ] 完整测试覆盖

### 第 3-4 周：Planning (S3-S6)
- [ ] TodoWrite 系统
- [ ] Subagent 隔离
- [ ] Skill 加载
- [ ] Context 压缩

### 第 5-6 周：Persistence (S7-S8)
- [ ] Task 数据模型和 JSONL 存储
- [ ] 任务依赖图
- [ ] 后台任务队列

### 第 7-8 周：Teams (S9-S12)
- [ ] Agent 团队注册和协调
- [ ] JSONL 邮箱通信协议
- [ ] 自主任务认领
- [ ] 工作树隔离

---

## 📚 参考学习路径

### 快速理解（30 分钟）
1. 阅读本文档
2. 查看 `docs/ARCHITECTURE.md` 的系统架构部分
3. 浏览 `zero-core/src/` 目录结构

### 深度学习（2-3 小时）
1. 完整阅读 `ARCHITECTURE.md`
2. 查看 `2026-03-01-implementation-roadmap.md`
3. 学习 learn-claude-code 的 Python 实现
4. 研究当前项目的 Trait 定义

### 着手开发（持续）
1. 按 Phase 1 详细计划逐步实现
2. 参考 learn-claude-code 的对应阶段
3. 写测试和文档
4. 提交 PR

---

## 🎓 学习资源

### 项目内文档
- `CLAUDE.md` - 开发协议和原则
- `ARCHITECTURE.md` - 完整架构设计
- `DEVELOPER_GUIDE.md` - 开发快速入门
- `docs/plans/` - 详细实现计划

### 参考实现
- `/home/o/space/zero/learn-claude-code/` - Python 教学实现
  - `agents/s01_agent_loop.py` - 循环核心
  - `agents/s02_tool_use.py` - 工具调度
  - `agents/s07_task_system.py` - 任务管理
  - `agents/s09_agent_teams.py` - 团队协作

### 外部资源
- Rust async-await 文档
- tokio 官方教程
- async-trait crate 说明

---

## ✅ 验收标准

### 每个阶段完成时
- ✅ 所有 Trait 正确实现
- ✅ 单元测试覆盖 > 80%
- ✅ `cargo build --release` 无错误
- ✅ `cargo clippy` 无警告
- ✅ `cargo test` 全部通过
- ✅ 文档和注释完整

### 整个项目完成时
- ✅ 所有 12 个阶段实现完毕
- ✅ 完整的文档和教程
- ✅ E2E 测试覆盖
- ✅ 性能基准数据
- ✅ 生产就绪

---

## 🔄 关键数据流

### Agent 循环（S1-S2）
```
User Input
    ↓
AgentLoop.execute(messages)
    ↓
LLM Call
    ├─ Tool Use? → Dispatch Tools → Append Results → Loop
    └─ End Turn? → Return Response
```

### 任务系统（S7-S9）
```
Task Created
    ↓
TaskManager.store()
    ↓
TeamCoordinator.distribute()
    ↓
Worker Agent claims & executes
    ↓
Mark Complete + Notify Dependents
```

---

## 💡 关键设计决策

| 决策 | 选择 | 理由 |
|------|------|------|
| 消息格式 | 结构化 Enum | 类型安全 + 模式匹配 |
| 存储格式 | JSONL | 流式友好 + 人类可读 |
| 执行模型 | 异步 + 超时 | 防止卡死 + 可配置 |
| 工具调度 | Registry + Dispatcher | 灵活注册 + 易于扩展 |
| 团队通信 | 邮箱 + Request/Response | 解耦合 + 容错 |

---

## 📈 项目复杂度曲线

```
复杂度
  │     S9-S12
  │    /|\
  │   / | \
  │  /  │  \      S7-S8
  │ /   │   \    /|
  │/    │    \  / |
  │     │     \/  |     S3-S6
  │     │     /\  |    /|  \
  │     │    /  \ |   / |   \
  │─────┼───/────\─┬─/──│────\────── 时间
  │     S1-S2     S7-8 S9-12
  │
  └─ Foundation → Planning → Persistence → Teams
```

---

## 🎯 成功指标

### 代码质量
- [ ] 类型覆盖 100%（编译通过）
- [ ] 测试覆盖 > 80%
- [ ] 文档完整度 100%
- [ ] Clippy 零警告

### 功能完整
- [ ] 所有 12 个阶段实现
- [ ] 参考实现功能对等
- [ ] 性能 > Python 版本

### 文档
- [ ] 架构文档完整
- [ ] API 文档完整
- [ ] 教程和示例完整

---

## 🚦 下一步行动

### 立即做
1. [ ] 理解本设计（1 小时）
2. [ ] 查看 learn-claude-code（1 小时）
3. [ ] 审视当前项目结构（30 分钟）

### 这周做
1. [ ] 按 Phase 1 详细计划实现 Message 类型
2. [ ] 编写对应的单元测试
3. [ ] 提交第一个 PR

### 这月做
1. [ ] 完成 Phase 1 (S1-S2)
2. [ ] 创建示例和集成测试
3. [ ] 开始 Phase 2 规划

---

## 📞 获取帮助

- **架构问题**：查看 `ARCHITECTURE.md`
- **开发问题**：查看 `DEVELOPER_GUIDE.md`
- **实现细节**：查看 `2026-03-01-phase1-detailed-plan.md`
- **代码示例**：查看 learn-claude-code 的对应阶段

---

## 📝 文档索引

| 文档 | 对象 | 内容 |
|------|------|------|
| CLAUDE.md | 所有人 | 开发协议 |
| ARCHITECTURE.md | 架构师 | 系统设计 |
| 2026-03-01-implementation-roadmap.md | 项目经理 | 全面计划 |
| 2026-03-01-phase1-detailed-plan.md | 开发者 | 代码细节 |
| DEVELOPER_GUIDE.md | 新开发者 | 快速入门 |

---

## 🎉 总结

Zero 项目是一个完整的、从 0 到 1 的 Agent 系统实现，结合了：

✅ **清晰的架构设计** - 5 大原则 + 4 层架构  
✅ **详细的实现计划** - 12 个阶段 + 每周目标  
✅ **完整的参考实现** - learn-claude-code Python 版本  
✅ **可靠的开发流程** - 测试 + 文档 + 审查  

**目标**：在 2 个月内完成基础系统，在 6 个月内完成完整的生产就绪 Agent 框架。

---

**让我们开始构建！** 🚀

