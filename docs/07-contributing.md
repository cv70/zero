# Contributing to Zero

Welcome! We're excited that you want to contribute to Zero. This guide will help you get started with the development process and make meaningful contributions to the project.

## Table of Contents

- [Development Setup](#development-setup)
- [Code Style and Standards](#code-style-and-standards)
- [Project Structure](#project-structure)
- [Git Workflow](#git-workflow)
- [Testing](#testing)
- [Trait Design Guidelines](#trait-design-guidelines)
- [Common Pitfalls](#common-pitfalls)

---

## Development Setup

### Prerequisites

Before you start, ensure you have:

- **Rust 2024 edition** (or later) - Install from [rustup.rs](https://rustup.rs/)
- **Git** - For version control
- **A text editor or IDE** - We recommend VS Code with rust-analyzer extension

### Setting Up the Development Environment

1. **Clone the repository**:
   ```bash
   git clone https://github.com/yourusername/zero.git
   cd zero
   ```

2. **Install Rust and dependencies**:
   ```bash
   rustup update
   rustup component add rustfmt clippy
   ```

3. **Build the project**:
   ```bash
   cargo build
   ```

4. **Run tests to verify setup**:
   ```bash
   cargo test
   ```

### Building from Source

The project is a Rust workspace with multiple crates:

```bash
# Build all crates
cargo build --release

# Build specific crate
cargo build -p zero-core
cargo build -p zero-cli

# With documentation
cargo doc --open
```

### Running Tests

```bash
# Run all tests
cargo test

# Run tests for specific module
cargo test agent::

# Run specific test with output
cargo test agent::loop_tests::test_simple_loop -- --nocapture

# Run with coverage (requires tarpaulin)
cargo tarpaulin --out Html
```

---

## Code Style and Standards

### Naming Conventions

Follow Rust naming conventions as defined in the [Rust API Guidelines](https://rust-lang.github.io/api-guidelines/):

- **Types, Traits, Enums**: `PascalCase` (e.g., `AgentLoop`, `ToolDispatcher`)
- **Functions, variables, parameters**: `snake_case` (e.g., `execute_agent`, `tool_name`)
- **Constants**: `UPPER_SNAKE_CASE` (e.g., `MAX_ITERATIONS`)
- **Private items**: prefix with underscore if needed (e.g., `_internal_method`)

### async-trait Usage Requirements

All trait definitions that contain async methods **must** use the `async-trait` attribute:

```rust
use async_trait::async_trait;

#[async_trait]
pub trait Agent: Send + Sync {
    async fn execute(&self, context: &AgentContext) -> Result<AgentResponse, AgentError>;

    fn name(&self) -> &str;
}
```

**Key requirements**:
- All async trait methods must have `async-trait` attribute on the trait
- Traits must implement `Send + Sync` bounds
- Return types should use `Result<T, E>` for error handling

### Error Handling

Use `thiserror` for error handling. **Never use `unwrap()` or `panic!()` in production code**:

**Good - Using thiserror**:
```rust
use thiserror::Error;

#[derive(Error, Debug)]
pub enum AgentError {
    #[error("Max iterations exceeded: {0}")]
    MaxIterationsExceeded(usize),

    #[error("Tool execution failed: {0}")]
    ToolError(#[from] ToolError),

    #[error("Provider error: {0}")]
    ProviderError(String),
}

// Usage
pub async fn execute(&self) -> Result<String, AgentError> {
    if iterations > max {
        return Err(AgentError::MaxIterationsExceeded(iterations));
    }
    Ok(result)
}
```

**Bad - Using unwrap**:
```rust
// DON'T DO THIS
let result = some_operation().unwrap();  // Can panic!
let value = option_value.expect("value");  // Can panic!
```

### Code Formatting and Linting

Before committing, ensure your code passes all checks:

```bash
# Format code
cargo fmt

# Check for common mistakes and improvements
cargo clippy

# Run clippy with pedantic warnings
cargo clippy -- -W clippy::all -W clippy::pedantic

# Fix clippy warnings automatically
cargo clippy --fix
```

### Documentation Comments

All public items should have documentation comments:

```rust
/// Executes the agent loop until a stop condition is met.
///
/// # Arguments
/// * `context` - The agent execution context
/// * `max_iterations` - Maximum number of iterations
///
/// # Returns
/// Returns the final agent response or an error
///
/// # Example
/// ```
/// let response = agent.execute(&context).await?;
/// ```
pub async fn execute(
    &self,
    context: &AgentContext,
    max_iterations: usize,
) -> Result<AgentResponse, AgentError> {
    // implementation
}
```

---

## Project Structure

### Workspace Organization

The project uses a Rust workspace with multiple crates:

```
zero/
├── docs/                          # Documentation
│   ├── 01-getting-started.md      # Getting started guide
│   ├── 03-trait-architecture.md   # Architecture details
│   ├── 07-contributing.md         # This file
│   └── plans/                     # Implementation roadmaps
│
├── zero-core/                     # Core library (Main implementation)
│   ├── Cargo.toml
│   └── src/
│       ├── lib.rs                 # Main library exports
│       ├── message.rs             # Message types
│       ├── error.rs               # Error definitions
│       ├── agent/                 # Agent trait and implementations
│       │   ├── trait.rs
│       │   ├── agent_loop.rs
│       │   └── mod.rs
│       ├── tool/                  # Tool trait and implementations
│       │   ├── trait.rs
│       │   ├── dispatcher.rs
│       │   ├── builtins/          # Built-in tools (bash, read, write)
│       │   └── mod.rs
│       ├── provider/              # LLM provider abstraction
│       │   ├── trait.rs
│       │   ├── anthropic.rs       # Anthropic provider
│       │   ├── openai.rs          # OpenAI provider
│       │   └── mod.rs
│       ├── channel/               # Message channel trait
│       │   ├── trait.rs
│       │   └── mod.rs
│       ├── memory/                # Memory system
│       │   ├── trait.rs
│       │   └── mod.rs
│       ├── hooks/                 # Hook system
│       │   ├── trait.rs
│       │   └── manager.rs
│       ├── config/                # Configuration management
│       │   └── mod.rs
│       └── security/              # Security sandbox
│           └── mod.rs
│
├── zero-cli/                      # Command-line interface
│   ├── Cargo.toml
│   └── src/
│       └── main.rs
│
├── zero-api/                      # REST API server
│   ├── Cargo.toml
│   └── src/
│       └── main.rs
│
├── CLAUDE.md                      # AI developer protocol
├── DEVELOPER_GUIDE.md             # Quick start guide
└── README.md                      # Project README
```

### Module Organization Principles

1. **Keep modules focused**: Each module should have a single responsibility
2. **Use mod.rs for organization**: Group related types in module files
3. **Export public API**: Use `pub use` in mod.rs to control what's exported
4. **Private by default**: Make items private unless they need to be public

Example module structure:

```rust
// src/agent/mod.rs
pub mod trait;
pub mod agent_loop;
pub mod context;

pub use trait::Agent;
pub use agent_loop::DefaultAgentLoop;
pub use context::AgentContext;
```

### Where to Add New Features

- **New Trait**: Create in appropriate module directory (e.g., `src/agent/new_trait.rs`)
- **New Implementation**: Add in submodule (e.g., `src/tool/builtins/newtool.rs`)
- **New Module**: Create directory with `mod.rs` and export from parent
- **Tests**: Keep adjacent to code using `#[cfg(test)]` modules

---

## Git Workflow

### Creating Feature Branches

Use descriptive branch names following the pattern: `<type>/<feature-description>`

```bash
# Feature branch
git checkout -b feat/agent-loop-implementation

# Bug fix branch
git checkout -b fix/memory-leak-in-dispatcher

# Documentation
git checkout -b docs/update-contributing-guide

# Refactoring
git checkout -b refactor/tool-trait-simplification
```

### Commit Message Conventions

Follow conventional commits format for clear commit history:

```
<type>(<scope>): <subject>

<body>

<footer>
```

**Types**:
- `feat`: A new feature
- `fix`: A bug fix
- `docs`: Documentation changes
- `style`: Code style changes (formatting, missing semicolons, etc.)
- `refactor`: Code refactoring without feature changes
- `test`: Adding or updating tests
- `chore`: Changes to build process, dependencies, etc.

**Examples**:

```bash
# Simple commit
git commit -m "feat(agent): implement agent loop execution

- Add AgentLoop struct with main loop logic
- Implement provider interaction
- Add tool execution integration
- Add unit tests for loop behavior"

# Bug fix with reference
git commit -m "fix(provider): handle timeout errors correctly

Previously, timeout errors were not propagated correctly.
Now they are caught and returned as ProviderError.

Fixes: #123"

# Documentation update
git commit -m "docs: add contributing guide

Add comprehensive contributing documentation covering:
- Development setup
- Code style standards
- Testing guidelines
- Git workflow"
```

### Pull Request Process

1. **Ensure code quality before submitting**:
   ```bash
   cargo build --release
   cargo clippy
   cargo fmt
   cargo test
   ```

2. **Push your branch**:
   ```bash
   git push origin feat/your-feature
   ```

3. **Create a pull request** with a clear description:
   - Reference any related issues
   - Explain the motivation for changes
   - Describe what was tested

4. **Code review process**:
   - Address all review comments
   - Push additional commits if needed
   - Re-request review once changes are made

5. **Merge requirements**:
   - All tests must pass
   - Code review approval required
   - Branch must be up to date with main

---

## Testing

### Running the Test Suite

```bash
# Run all tests
cargo test

# Run with output (useful for debugging)
cargo test -- --nocapture

# Run specific module tests
cargo test agent::
cargo test tool::dispatcher::

# Run a single test
cargo test agent::loop_tests::test_basic_execution

# Run tests in release mode (faster)
cargo test --release
```

### Writing Tests

Follow Rust testing conventions:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_agent_execution() {
        // Arrange
        let agent = TestAgent::new();
        let context = AgentContext::default();

        // Act
        let result = agent.execute(&context).await;

        // Assert
        assert!(result.is_ok());
        let response = result.unwrap();
        assert_eq!(response.content, "expected");
    }

    #[test]
    fn test_sync_operation() {
        let value = compute_something();
        assert_eq!(value, 42);
    }
}
```

**Key points**:
- Use `#[tokio::test]` for async tests
- Use `#[test]` for synchronous tests
- Follow Arrange-Act-Assert pattern
- Use descriptive test names
- Test both success and error cases

### Coverage Expectations

Aim for:
- **Core traits**: 80%+ coverage
- **Implementations**: 70%+ coverage
- **Utilities**: 60%+ coverage

Run coverage:
```bash
# Generate coverage report (requires installation)
cargo tarpaulin --out Html
```

### Test Organization

Organize tests logically:

```rust
#[cfg(test)]
mod tests {
    mod agent_loop {
        mod basic {
            #[test]
            fn test_single_iteration() { }
        }

        mod error_handling {
            #[test]
            fn test_max_iterations_exceeded() { }
        }
    }
}
```

---

## Trait Design Guidelines

### When to Create New Traits

Create a new trait when:
- Multiple types need to implement the same interface
- You want to allow different implementations for testing
- The behavior can be substituted or extended

**Bad** - Too specific:
```rust
pub trait SpecificAgent { }  // Only for one use case
```

**Good** - Reusable:
```rust
pub trait LLMProvider: Send + Sync {
    async fn complete(&self, messages: &[Message]) -> Result<Response>;
}
```

### Trait Composition Patterns

Create composable traits by combining simpler ones:

```rust
// Basic trait
pub trait Named {
    fn name(&self) -> &str;
}

// Extended trait
#[async_trait]
pub trait Agent: Named + Send + Sync {
    async fn execute(&self) -> Result<AgentResponse>;
}

// Combining traits
pub struct MyAgent;

impl Named for MyAgent {
    fn name(&self) -> &str { "MyAgent" }
}

#[async_trait]
impl Agent for MyAgent {
    async fn execute(&self) -> Result<AgentResponse> {
        // Implementation
    }
}
```

### Making Traits Composable and Extensible

Use trait objects and bounds effectively:

```rust
#[async_trait]
pub trait Tool: Send + Sync {
    fn name(&self) -> &str;
    async fn execute(&self, input: &str) -> Result<String>;
}

// Composing traits
pub struct ToolDispatcher {
    tools: Vec<Arc<dyn Tool>>,
}

impl ToolDispatcher {
    pub fn register(&mut self, tool: Arc<dyn Tool>) {
        self.tools.push(tool);
    }
}
```

### Documentation for Custom Traits

Always document traits with examples:

```rust
/// Represents an LLM provider interface.
///
/// Implementations should handle API calls, retries, and error handling.
///
/// # Example
/// ```
/// use zero_core::provider::LLMProvider;
///
/// struct MyProvider;
///
/// #[async_trait]
/// impl LLMProvider for MyProvider {
///     async fn complete(&self, messages: &[Message]) -> Result<Response> {
///         // Implementation
///         todo!()
///     }
/// }
/// ```
#[async_trait]
pub trait LLMProvider: Send + Sync {
    async fn complete(&self, messages: &[Message]) -> Result<Response>;
}
```

---

## Common Pitfalls

### 1. Using unwrap() in Production Code

**Problem**: `unwrap()` panics if the value is `None` or an error:
```rust
// DON'T DO THIS
let result = operation().unwrap();  // Panics on error!
```

**Solution**: Use `Result` and `?` operator:
```rust
// DO THIS
let result = operation()?;  // Propagates error gracefully
```

### 2. Forgetting async-trait Attribute

**Problem**: Async methods in traits require `async-trait`:
```rust
// WON'T COMPILE
pub trait Agent {
    async fn execute(&self) -> Result<()>;  // Error!
}
```

**Solution**: Add the attribute:
```rust
use async_trait::async_trait;

#[async_trait]
pub trait Agent {
    async fn execute(&self) -> Result<()>;  // Compiles!
}
```

### 3. Missing Send + Sync Bounds

**Problem**: Traits used across threads need these bounds:
```rust
pub trait Agent {  // Error with async operations!
    async fn execute(&self) -> Result<()>;
}
```

**Solution**: Add bounds:
```rust
pub trait Agent: Send + Sync {  // Correct!
    async fn execute(&self) -> Result<()>;
}
```

### 4. Performance Issues with Cloning

**Problem**: Cloning expensive objects repeatedly:
```rust
let tool = expensive_tool.clone();  // Expensive if done in loops!
```

**Solution**: Use references or Arc:
```rust
let tool = Arc::new(expensive_tool);
let tool_ref = Arc::clone(&tool);  // Cheap clone of reference
```

### 5. Blocking in Async Code

**Problem**: Using blocking operations in async context:
```rust
#[tokio::main]
async fn main() {
    let data = std::fs::read_to_string("file.txt").unwrap();  // Blocks!
}
```

**Solution**: Use async I/O:
```rust
#[tokio::main]
async fn main() {
    let data = tokio::fs::read_to_string("file.txt").await.unwrap();
}
```

### 6. Inconsistent Error Types

**Problem**: Different error types make handling difficult:
```rust
fn operation1() -> Result<String, String> { }
fn operation2() -> Result<String, io::Error> { }
// Can't use ? operator between them!
```

**Solution**: Use a unified error enum:
```rust
#[derive(Error, Debug)]
pub enum AgentError {
    #[error("IO error: {0}")]
    IoError(#[from] io::Error),
    #[error("Custom error: {0}")]
    Custom(String),
}

fn operation1() -> Result<String, AgentError> { }
fn operation2() -> Result<String, AgentError> { }
```

### 7. Not Running Cargo Check Before Commit

**Problem**: Committing broken code:
```bash
git commit -m "Update code"  # Might not compile!
```

**Solution**: Always verify before committing:
```bash
cargo build --release  # Must pass
cargo clippy          # No warnings
cargo fmt             # Code formatted
cargo test            # All tests pass
git commit -m "feat: update code"
```

---

## Summary of Best Practices

Before submitting a PR, ensure:

✓ Code compiles: `cargo build --release`
✓ No clippy warnings: `cargo clippy`
✓ Properly formatted: `cargo fmt`
✓ All tests pass: `cargo test`
✓ Good documentation: `cargo doc --open`
✓ Descriptive commit messages
✓ No unwrap/panic in production code
✓ Proper error handling with thiserror
✓ async-trait used for async traits
✓ Send + Sync bounds where needed

---

## Getting Help

- **Questions about the project**: See [01-getting-started.md](./01-getting-started.md)
- **Architecture details**: See [03-trait-architecture.md](./03-trait-architecture.md)
- **API reference**: See [05-api-reference.md](./05-api-reference.md)
- **Implementation roadmap**: See `docs/plans/`
- **Code examples**: See `examples/` directory

Thank you for contributing to Zero!
