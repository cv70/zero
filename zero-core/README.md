# zero-core

[![License](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)
[![Rust](https://img.shields.io/badge/rust-1.74+-orange.svg)](https://rust-lang.org)
[![Documentation](https://docs.rs/zero-core/badge.svg)](https://docs.rs/zero-core)

The core trait definitions and shared abstractions for the zero agent framework.

## Overview

`zero-core` provides the foundational traits and data structures that define the zero agent architecture. This crate contains the interface definitions for all core abstractions, including:

- **[Agent]** - The Agent factory trait
- **[Tool]** - The unified tool abstraction
- **[GlobalSharedMemory]** - The global shared memory trait
- **[LLMProvider]** - The LLM provider trait
- **[Channel]** - The message channel trait
- **[Hook]** - The hook trait for extensibility

All core capabilities are defined as traits, enabling loose coupling and easy testing.

## Features

- **Trait-based architecture** - All core capabilities are defined as traits
- **Async first** - Built on `tokio` for high-performance async execution
- **Error handling** - Consistent error types using `thiserror`
- **Serialization** - Full `serde` support for message and tool definitions
- **Extensible hooks** - Hook traits for Agent, Tool, Channel, Provider, and Memory operations

## Core Traits

### Agent Trait

```rust
use zero_core::Agent;
use zero_core::error::AgentError;
use zero_core::message::Message;
use async_trait::async_trait;

#[async_trait]
pub trait Agent: Send + Sync {
    fn name(&self) -> &str;
    fn system_prompt(&self) -> &str;
    fn description(&self) -> &str;
    async fn execute(&self, context: &AgentContext) -> Result<AgentResponse, AgentError>;
}
```

### Tool Trait

```rust
use zero_core::Tool;
use zero_core::error::ToolError;
use zero_core::tool::ToolContext;
use async_trait::async_trait;

#[async_trait]
pub trait Tool: Send + Sync {
    fn metadata(&self) -> ToolMetadata;
    async fn execute(&self, input: &str, ctx: &ToolContext) -> Result<ToolOutput, ToolError>;
}
```

### Channel Trait

```rust
use zero_core::Channel;
use zero_core::message::Message;
use zero_core::error::ChannelError;
use async_trait::async_trait;

#[async_trait]
pub trait Channel: Send + Sync {
    fn name(&self) -> &str;
    async fn send(&self, msg: &Message) -> Result<(), ChannelError>;
    async fn receive(&self) -> Result<Option<Message>, ChannelError>;
    async fn connect(&self) -> Result<(), ChannelError>;
    async fn disconnect(&self) -> Result<(), ChannelError>;
}
```

### LLM Provider Trait

```rust
use zero_core::LLMProvider;
use zero_core::error::ProviderError;
use zero_core::provider::{CompleteOpts, ToolCallResult, MediaInput, ToolMetadata};
use async_trait::async_trait;

#[async_trait]
pub trait LLMProvider: Send + Sync {
    fn name(&self) -> &str;
    fn capabilities(&self) -> ModelCapability;
    fn available_models(&self) -> Vec<String>;
    async fn complete(&self, prompt: &str, opts: CompleteOpts) -> Result<String, ProviderError>;
}
```

### Global Shared Memory Trait

```rust
use zero_core::GlobalSharedMemory;
use zero_core::error::MemoryError;
use zero_core::memory::MemoryEntry;
use async_trait::async_trait;

#[async_trait]
pub trait GlobalSharedMemory: Send + Sync {
    async fn store(&self, namespace: &str, key: &str, value: &str) -> Result<(), MemoryError>;
    async fn retrieve(&self, namespace: &str, key: &str) -> -> Result<Option<String>, MemoryError>;
    async fn search(&self, namespace: &str, query: &str, limit: usize) -> -> Result<Vec<MemoryEntry>, MemoryError>;
    async fn delete(&self, namespace: &str, key: &str) -> -> Result<(), MemoryError>;
    async fn list_keys(&self, namespace: &str) -> -> Result<Vec<String>, MemoryError>;
}
```

### Hook Trait

```rust
use zero_core::Hook;
use zero_core::error::*;
use async_trait::async_trait;

#[async_trait]
pub trait Hook: Send + Sync {
    fn name(&self) -> &str;
    fn priority(&self) -> i32 { 0 }
}

#[async_trait]
pub trait AgentHook: Hook {
    async fn on_agent_init(&self, agent_name: &str) -> Result<(), String>;
    async fn on_agent_run(&self, agent_name: &str) -> Result<(), String>;
    async fn on_agent_run_done(&self, agent_name: &str, result: &str) -> Result<(), String>;
    async fn on_agent_error(&self, agent_name: &str, error: &str) -> Result<(), String>;
}
```

## Installation

```toml
[dependencies]
zero-core = { path = "path/to/zero-core" }
tokio = { version = "1.0", features = ["full"] }
async-trait = "0.1"
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
thiserror = "1.0"
tracing = "0.1"
uuid = { version = "1.0", features = ["v4"] }
reqwest = { version = "0.12", features = ["json"] }
futures = "0.3"
anyhow = "1.0"
```

## Examples

See the [examples](../examples/) directory for working examples:

- **[01-simple-agent]** - A simple greeting agent that demonstrates basic agent execution
- **[02-custom-tool]** - A custom calculator tool demonstrating tool implementation
- **[03-multi-agent]** - A multi-agent coordination example with research, analysis, and report agents

## Project Structure

```
zero-core/
├── src/
│   ├── agent/            # Agent trait and implementations
│   │   ├── trait.rs      # Agent trait definition
│   │   ├── agent_loop.rs # Agent execution loop
│   │   └── context.rs    # Agent context
│   ├── tool/            # Tool trait and built-in tools
│   │   ├── trait.rs     # Tool trait definition
│   │   └── builtins/    # Built-in tools (file operations, bash, etc.)
│   ├── channel/         # Channel trait and implementations
│   │   ├── trait.rs     # Channel trait definition
│   │   └── mod.rs       # Channel module
│   ├── provider/        # LLM provider trait and implementations
│   │   ├── trait.rs     # LLMProvider trait definition
│   │   └── mod.rs       # Provider module
│   ├── memory/          # GlobalSharedMemory trait and implementations
│   │   ├── trait.rs     # GlobalSharedMemory trait definition
│   │   └── mod.rs       # Memory module
│   ├── message.rs       # Message and ContentBlock definitions
│   ├── hooks/           # Hook traits for extensibility
│   └── lib.rs           # Library root
├── examples/            # Example implementations
│   ├── 01-simple-agent.rs
│   ├── 02-custom-tool.rs
│   └── 03-multi-agent.rs
├── Cargo.toml           # Dependencies and package info
└── README.md           # This file
```

## License

This project is licensed under the MIT License - see the [LICENSE](../LICENSE) file for details.

## Contributing

Contributions are welcome! Please feel free to submit issues and pull requests.

1. Fork the repository
2. Create your feature branch (`git checkout -b feature/AmazingFeature`)
3. Commit your changes (`git commit -m 'Add some amazing feature')
4. Push to the branch (`git push origin feature/AmazingFeature`)
5. Open a Pull Request

## Credits

- [Claude Opus 4.6](https://claude.com/claude-code) - AI Assistant
- [Rust](https://www.rust-lang.org/) - The Rust Language
- [OpenAI](https://openai.com/) - For GPT models
- [Anthropic](https://www.anthropic.com/) - For Claude models
- [Ollama](https://ollama.com/) - For local LLM inference

---

<div align="center">

**Built with [Claude Code](https://claude.com/claude-code) and [Rust](https://www.rust-lang.org/)

</div>
