# Getting Started with Zero

Welcome to the Zero framework! This guide will help you set up the project and run your first program in just 5 minutes.

## Table of Contents

- [System Requirements](#system-requirements)
- [Installation](#installation)
- [Your First Program](#your-first-program)
- [Quick Command Reference](#quick-command-reference)
- [Troubleshooting](#troubleshooting)
- [What's Next](#whats-next)

## System Requirements

Before you begin, ensure your system meets these requirements:

### Operating System
- **macOS**: 10.13 or later (Intel or Apple Silicon)
- **Linux**: Ubuntu 18.04+, Debian 10+, or equivalent
- **Windows**: Windows 10 or later (with WSL2 recommended)

### Rust Environment
- **Rust 1.70+**: Required for the 2024 edition
  - Install via [rustup](https://rustup.rs/): `curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh`
  - Verify installation: `rustc --version` (should show 1.70 or later)

### Additional Tools
- **Cargo**: Comes with Rust (package manager)
- **Git**: Required for cloning the repository
- **4GB+ RAM**: For compilation
- **2GB+ Disk Space**: For the build artifacts

### Verify Installation

```bash
# Check Rust version
rustc --version  # Should output: rustc 1.70.0 or later

# Check Cargo version
cargo --version  # Should output: cargo 1.70.0 or later
```

If you see version numbers, you're ready to proceed!

## Installation

### Step 1: Clone the Repository

```bash
git clone https://github.com/yourusername/zero.git
cd zero
```

### Step 2: Build the Project

Navigate to the main project directory and build it:

```bash
cd /path/to/zero
cargo build --release
```

This may take a few minutes the first time as it compiles all dependencies. You should see output like:

```
   Compiling zero-core v0.1.0
   Compiling zero-cli v0.1.0
   Compiling zero-api v0.1.0
    Finished release [optimized] target(s) in 123.45s
```

### Step 3: Verify Installation

```bash
# Run the test suite
cargo test --lib

# Expected output shows tests passing
running 15 tests
test result: ok. 15 passed

# Generate and view documentation
cargo doc --open
```

If all tests pass and documentation generates successfully, your installation is complete!

## Your First Program

Let's create a simple agent that uses the core traits. This program demonstrates the fundamental concept of the Zero framework.

### Create a New Rust File

Create `examples/hello_agent.rs`:

```rust
use async_trait::async_trait;
use serde_json::json;
use std::sync::Arc;

// Define a simple custom message
#[derive(Debug, Clone)]
pub struct SimpleMessage {
    pub role: String,
    pub content: String,
}

// Define a minimal mock LLM provider
pub struct MockProvider;

#[async_trait]
pub trait LLMProvider: Send + Sync {
    async fn call(&self, messages: Vec<SimpleMessage>) -> Result<String, Box<dyn std::error::Error>>;
}

#[async_trait]
impl LLMProvider for MockProvider {
    async fn call(&self, _messages: Vec<SimpleMessage>) -> Result<String, Box<dyn std::error::Error>> {
        Ok("Hello from Zero! I'm running successfully.".to_string())
    }
}

// Define a simple agent
pub struct SimpleAgent {
    provider: Arc<dyn LLMProvider>,
}

impl SimpleAgent {
    pub fn new(provider: Arc<dyn LLMProvider>) -> Self {
        Self { provider }
    }

    pub async fn run(&self, prompt: &str) -> Result<String, Box<dyn std::error::Error>> {
        let messages = vec![SimpleMessage {
            role: "user".to_string(),
            content: prompt.to_string(),
        }];

        self.provider.call(messages).await
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create agent with mock provider
    let provider = Arc::new(MockProvider);
    let agent = SimpleAgent::new(provider);

    // Run the agent
    let response = agent.run("What can you do?").await?;
    println!("Agent response: {}", response);

    println!("\nCongratulations! Your first Zero agent is running!");
    Ok(())
}
```

### Run Your First Program

```bash
cd /path/to/zero
cargo run --example hello_agent
```

Expected output:
```
Agent response: Hello from Zero! I'm running successfully.

Congratulations! Your first Zero agent is running!
```

### What Just Happened?

1. **Provider**: The `MockProvider` simulates an LLM (Language Model)
2. **Agent**: The `SimpleAgent` orchestrates communication with the provider
3. **Messages**: The framework passes structured messages between components
4. **Async/Await**: Everything runs asynchronously using `tokio`

This demonstrates the core trait-driven architecture that powers Zero!

## Quick Command Reference

Here are the most common commands you'll use during development:

### Building and Testing

```bash
# Build in debug mode (faster, larger binary)
cargo build

# Build in release mode (slower, optimized binary)
cargo build --release

# Run all tests
cargo test --lib

# Run tests in a specific module
cargo test agent:: --lib
cargo test tool:: --lib
cargo test task:: --lib

# Run tests with output
cargo test -- --nocapture
```

### Running Code

```bash
# Run the hello_agent example
cargo run --example hello_agent

# Run specific binaries
cargo run -p zero-cli -- [arguments]
cargo run -p zero-api

# Run with release optimizations
cargo run --release --example hello_agent
```

### Documentation and Exploration

```bash
# Generate and open API documentation
cargo doc --open

# Check for compilation issues without building
cargo check

# View warnings and best practices
cargo clippy

# Format code to project style
cargo fmt
```

### Development Workflow

```bash
# Check if code compiles
cargo check

# Fix formatting issues
cargo fmt --check  # Check only
cargo fmt          # Fix in place

# Before committing
cargo build        # Must succeed
cargo test --lib   # All tests must pass
```

## Troubleshooting

### Rust Version Too Old

**Problem**: `error: package requires rustc 1.70 or newer`

**Solution**:
```bash
# Update Rust to the latest stable version
rustup update stable

# Set the default toolchain
rustup default stable

# Verify the version
rustc --version
```

### Compilation Errors

**Problem**: `error: could not compile 'zero-core'`

**Solution**:
```bash
# Clean the build cache
cargo clean

# Try building again
cargo build

# If still failing, check Rust is fully updated
rustup update
cargo update
```

### Tests Failing

**Problem**: Some tests are failing after installation

**Solution**:
```bash
# Run tests with verbose output to see details
cargo test --lib -- --nocapture

# Run a specific failing test
cargo test <test_name> -- --nocapture --test-threads=1
```

### Port Already in Use

**Problem**: `error: bind error, address already in use`

**Solution**:
```bash
# Find process using the port (macOS/Linux)
lsof -i :8080

# Kill the process
kill -9 <PID>

# Or use a different port
export PORT=8081
cargo run
```

### Out of Memory During Build

**Problem**: `error: could not compile due to previous error`

**Solution**:
```bash
# Use single-threaded compilation
cargo build -j 1

# Or increase swap space (Linux):
# Add a swap file following your OS documentation
```

### Cargo Not Found

**Problem**: `command not found: cargo`

**Solution**:
```bash
# Ensure Rust is installed correctly
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Source the environment (if needed)
source $HOME/.cargo/env

# Verify installation
cargo --version
```

## What's Next?

Congratulations! You've successfully set up Zero and run your first agent program. Here's where to go from here:

### Continue Learning

1. **Core Concepts**: Read [02-core-concepts.md](./02-core-concepts.md) to understand Agents, Tools, Tasks, and Planning
2. **Architecture Details**: Review [03-trait-architecture.md](./03-trait-architecture.md) to learn about the trait-driven design
3. **Examples**: Explore [04-examples.md](./04-examples.md) for practical code samples
4. **API Reference**: Check [05-api-reference.md](./05-api-reference.md) for detailed API documentation

### Build Your Own

- Create custom tools by implementing the `Tool` trait
- Build agents with specialized capabilities
- Extend the framework with hooks (see [06-hooks-system.md](./06-hooks-system.md))
- Coordinate multiple agents using `TeamCoordinator`

### Get Involved

- Check [07-contributing.md](./07-contributing.md) for contribution guidelines
- Review the architecture in `ARCHITECTURE.md`
- Explore the test suite to understand implementation patterns

**Ready to dive deeper? Start with the Core Concepts guide!**
