# Zero — Universal Agent Runtime Platform

> **简体中文**: [README.zh-CN.md](./README.zh-CN.md)

## What is Zero?

**Zero** is a **universal Agent runtime platform** built with Rust, designed for building intelligent, extensible AI applications through a trait-driven architecture.

### Core Values

- **Trait-Driven** — All core capabilities are swappable through Rust Traits
- **High Performance** — Efficient concurrent execution, minimal resource footprint
- **Fully Extensible** — Core stays minimal, capabilities come from extensions

## Architecture Overview

```
┌─────────────────────────────────────────┐
│      Applications Layer (CLI/Web)       │
├─────────────────────────────────────────┤
│      Core Kernel (Trait-Based)          │
│  Agent | Tool | Memory | Provider       │
├─────────────────────────────────────────┤
│      Extension Ecosystem                │
│  Tools | Providers | Channels           │
└─────────────────────────────────────────┘
```

## Learning Path

Choose your journey based on what you want to do:

### Quick Start (5 min)
[Get Zero running in under 5 minutes](./docs/01-getting-started.md). Install, write your first Agent, run it.

**Perfect for:** Anyone who wants to see it work immediately

### Core Concepts (15 min)
[Understand the Trait-driven design philosophy and 5 core principles](./docs/02-core-concepts.md) that make Zero extensible.

**Perfect for:** Developers who want to understand the "why" behind the design

### Trait Architecture (30 min)
[Deep dive into each core Trait](./docs/03-trait-architecture.md): Agent, Tool, Memory, Provider, Channel. Learn how they interact and how to extend them.

**Perfect for:** Contributors and advanced users building custom implementations

### Code Examples (30 min)
[From simple "Hello Agent" to multi-Agent coordination](./docs/04-examples.md). Real, runnable code with detailed explanations.

**Perfect for:** Learners who prefer "show me the code"

### API Reference
[Complete API documentation for all core modules](./docs/05-api-reference.md). Type signatures, parameters, return values.

**Perfect for:** Building with Zero, looking up specific APIs

### Hooks System (20 min)
[Learn about Zero's plugin/extension system](./docs/06-hooks-system.md). 6 hook types, when to use them, implementation patterns.

**Perfect for:** Building extensions and customizations

### Contributing Guide
[Development setup, coding standards, Git workflow, testing requirements](./docs/07-contributing.md). Everything you need to contribute.

**Perfect for:** Contributors to Zero itself

## 5-Minute Example

```rust
use zero_core::{Agent, AgentContext};
use async_trait::async_trait;

struct MyAgent;

#[async_trait]
impl Agent for MyAgent {
    async fn execute(&self, context: &AgentContext) -> Result<String> {
        Ok(format!("Hello from {}!", context.name))
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    let agent = MyAgent;
    let context = AgentContext::new("MyAgent");
    let result = agent.execute(&context).await?;
    println!("{}", result);
    Ok(())
}
```

## Feature Highlights

- **Multi-Model Support** — OpenAI, Anthropic, Ollama, and custom providers
- **Unified Tool System** — JSON/YAML tools, MCP integration, Rust implementations
- **Layered Memory** — Agent isolation + global shared memory
- **Channel System** — CLI, Web, Discord, Email, and more
- **Hook System** — Extensibility at every critical point
- **Production Ready** — Error handling, async/await, type safety

## Project Status

- ✅ **Phase 1**: Core Trait definitions and basic execution
- ✅ **Phase 2**: Extension ecosystem (tools, memory, providers)
- ✅ **Phase 3**: Multi-modal UIs (Web, Desktop)
- ✅ **Phase 4**: Advanced features (security, RAG)

## Installation

```bash
# Build from source
cargo build -p zero-core

# Run CLI
cargo run -p zero-cli -- --help
```

For detailed setup instructions, see [Getting Started](./docs/01-getting-started.md).

## Roadmap

- Short-term: Documentation improvements, community examples
- Medium-term: Performance optimization, advanced RAG capabilities
- Long-term: Full multi-Agent team coordination, autonomous agents

## Contributing

Zero welcomes contributions! See [Contributing Guide](./docs/07-contributing.md) for:
- Development environment setup
- Coding standards and conventions
- Git workflow and commit process
- Testing requirements

## License

MIT

---

**Have questions?** Check out our [FAQ section](./docs/02-core-concepts.md#faq) or [open an issue on GitHub](https://github.com/your-org/zero/issues).
