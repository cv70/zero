# Zero Project - Quick Start Guide

## 🚀 5 Minute Quick Start

### 1. Build the Project
```bash
cd /home/o/space/zero/zero
cargo build --release
```

### 2. Run Tests
```bash
# All tests
cargo test --lib

# Specific component
cargo test agent:: --lib
cargo test task:: --lib
```

### 3. Generate Documentation
```bash
cargo doc --open
```

---

## 💡 Core Concepts

### Agent Loop
The main execution loop that:
1. Sends messages to LLM
2. Checks response for tool calls
3. Executes tools if needed
4. Appends results to message history
5. Repeats until agent stops

```rust
let loop_impl = DefaultAgentLoop::new(provider, dispatcher);
let mut messages = vec![Message::user("Do something")];
let response = loop_impl.execute(&mut messages, &config).await?;
```

### Tools
Execute specific actions:
- `BashTool` - Run shell commands
- `ReadFileTool` - Read files
- `WriteFileTool` - Create/write files
- `EditFileTool` - Edit file contents

### Tasks
Persistent units of work with:
- Unique ID
- Title and description
- Status (Pending, Running, Completed, Failed)
- Dependencies on other tasks
- Custom metadata

```rust
let task = Task::new("id", "Title", "Description");
manager.create(task).await?;
```

### Planning
Decompose goals into steps:
```rust
let planner = SimplePlanner;
let plan = planner.make_plan("Build a web API").await?;
// plan contains TodoList with steps
```

### Teams
Coordinate multiple agents:
```rust
let coordinator = DefaultTeamCoordinator::new();
coordinator.register_agent("agent_1".to_string()).await?;
coordinator.distribute_task(task).await?;
```

---

## 📂 Project Structure

```
zero/
├── zero-core/           # Main library
│   └── src/
│       ├── message.rs   # Message types
│       ├── agent/       # Agent loop
│       ├── tool/        # Tool system
│       ├── task/        # Task management
│       ├── planning/    # Planning system
│       ├── team/        # Team coordination
│       └── ...
│
├── zero-cli/            # Command-line interface
├── zero-api/            # REST API
│
├── docs/                # Documentation
├── ARCHITECTURE.md      # Architecture details
├── DEVELOPER_GUIDE.md   # Development guide
└── README.md            # This file
```

---

## 🔧 Development Workflow

### Add a Custom Tool

```rust
use zero_core::tool::{Tool, ToolMetadata, ToolContext, ToolOutput};
use serde_json::json;

pub struct MyTool;

#[async_trait::async_trait]
impl Tool for MyTool {
    fn metadata(&self) -> ToolMetadata {
        ToolMetadata {
            name: "my_tool".to_string(),
            description: "Does something special".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "param": { "type": "string" }
                }
            }),
        }
    }

    async fn execute(&self, input: &str, ctx: &ToolContext) -> Result<ToolOutput> {
        // Your implementation
        Ok(ToolOutput::text("Result"))
    }
}
```

### Create a Task

```rust
use zero_core::task::{Task, TaskManager, InMemoryTaskManager};

let manager = InMemoryTaskManager::new();

let task = Task::new(
    "task_1".to_string(),
    "Write documentation".to_string(),
    "Document the API endpoints".to_string(),
);

manager.create(task).await?;
```

### Make a Plan

```rust
use zero_core::planning::Planner;

let planner = SimplePlanner;
let plan = planner.make_plan("Implement authentication").await?;

for item in &plan.items {
    println!("- {}", item.text);
}
```

---

## 📖 Module Guide

### Agent (`zero_core::agent`)
- `AgentLoop` - Execute agent iterations
- `DefaultAgentLoop` - Default implementation
- `Message` - Communication format
- `AgentLoopConfig` - Configuration

### Tool (`zero_core::tool`)
- `Tool` - Tool trait
- `ToolDispatcher` - Execute tool calls
- `BashTool` - Shell execution
- `ReadFileTool` - File reading
- `WriteFileTool` - File writing
- `EditFileTool` - File editing

### Task (`zero_core::task`)
- `Task` - Task model
- `TaskManager` - CRUD operations
- `InMemoryTaskManager` - In-memory implementation
- `TaskStatus` - Status enum

