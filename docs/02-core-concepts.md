# Core Concepts

> **Back to Home**: [README.md](../README.md)

Welcome to the core concepts guide! This document explores the fundamental design principles that make Zero a powerful and flexible Agent framework. Whether you're planning to extend Zero or simply understand how it works, these concepts will provide the foundation you need.

## Table of Contents

- [What is Trait-Driven Design?](#what-is-trait-driven-design)
- [5 Core Design Principles](#5-core-design-principles)
  - [1. Trait-First Design](#1-trait-first-design)
  - [2. Async-First Architecture](#2-async-first-architecture)
  - [3. Progressive Layering](#3-progressive-layering)
  - [4. Error Handling Standards](#4-error-handling-standards)
  - [5. Hook System](#5-hook-system)
- [FAQ](#faq)
- [Next Steps](#next-steps)

## What is Trait-Driven Design?

Trait-driven design is an architectural philosophy where **Traits serve as the primary abstractions**, and concrete implementations are secondary concerns. In Rust, a Trait is like an interface or contract that defines what an object can do, without dictating how it does it.

### Why This Matters

Traditional monolithic architectures couple your code tightly together. If you want to swap out your LLM provider from Anthropic to OpenAI, you might need to rewrite significant portions of your Agent logic. With trait-driven design, the Agent doesn't care which specific provider it uses—it only cares that the provider implements the `LLMProvider` trait.

```rust
// Your Agent doesn't care about implementation details
#[async_trait]
pub trait Agent: Send + Sync {
    async fn execute(&self, messages: &[Message]) -> Result<Response>;
}

// Multiple implementations can exist, all interchangeable
pub struct MyAgent {
    provider: Arc<dyn LLMProvider>,  // Could be Anthropic, OpenAI, local LLM...
    tools: Arc<ToolRegistry>,
    memory: Arc<dyn GlobalSharedMemory>,
}
```

### Core Advantages

1. **High Pluggability**: Swap implementations without changing your Agent code
   - Change LLM providers in configuration, not code
   - Add new Tool types without modifying existing code
   - Support multiple communication channels simultaneously

2. **Easy Testing**: Create Mock implementations for testing
   ```rust
   pub struct MockProvider {
       responses: Vec<String>,
   }

   #[async_trait]
   impl LLMProvider for MockProvider {
       async fn complete(&self, _: CompletionRequest) -> Result<CompletionResponse> {
           // Return predetermined responses for testing
       }
   }
   ```

3. **Clear Interface Contracts**: Traits serve as documentation
   - Traits define exactly what methods must exist
   - Type system enforces correct usage
   - Less need for external documentation

4. **Future-Proof Architecture**: Easy to extend without breaking changes
   - New features become new Traits
   - Old code doesn't need to know about new features
   - Systems can evolve independently

### Comparison with Other Approaches

| Aspect | Monolithic | Inheritance | Trait-Driven |
|--------|-----------|-------------|--------------|
| **Coupling** | High | Medium | Low |
| **Testability** | Hard | Medium | Easy |
| **Flexibility** | Low | Medium | High |
| **Type Safety** | Low | High | Very High |
| **Learning Curve** | Low | Medium | Medium |

---

## 5 Core Design Principles

### 1. Trait-First Design

#### What It Is

Every major component in Zero is defined as a Trait first:
- `Agent` - The execution engine
- `Tool` - Reusable capabilities
- `LLMProvider` - Language model abstraction
- `Channel` - Communication medium
- `GlobalSharedMemory` - Persistent state

Then, multiple implementations can provide different behaviors while maintaining the same interface.

#### Why It Matters

This design enables Zero to be incredibly flexible:

1. **LLM Provider Independence**: Your Agent code doesn't change when you switch providers
2. **Tool Extensibility**: Users can write custom Tools that integrate seamlessly
3. **Channel Flexibility**: The same Agent can communicate via Slack, Discord, email, or custom channels
4. **Storage Agnostic**: Replace the filesystem backend with a database without touching core logic

#### Code Example

```rust
use async_trait::async_trait;

// Define what a Tool must do (the contract)
#[async_trait]
pub trait Tool: Send + Sync {
    async fn execute(&self, input: ToolInput) -> Result<String>;
    fn name(&self) -> &str;
    fn description(&self) -> &str;
}

// Implement a specific Tool
pub struct CalculatorTool;

#[async_trait]
impl Tool for CalculatorTool {
    async fn execute(&self, input: ToolInput) -> Result<String> {
        let result = input.operation + input.operands[0] + input.operands[1];
        Ok(result.to_string())
    }

    fn name(&self) -> &str {
        "calculator"
    }

    fn description(&self) -> &str {
        "Performs basic arithmetic"
    }
}

// Your Agent works with any Tool
pub struct DefaultAgent {
    tools: Arc<ToolRegistry>,
}

impl DefaultAgent {
    pub async fn execute_tool(&self, tool_call: ToolCall) -> Result<String> {
        let tool = self.tools.get(&tool_call.name)?;
        tool.execute(tool_call.input).await
    }
}
```

#### Comparison

**Without Traits (Monolithic)**:
```rust
pub struct Agent {
    // Tightly coupled to specific implementations
    provider: AnthropicProvider,
    tools: CalculatorTool,  // What if I need a different tool?
}
```

**With Traits (Flexible)**:
```rust
pub struct Agent {
    // Works with ANY provider and ANY tools
    provider: Arc<dyn LLMProvider>,
    tools: Arc<dyn ToolRegistry>,
}
```

---

### 2. Async-First Architecture

#### What It Is

All I/O operations in Zero are asynchronous using Rust's `async/await` syntax and `tokio` runtime. This includes:
- LLM API calls
- Tool execution
- Memory access
- Channel message sending

#### Why It Matters

**Performance**: When one operation waits (e.g., calling the LLM API), other operations continue instead of blocking the entire system.

```
Traditional (Blocking)         Async (Non-blocking)
─────────────────────         ───────────────────
Task 1: [===LLM API===]       Task 1: [===LLM API===]
Task 2: [Waiting......]       Task 2: [===Running  ===]
Task 3: [Waiting......]       Task 3: [===Running  ===]

Total time: ~3 units          Total time: ~1 unit
```

**Scalability**: A single machine can manage hundreds of concurrent Agents without creating hundreds of threads.

**Natural Expression**: Agent workflows are inherently asynchronous—multiple Agents work in parallel, tools take time to execute, etc. Async code naturally models this.

#### Code Example

```rust
use async_trait::async_trait;
use tokio;

#[async_trait]
pub trait LLMProvider: Send + Sync {
    async fn complete(&self, request: CompletionRequest) -> Result<CompletionResponse>;
}

pub struct DefaultAgentLoop {
    provider: Arc<dyn LLMProvider>,
}

impl DefaultAgentLoop {
    pub async fn execute(&self, messages: &mut Vec<Message>) -> Result<Response> {
        loop {
            // Non-blocking LLM call - other code can run while we wait
            let response = self.provider.complete(CompletionRequest {
                messages: messages.clone(),
                tools: self.get_tools(),
                max_tokens: 1024,
            }).await?;

            match response.stop_reason {
                StopReason::ToolUse => {
                    // Execute tools asynchronously
                    for tool_call in response.tool_calls {
                        let result = self.execute_tool(tool_call).await?;
                        messages.push(Message::tool_result(result));
                    }
                }
                StopReason::EndTurn => {
                    return Ok(response);
                }
            }
        }
    }

    async fn execute_tool(&self, call: ToolCall) -> Result<String> {
        // Tool execution is also async - could be network call, subprocess, etc.
        tokio::time::timeout(
            Duration::from_secs(30),
            self.dispatcher.execute(call)
        ).await??
    }
}

// Multiple agents can run concurrently
#[tokio::main]
async fn main() {
    let agent1 = Arc::new(create_agent());
    let agent2 = Arc::new(create_agent());

    // Both execute in parallel, not sequentially
    let (result1, result2) = tokio::join!(
        agent1.execute(msg1),
        agent2.execute(msg2)
    );
}
```

#### In the Context of Zero

Zero leverages async to enable:

1. **Multi-Agent Execution**: Many Agents run concurrently without thread overhead
2. **Parallel Tool Execution**: Multiple tools can execute simultaneously
3. **Responsive Teams**: Lead agents can coordinate while workers execute independently
4. **Built-in Timeouts**: Prevent rogue tools from hanging the system

---

### 3. Progressive Layering

#### What It Is

Zero is built in 12 stages (S1-S12), where each stage adds exactly one new capability without modifying previous layers. This creates a clean, understandable progression:

```
┌─────────────────────────────────────┐
│  Layer 4: Team Coordination (S9-S12) │
│  Multi-Agent Systems & Coordination  │
├─────────────────────────────────────┤
│  Layer 3: Task Persistence (S7-S8)   │
│  Long-running Tasks & Workflows      │
├─────────────────────────────────────┤
│  Layer 2: Planning & Knowledge (S3-S6)│
│  Reasoning, Context, Subagents      │
├─────────────────────────────────────┤
│  Layer 1: Core Loop (S1-S2)          │
│  Agent Loop & Tool Dispatching       │
└─────────────────────────────────────┘
```

#### Why It Matters

1. **Verifiable Progress**: Each stage is a working system you can build and test
2. **Learning Path**: Start simple, add complexity gradually
3. **Easy Debugging**: Problem in Stage 3? Previous layers work fine
4. **Minimal Changes**: Each PR implements one clear concept

#### Architecture Progression

| Stage | Capability | Key Addition |
|-------|-----------|--------------|
| S1 | Agent Loop | Core message loop with tool calling |
| S2 | Tool Use | Tool registry, dispatcher, execution |
| S3 | Planning | Agents can plan before acting |
| S4 | Subagents | Create specialized agents for tasks |
| S5 | Skills | Load background knowledge dynamically |
| S6 | Context Compression | Handle infinite conversations |
| S7 | Task Persistence | Save/resume tasks from storage |
| S8 | Dependencies | Express task dependencies as graphs |
| S9 | Team Coordination | Lead agent distributes work to workers |
| S10 | Autonomy | Workers autonomously claim and execute tasks |
| S11 | Work Trees | Each task gets isolated work directory |
| S12 | Production Ready | Security, monitoring, configuration |

#### Example: How Layering Works

```rust
// Stage 1-2: Basic Agent Loop
pub async fn stage1_execute(
    messages: &mut Vec<Message>,
    provider: &dyn LLMProvider,
) -> Result<()> {
    let response = provider.complete(messages).await?;
    // Handle tool calls...
}

// Stage 3: Add Planning (no change to S1-2 code)
pub async fn stage3_execute(
    messages: &mut Vec<Message>,
    provider: &dyn LLMProvider,
) -> Result<()> {
    // ASK provider to create a plan first
    let plan = provider.create_plan(messages).await?;
    messages.push(Message::assistant(format!("Plan: {}", plan)));

    // THEN execute the original S1-2 loop
    stage1_execute(messages, provider).await?
}

// Stage 7: Add Task Persistence (no change to S1-6 code)
pub async fn stage7_execute(
    task: Task,
    provider: &dyn LLMProvider,
    storage: &dyn Storage,
) -> Result<()> {
    // SAVE the task
    storage.save_task(&task).await?;

    // EXECUTE using stage3 logic
    stage3_execute(&mut task.messages, provider).await?;

    // PERSIST the result
    storage.update_task(&task).await?;
}
```

---

### 4. Error Handling Standards

#### What It Is

Zero uses the `thiserror` crate to define structured, enumerated error types. This replaces `.unwrap()` and `panic!` with proper error handling throughout the codebase.

#### Why It Matters

1. **Production Safety**: Errors are handled gracefully, not with crashes
2. **Debugging**: Error messages tell you exactly what went wrong
3. **Recovery**: Code can catch and handle specific errors
4. **Type Safety**: Rust compiler ensures you handle all error cases

#### Code Example

```rust
use thiserror::Error;

#[derive(Error, Debug)]
pub enum AgentError {
    #[error("Max iterations exceeded: {0}")]
    MaxIterationsExceeded(usize),

    #[error("Provider error: {0}")]
    ProviderError(String),

    #[error("Tool not found: {0}")]
    ToolNotFound(String),

    #[error("Tool execution failed: {0}")]
    ToolError(#[from] ToolError),

    #[error("Invalid tool input: {0}")]
    InvalidToolInput(String),

    #[error("Memory error: {0}")]
    MemoryError(String),

    #[error("Timeout: {0}")]
    Timeout(String),
}

#[derive(Error, Debug)]
pub enum ToolError {
    #[error("Tool execution timeout")]
    ExecutionTimeout,

    #[error("Tool failed: {0}")]
    ExecutionFailed(String),

    #[error("Invalid parameters: {0}")]
    InvalidParameters(String),
}

// Usage: Errors propagate automatically with ?
#[async_trait]
pub trait Tool: Send + Sync {
    async fn execute(&self, input: ToolInput) -> Result<String, ToolError>;
}

pub async fn run_agent(mut messages: Vec<Message>) -> Result<String, AgentError> {
    let response = self.provider
        .complete(&messages)
        .await
        .map_err(|e| AgentError::ProviderError(e.to_string()))?;

    for tool_call in response.tool_calls {
        let tool = self.tools
            .get(&tool_call.name)
            .ok_or_else(|| AgentError::ToolNotFound(tool_call.name.clone()))?;

        // Tool errors are automatically converted to AgentError via #[from]
        let result = tool.execute(tool_call.input).await?;
        messages.push(Message::tool_result(result));
    }

    Ok(format_final_response(&response))
}
```

#### Best Practices in Zero

1. **Define Errors by Domain**: Each module has its own error enum
   ```rust
   pub enum TaskError { ... }      // Task module errors
   pub enum ToolError { ... }      // Tool module errors
   pub enum ProviderError { ... }  // Provider module errors
   ```

2. **Use `map_err()` for Context**: Add context when converting errors
   ```rust
   storage.save(data)
       .await
       .map_err(|e| AgentError::StorageError(e.to_string()))?
   ```

3. **Never Unwrap**: Use `?` operator or match instead
   ```rust
   // Bad
   let value = risky_operation().unwrap();

   // Good
   let value = risky_operation()?;
   ```

4. **Distinguish User Errors from System Errors**
   ```rust
   #[error("Invalid tool name: {0} (use --list to see available tools)")]
   UserError(String),

   #[error("Internal system error: {0}")]
   SystemError(String),
   ```

---

### 5. Hook System

#### What It Is

The Hook System allows you to observe and extend Zero's behavior by registering callbacks at critical points in the execution flow. These hooks enable observability, logging, metrics, and custom extensions without modifying core code.

#### Why It Matters

**Observability**: See what's happening inside your Agent system:
- When was this tool called?
- How long did the LLM API take?
- What memory was accessed?

**Extensibility**: Add features without modifying core code:
- Log all LLM calls to a file
- Send metrics to Prometheus
- Cache tool results
- Track token usage

**Debugging**: Understand complex Agent behavior:
- Visualize the decision flow
- See exactly which tools are being called
- Trace through context compressions

#### Hook Types Overview

| Hook Type | What It Observes |
|-----------|-----------------|
| `AgentHook` | Agent execution start/end, iterations |
| `ToolHook` | Tool lookup, execution start/end |
| `ChannelHook` | Message sending/receiving |
| `ProviderHook` | LLM API calls, token usage |
| `MemoryHook` | Memory read/write operations |
| `ConfigHook` | Configuration loading/saving |

#### Code Example: Creating and Using Hooks

```rust
use async_trait::async_trait;

// Define a simple logging hook
#[derive(Debug, Clone)]
pub struct LoggingHook {
    name: String,
}

impl LoggingHook {
    pub fn new(name: &str) -> Self {
        Self { name: name.to_string() }
    }
}

// Implement ToolHook to track tool execution
#[async_trait]
pub trait ToolHook: Send + Sync {
    async fn before_execution(&self, tool_call: &ToolCall);
    async fn after_execution(&self, tool_call: &ToolCall, result: &Result<String>);
}

#[async_trait]
impl ToolHook for LoggingHook {
    async fn before_execution(&self, tool_call: &ToolCall) {
        println!("[{}] Executing tool: {}", self.name, tool_call.name);
    }

    async fn after_execution(&self, tool_call: &ToolCall, result: &Result<String>) {
        match result {
            Ok(output) => println!("[{}] Tool {} succeeded: {}", self.name, tool_call.name, output),
            Err(e) => println!("[{}] Tool {} failed: {}", self.name, tool_call.name, e),
        }
    }
}

// Register hooks with HookManager
pub async fn setup_hooks() {
    let mut hook_manager = HookManager::new();

    // Register logging hook
    hook_manager.register_tool_hook(Arc::new(LoggingHook::new("default"))).await;

    // Later: Register metrics hook
    hook_manager.register_provider_hook(Arc::new(MetricsHook::new())).await;
}
```

#### Example Use Case: Token Usage Tracking

```rust
// Create a hook that tracks token usage
#[derive(Debug, Clone)]
pub struct TokenMetricsHook {
    total_tokens: Arc<AtomicUsize>,
}

#[async_trait]
pub trait ProviderHook: Send + Sync {
    async fn before_completion(&self, request: &CompletionRequest);
    async fn after_completion(&self, response: &CompletionResponse);
}

#[async_trait]
impl ProviderHook for TokenMetricsHook {
    async fn before_completion(&self, _request: &CompletionRequest) {
        // Could log request details
    }

    async fn after_completion(&self, response: &CompletionResponse) {
        if let Some(usage) = &response.usage {
            let total = usage.input_tokens + usage.output_tokens;
            self.total_tokens.fetch_add(total, Ordering::SeqCst);
            println!("Tokens used: {}", total);
        }
    }
}

// In your Agent loop:
pub async fn execute_with_tracking(&self) -> Result<()> {
    let metrics_hook = Arc::new(TokenMetricsHook {
        total_tokens: Arc::new(AtomicUsize::new(0)),
    });

    self.hook_manager.register_provider_hook(metrics_hook.clone()).await;

    // Execute Agent loop (hooks fire automatically)
    self.run().await?;

    // Get metrics
    println!("Total tokens used: {}", metrics_hook.total_tokens.load(Ordering::SeqCst));
    Ok(())
}
```

#### Hook Execution Flow

```
User Input
    ↓
[Agent Hook: before_execute]
    ↓
LLM Request
[Provider Hook: before_completion]
    ↓
LLM API Call
    ↓
[Provider Hook: after_completion]
    ↓
Parse Response
    ├─ Tool Use?
    │   ├─ [Tool Hook: before_execution]
    │   ├─ Execute Tool
    │   └─ [Tool Hook: after_execution]
    └─ End Turn?
        └─ Return Result
            ↓
[Agent Hook: after_execute]
    ↓
Final Response
```

---

## FAQ

### Why did Zero choose Rust?

**Type Safety**: Rust's compiler catches errors at compile time that would crash Python code at runtime.

**Performance**: No garbage collection overhead—Zero can manage hundreds of concurrent Agents efficiently.

**Concurrency**: Async/await is more ergonomic than thread management and more efficient than traditional threading.

**Production Readiness**: Forces you to handle errors explicitly, not with try-except that might silently fail.

**Learning Value**: Learning Rust teaches you about memory, concurrency, and type safety—valuable concepts for any developer.

### Will Trait-Driven Design make my code more complex?

Not necessarily. The initial setup is a bit more verbose (defining Traits), but you gain flexibility that simplifies future changes. Compare:

**Without Traits** (seems simpler initially):
```rust
let agent = Agent::new(AnthropicProvider::new());  // Hardcoded
```

**With Traits** (slightly more setup, much more flexible):
```rust
let provider: Arc<dyn LLMProvider> = match config.provider {
    "anthropic" => Arc::new(AnthropicProvider::new()),
    "openai" => Arc::new(OpenAIProvider::new()),
    _ => panic!("Unknown provider"),
};
let agent = Agent::new(provider);  // Works with any provider
```

Later, when requirements change? With Traits, it's a one-line config change. Without Traits, you're rewriting code.

### How do I learn these principles?

1. **Start with Getting Started** ([01-getting-started.md](./01-getting-started.md)): Get the basics working
2. **Read Trait Architecture** ([03-trait-architecture.md](./03-trait-architecture.md)): Deep dive into each Trait
3. **Study Examples** ([04-examples.md](./04-examples.md)): See real code using these principles
4. **Build Something**: Create a custom Tool or implement a new Channel
5. **Read the Source**: The code is well-commented and follows these principles throughout

### Can I use Zero without understanding these concepts?

Yes! You can use Zero by following examples and documentation. But understanding these concepts will help you:
- Debug issues more effectively
- Customize Zero to your needs
- Contribute improvements
- Build production-ready systems

Think of these concepts as the "why" behind Zero's design. The examples show you the "how," but understanding the "why" makes you a better developer.

### What if I find a bug related to these design principles?

Great! Report it on GitHub. Potential bugs related to design principles:
- Race conditions in async code
- Error handling that doesn't use `thiserror`
- Missing hooks at critical points
- Breaking changes between stages

Include specific details about which principle is involved, and we'll help you debug.

### How do these principles relate to the 12 stages?

Each stage builds on the previous ones without breaking them:

- **S1-S2** apply all principles to the core loop
- **S3-S6** extend principles to planning and knowledge
- **S7-S8** maintain principles while adding persistence
- **S9-S12** scale principles to multi-Agent systems

You don't need to implement all 12 stages to use these principles—even a Stage 2 Agent follows all 5 principles.

---

## Next Steps

Now that you understand Zero's core concepts, you're ready to dive deeper:

1. **Trait Architecture** → Learn the detailed interface of each core Trait
   - Read [03-trait-architecture.md](./03-trait-architecture.md)

2. **Practical Examples** → See these concepts in actual code
   - Check [04-examples.md](./04-examples.md)

3. **API Reference** → Detailed documentation of every public API
   - Reference [05-api-reference.md](./05-api-reference.md)

4. **Hook System** → Deep dive into hooks and observability
   - Study [06-hooks-system.md](./06-hooks-system.md)

5. **Build Your First Extension** → Create a custom Tool or Channel
   - Follow tutorials in [07-contributing.md](./07-contributing.md)

Or, if you prefer hands-on learning, jump back to [Getting Started](./01-getting-started.md) and start coding with these concepts in mind!
