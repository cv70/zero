# zero — Universal Agent Runtime Platform

> **简体中文**: [README.zh-CN.md](./README.zh-CN.md)

## Overview

**zero** is a **universal Agent runtime platform** built with Rust, designed to be:

- **Universal** — Works for developers, enterprises, and individual users
- **Rust-first** — High performance, low resource footprint (<5MB RAM)
- **Trait-driven** — Everything is swappable through Trait definitions
- **Extensible** — Core stays minimal, capabilities come from extensions

## ⚡ Core Architecture

```
╔═══════════════════════════════════════════════════════════════════════════════╗
║                                                                               ║
║   █████╗  ██████╗ ██████╗███████╗███████╗███╗   ███╗     ██████╗ ███╗   ██╗   ║
║  ██╔══██╗██╔════╝██╔════╝██╔════╝██╔════╝████╗ ████║    ██╔═══██╗████╗  ██║   ║
║  ███████║██║  ███╗██║     █████╗  ███████╗██╔████╔██║    ██║   ██║██╔██╗ ██║   ║
║  ██╔══██║██║   ██║██║     ██╔══╝  ╚════██║██║╚██╔╝██║    ██║   ██║██║╚██╗██║   ║
║  ██║  ██║╚██████╔╝╚██████╗███████╗███████║██║ ╚═╝ ██║    ╚██████╔╝██║ ╚████║   ║
║  ╚═╝  ╚═╝ ╚═════╝  ╚═════╝╚══════╝╚══════╝╚═╝     ╚═╝     ╚═════╝ ╚═╝  ╚═══╝   ║
║                                                                               ║
╠═══════════════════════════════════════════════════════════════════════════════╣
║                                                                               ║
║  ┌─────────────────────────────┐    ┌─────────────────────────────┐          ║
║  │      🚀 APPLICATION LAYER  │    │     ⚡ EXTENSIONS LAYER     │          ║
║  ├─────────────────────────────┤    ├─────────────────────────────┤          ║
║  │                             │    │                             │          ║
║  │   ┌─────┐ ┌─────┐ ┌─────┐ │    │  ┌──────┐ ┌──────┐ ┌────┐ │          ║
║  │   │CLI  │ │ Web │ │Desk │ │    │  │tools │ │memory│ │prov│ │          ║
║  │   │TUI  │ │ UI  │ │top  │ │    │  │  *   │ │  *   │ │iders│ │          ║
║  │   └──┬──┘ └──┬──┘ └──┬──┘ │    │  └──┬──┘ └──┬──┘ └──┬─┘ │          ║
║  │      │       │       │    │    │     │       │       │   │          ║
║  └──────┼───────┼───────┼────┘    └──────┼───────┼───────┼───┘          ║
║         │       │       │                  │       │       │               ║
║         └───────┴───────┴──────────────────┴───────┴───────┘               ║
║                                                                               ║
╠═══════════════════════════════════════════════════════════════════════════════╣
║                                                                               ║
║         ██████╗ ███████╗██╗   ██╗██╗  ██╗██╗███╗   ██╗██████╗                ║
║         ██╔══██╗██╔════╝██║   ██║██║ ██╔╝██║████╗  ██║██╔══██╗               ║
║         ██║  ██║█████╗  ██║   ██║█████╔╝ ██║██╔██╗ ██║██║  ██║               ║
║         ██║  ██║██╔══╝  ██║   ██║██╔═██╗ ██║██║╚██╗██║██║  ██║               ║
║         ██████╔╝███████╗╚██████╔╝██║  ██╗██║██║ ╚████║██████╔╝               ║
║         ╚═════╝ ╚══════╝ ╚═════╝ ╚═╝  ╚═╝╚═╝╚═╝  ╚═══╝╚═════╝                ║
║                                                                               ║
║                         ┌─────────────────────────────────────┐                ║
║                         │        🧠 CORE KERNEL               │                ║
║                         ├─────────────────────────────────────┤                ║
║                         │                                     │                ║
║  ┌────────────┐  ┌─────┴─────┐  ┌────────────┐  ┌────────┴────────┐         ║
║  │    🤖      │  │    🔧     │  │    💾     │  │       📡        │         ║
║  │   AGENT    │  │   TOOL    │  │   MEMORY  │  │     PROVIDER    │         ║
║  │            │  │           │  │           │  │                 │         ║
║  │ • Factory  │  │ • Unified │  │ • Layered │  │ • Multi-model  │         ║
║  │ • Multi    │  │ • Adapter │  │ • Global  │  │ • Capability   │         ║
║  │ • Coord    │  │ • Registry│  │ • Filesys │  │ • Router       │         ║
║  └────────────┘  └───────────┘  └───────────┘  └────────────────┘         ║
║                                                                               ║
║         ┌───────────────────────────────────────────────────────┐             ║
║         │                       📱                              │             ║
║         │                      CHANNEL                          │             ║
║         │  ┌─────────┐ ┌─────────┐ ┌─────────┐ ┌─────────┐    │             ║
║         │  │ Telegram │ │ Discord │ │  Slack  │ │ Email   │    │             ║
║         │  └─────────┘ └─────────┘ └─────────┘ └─────────┘    │             ║
║         └───────────────────────────────────────────────────────┘             ║
║                                                                               ║
╚═══════════════════════════════════════════════════════════════════════════════╝
```