### Planning (`zero_core::planning`)
- `TodoList` - List of steps
- `TodoItem` - Individual step
- `Planner` - Planning trait
- `SimplePlanner` - Basic planner

### Team (`zero_core::team`)
- `TeamCoordinator` - Multi-agent coordination
- `DefaultTeamCoordinator` - Default implementation
- `TeamMessage` - Communication envelope
- `MessageType` - Message types

---

## 🧪 Testing

### Run All Tests
```bash
cargo test --lib
```

### Run Specific Tests
```bash
# Agent loop tests
cargo test agent::agent_loop --lib

# Tool tests
cargo test tool::builtins --lib

# Task tests
cargo test task:: --lib
```

### Test with Output
```bash
cargo test -- --nocapture
```

---

## 📊 Example: Complete Agent Workflow

```rust
use zero_core::{
    agent::{DefaultAgentLoop, AgentLoopConfig, Message},
    tool::{SimpleToolDispatcher},
    task::{Task, InMemoryTaskManager},
    planning::{SimplePlanner, Planner},
    team::{DefaultTeamCoordinator},
};
use std::sync::Arc;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 1. Create components
    let provider = Arc::new(MockProvider::new(vec![]));
    let dispatcher = Arc::new(SimpleToolDispatcher);

    // 2. Create agent loop
    let agent = DefaultAgentLoop::new(provider, dispatcher);
    let config = AgentLoopConfig::default();

    // 3. Run agent
    let mut messages = vec![Message::user("Create a plan")];
    let response = agent.execute(&mut messages, &config).await?;
    println!("Agent response: {}", response);

    // 4. Make a plan
    let planner = SimplePlanner;
    let plan = planner.make_plan("Build a feature").await?;
    println!("Plan items: {}", plan.items.len());

    // 5. Manage tasks
    let task_manager = InMemoryTaskManager::new();
    let task = Task::new("1".to_string(), "Task 1".to_string(), "Do work".to_string());
    task_manager.create(task).await?;

    // 6. Coordinate teams
    let coordinator = DefaultTeamCoordinator::new();
    coordinator.register_agent("agent_1".to_string()).await?;

    println!("Workflow complete!");
    Ok(())
}
```

---

## 🔗 Related Resources

- **Full Architecture**: See `ARCHITECTURE.md`
- **Implementation Roadmap**: See `docs/plans/2026-03-01-implementation-roadmap.md`
- **Phase 1 Details**: See `docs/plans/2026-03-01-phase1-detailed-plan.md`
- **Developer Guide**: See `DEVELOPER_GUIDE.md`
- **Reference Implementation**: See `/home/o/space/zero/learn-claude-code/`

---

## ⚡ Common Tasks

### Execute a Shell Command
```rust
use zero_core::tool::{BashTool, Tool, ToolContext};

let bash = BashTool::new();
let input = r#"{"command": "ls -la"}"#;
let ctx = ToolContext::new("session_1".to_string());
let result = bash.execute(input, &ctx).await?;
```

### Read a File
```rust
use zero_core::tool::{ReadFileTool, Tool, ToolContext};

let reader = ReadFileTool;
let input = r#"{"path": "README.md"}"#;
let ctx = ToolContext::new("session_1".to_string());
let result = reader.execute(input, &ctx).await?;
```

### Track a Task
```rust
use zero_core::task::{Task, TaskStatus, InMemoryTaskManager};

let manager = InMemoryTaskManager::new();
let task = Task::new("1".to_string(), "Title".to_string(), "Desc".to_string());
manager.create(task).await?;
manager.complete("1").await?;
```

---

## 🎯 What's Next?

1. **Read the docs**: Start with `ARCHITECTURE.md`
2. **Explore examples**: Look at unit tests in source files
3. **Build custom tools**: Add your own Tool implementations
4. **Extend planning**: Create custom Planner implementations
5. **Coordinate agents**: Use TeamCoordinator for multi-agent work

---

## 💬 Getting Help

- **Architecture questions**: Read `ARCHITECTURE.md`
- **Development questions**: Read `DEVELOPER_GUIDE.md`
- **API documentation**: Run `cargo doc --open`
- **Code examples**: Look at test sections in source files

---

**Ready to build? Start with the examples above! 🚀**