### Data Flow

```
┌──────────────┐     ┌──────────────┐     ┌──────────────┐     ┌──────────────┐
│   Client     │────►│   Agent      │────►│    Tool      │────►│   Provider   │
│ (CLI/Web/API)│     │   Engine     │     │   System     │     │    (LLM)     │
└──────────────┘     └──────────────┘     └──────────────┘     └──────────────┘
       │                    │                    │                    │
       │                    ▼                    ▼                    │
       │              ┌──────────────┐     ┌──────────────┐          │
       │              │   Memory     │◄────│   Adapter    │          │
       │              │   System     │     │   Layer      │          │
       │              └──────────────┘     └──────────────┘          │
       │                    │                                             │
       └────────────────────┴─────────────────────────────────────────────┘
                           │
                           ▼
                    ┌──────────────┐
                    │    Output    │
                    │  (Response)  │
                    └──────────────┘
```

## Core Features

### Trait-Driven Architecture

All core capabilities are defined as Rust Traits:

| Trait | Purpose |
|-------|---------|
| `Agent` | Agent factory and execution |
| `Tool` | Unified tool abstraction |
| `GlobalSharedMemory` | Cross-agent memory |
| `LLMProvider` | Model provider abstraction |
| `Channel` | Message channel abstraction |

### Unified Tool System

- **Declarative Tools** — JSON/YAML definitions
- **MCP Integration** — Model Context Protocol support
- **Skills** — File-based skill definitions
- **Rust Implementation** — Direct Trait implementation

### Multi-Model Support

```rust
pub enum ModelCapability {
    TextOnly,          // Text only
    TextAndImages,     // Text + Images
    TextAndVideo,      // Text + Video
    Multimodal,        // Text + Images + Video + Audio
}
```

### Layered Memory System

- **Agent Isolation** — Each agent has independent context
- **Global Shared Memory** — Cross-agent knowledge sharing via Trait
- **Filesystem Storage** — Default implementation using filesystem structure

## Quick Start

### Installation

```bash
# Build from source
cargo build -p zero-cli

# Run CLI
cargo run -p zero-cli -- --help
```

### Basic Usage

```bash
# Start interactive session
zero run

# Execute single command
zero exec "Hello, world!"

# List available agents
zero list-agents

# List available tools
zero list-tools
```

### Configure Models

```bash
# Add model providers
zero provider add openai --api-key $OPENAI_API_KEY
zero provider add anthropic --api-key $ANTHROPIC_API_KEY
zero provider add ollama --endpoint http://localhost:11434
```

## Project Structure

```
zero/
├── zero-core/           # Core kernel (Trait definitions)
│   └── src/
│       ├── agent/       # Agent engine
│       ├── tool/        # Unified tool system
│       ├── memory/      # Memory system
│       ├── provider/   # LLM providers
│       └── channel/    # Message channels
├── zero-cli/           # CLI application (TUI)
├── zero-web/           # Web UI (React)
├── zero-desktop/       # Desktop app (Tauri)
├── zero-api/           # REST/gRPC API server
└── zero-ext/           # Extension ecosystem
    ├── tools-*         # Tool extensions
    ├── memory-*        # Memory backends
    ├── providers-*     # LLM provider implementations
    └── channels-*     # Channel implementations
```

## API Endpoints

```
/api/v1/
├── agents/
│   ├── GET    /list
│   ├── POST   /create
│   ├── POST   /{id}/run
├── tools/
│   ├── GET    /list
│   ├── POST   /register
│   ├── POST   /{name}/execute
├── memory/
│   ├── GET    /{namespace}/{key}
│   ├── POST   /{namespace}/{key}
│   └── GET    /{namespace}/search
└── models/
    ├── GET    /list
    └── POST   /complete
```

## Design Principles

1. **YAGNI** — Remove all unnecessary features
2. **Trait-driven** — Everything is swappable
3. **Extension-first** — Core stays minimal
4. **Unified abstraction** — Eliminate differences through Traits

## Roadmap

### Phase 1: Core Validation
- Agent Trait + basic execution
- Unified Tool Trait + adapters
- Filesystem Memory backend
- CLI + REST API

### Phase 2: Extension Ecosystem
- Tool extensions (files, shell, websearch)
- MCP adapter
- Skills adapter
- SQLite memory backend
- Provider extensions (OpenAI, Anthropic, Ollama)

### Phase 3: Multi-modal UI
- Web UI
- Desktop app
- More channels

### Phase 4: Advanced Features
- Security sandbox
- RAG capabilities
- More channels

## License

MIT
