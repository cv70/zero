# Hooks System

## Table of Contents

- [Overview](#overview)
- [Hook Types Comparison](#hook-types-comparison)
- [AgentHook](#agenthook)
- [ToolHook](#toolhook)
- [ChannelHook](#channelhook)
- [ProviderHook](#providerhook)
- [MemoryHook](#memoryhook)
- [ConfigHook](#confighook)
- [Hook Lifecycle](#hook-lifecycle)
- [Best Practices](#best-practices)
- [Complete Example](#complete-example)

## Overview

The hooks system in Zero Core provides a powerful plugin-based extension mechanism that allows you to intercept and customize behavior at critical points in the execution pipeline. Rather than modifying core code, hooks enable you to inject custom logic non-intrusively into Agent execution, Tool operations, Channel messaging, LLM Provider calls, Memory access, and Configuration management.

### Why Use Hooks?

Hooks are useful for a variety of scenarios:

- **Observability**: Track execution flow, monitor performance, collect metrics
- **Security**: Validate inputs, enforce policies, audit operations
- **Transformation**: Modify data before/after operations
- **Integration**: Connect external systems, log to monitoring platforms
- **Rate Limiting**: Control resource consumption, implement backpressure
- **Caching**: Deduplicate operations, improve performance
- **Validation**: Enforce business rules, check constraints

### Hook Characteristics

All hooks in Zero Core share common characteristics:

- **Async-first**: Implemented using `async-trait`, non-blocking
- **Non-intrusive**: Can be added without modifying core code
- **Composable**: Multiple hooks can be registered and executed in order
- **Prioritized**: Hooks execute in priority order (lower priority value = earlier execution)
- **Type-safe**: Leverages Rust's type system for safety

## Hook Types Comparison

| Hook Type | Trigger Points | Use Cases | Error Handling |
|-----------|---|---|---|
| **AgentHook** | Agent initialization, run, completion, errors | Logging execution, metrics, profiling | Returns `String` error |
| **ToolHook** | Tool validation, execution, completion, errors | Input validation, execution tracking, profiling | Returns `String` error |
| **ChannelHook** | Message send/receive, connection, errors | Message transformation, filtering, logging | Returns `String` error |
| **ProviderHook** | Provider call, response, errors | Token counting, caching, rate limiting, logging | Returns `ProviderError` |
| **MemoryHook** | Memory get/set/delete, errors | Access logging, indexing, validation | Returns `MemoryError` |
| **ConfigHook** | Config load/save | Validation, encryption, migration | Returns `ConfigResult` |

## AgentHook

AgentHook allows you to monitor and extend Agent lifecycle events, including initialization, execution, and error handling.

### Hook Points

- `on_agent_init(agent_name)` - Called before agent initialization
- `on_agent_init_done(agent_name)` - Called after agent initialization
- `on_agent_run(agent_name)` - Called before agent execution starts
- `on_agent_run_done(agent_name, result)` - Called after agent completes with result
- `on_agent_error(agent_name, error)` - Called when an error occurs during execution

### Trait Definition

```rust
#[async_trait]
pub trait AgentHook: Hook {
    async fn on_agent_init(&self, agent_name: &str) -> Result<(), String> {
        Ok(())
    }

    async fn on_agent_init_done(&self, agent_name: &str) -> Result<(), String> {
        Ok(())
    }

    async fn on_agent_run(&self, agent_name: &str) -> Result<(), String> {
        Ok(())
    }

    async fn on_agent_run_done(&self, agent_name: &str, result: &str) -> Result<(), String> {
        Ok(())
    }

    async fn on_agent_error(&self, agent_name: &str, error: &str) -> Result<(), String> {
        Ok(())
    }
}
```

### Implementation Example

```rust
use async_trait::async_trait;
use zero_core::hooks::{Hook, AgentHook};
use std::time::Instant;

/// Monitoring hook that tracks Agent execution time
#[derive(Debug, Clone)]
pub struct AgentMonitoringHook {
    start_time: std::sync::Arc<std::sync::Mutex<Option<Instant>>>,
}

impl AgentMonitoringHook {
    pub fn new() -> Self {
        Self {
            start_time: std::sync::Arc::new(std::sync::Mutex::new(None)),
        }
    }
}

impl Hook for AgentMonitoringHook {
    fn name(&self) -> &str {
        "agent-monitoring"
    }

    fn priority(&self) -> i32 {
        0
    }
}

#[async_trait]
impl AgentHook for AgentMonitoringHook {
    async fn on_agent_init(&self, agent_name: &str) -> Result<(), String> {
        println!("Agent '{}' initializing...", agent_name);
        Ok(())
    }

    async fn on_agent_init_done(&self, agent_name: &str) -> Result<(), String> {
        println!("Agent '{}' initialized successfully", agent_name);
        Ok(())
    }

    async fn on_agent_run(&self, agent_name: &str) -> Result<(), String> {
        println!("Agent '{}' starting execution", agent_name);
        *self.start_time.lock().unwrap() = Some(Instant::now());
        Ok(())
    }

    async fn on_agent_run_done(&self, agent_name: &str, result: &str) -> Result<(), String> {
        if let Some(start) = *self.start_time.lock().unwrap() {
            let elapsed = start.elapsed();
            println!("Agent '{}' completed in {:?}", agent_name, elapsed);
            println!("Result: {}", result);
        }
        Ok(())
    }

    async fn on_agent_error(&self, agent_name: &str, error: &str) -> Result<(), String> {
        eprintln!("Agent '{}' encountered error: {}", agent_name, error);
        Ok(())
    }
}
```

### Common Patterns

- **Execution Timing**: Measure how long Agent execution takes
- **Request/Response Logging**: Log what the agent received and returned
- **Metrics Collection**: Track success rates, error frequencies
- **Distributed Tracing**: Integrate with tracing frameworks

## ToolHook

ToolHook provides fine-grained control over Tool execution, including validation and execution tracking.

### Hook Points

- `on_tool_validate(tool_name, input)` - Called before tool input validation
- `on_tool_validate_done(tool_name, input)` - Called after validation completes
- `on_tool_execute(tool_name, input)` - Called before tool execution
- `on_tool_execute_done(tool_name, input, result)` - Called after execution completes
- `on_tool_error(tool_name, input, error)` - Called when an error occurs

### Trait Definition

```rust
#[async_trait]
pub trait ToolHook: Hook {
    async fn on_tool_validate(&self, tool_name: &str, input: &str) -> Result<(), String> {
        Ok(())
    }

    async fn on_tool_validate_done(&self, tool_name: &str, input: &str) -> Result<(), String> {
        Ok(())
    }

    async fn on_tool_execute(&self, tool_name: &str, input: &str) -> Result<(), String> {
        Ok(())
    }

    async fn on_tool_execute_done(&self, tool_name: &str, input: &str, result: &str) -> Result<(), String> {
        Ok(())
    }

    async fn on_tool_error(&self, tool_name: &str, input: &str, error: &str) -> Result<(), String> {
        Ok(())
    }
}
```

### Implementation Example

```rust
use async_trait::async_trait;
use zero_core::hooks::{Hook, ToolHook};
use std::collections::HashMap;
use std::sync::Mutex;

/// Tool execution metrics collector
#[derive(Debug, Clone)]
pub struct ToolMetricsHook {
    call_counts: std::sync::Arc<Mutex<HashMap<String, u64>>>,
    error_counts: std::sync::Arc<Mutex<HashMap<String, u64>>>,
}

impl ToolMetricsHook {
    pub fn new() -> Self {
        Self {
            call_counts: std::sync::Arc::new(Mutex::new(HashMap::new())),
            error_counts: std::sync::Arc::new(Mutex::new(HashMap::new())),
        }
    }

    pub fn get_stats(&self) -> (HashMap<String, u64>, HashMap<String, u64>) {
        (
            self.call_counts.lock().unwrap().clone(),
            self.error_counts.lock().unwrap().clone(),
        )
    }
}

impl Hook for ToolMetricsHook {
    fn name(&self) -> &str {
        "tool-metrics"
    }

    fn priority(&self) -> i32 {
        10
    }
}

#[async_trait]
impl ToolHook for ToolMetricsHook {
    async fn on_tool_execute(&self, tool_name: &str, _input: &str) -> Result<(), String> {
        let mut counts = self.call_counts.lock().unwrap();
        *counts.entry(tool_name.to_string()).or_insert(0) += 1;
        Ok(())
    }

    async fn on_tool_error(&self, tool_name: &str, _input: &str, _error: &str) -> Result<(), String> {
        let mut counts = self.error_counts.lock().unwrap();
        *counts.entry(tool_name.to_string()).or_insert(0) += 1;
        Ok(())
    }
}
```

### Common Patterns

- **Call Counting**: Track how many times each tool is invoked
- **Input Validation**: Enforce constraints on tool inputs before execution
- **Execution Profiling**: Measure tool performance and latency
- **Error Tracking**: Monitor failure rates and error patterns

## ChannelHook

ChannelHook enables you to intercept and monitor message channel operations, from sending to receiving messages.

### Hook Points

- `on_message_send(channel_name, to, content)` - Called before sending a message
- `on_message_sent(channel_name, to, content)` - Called after message sent
- `on_message_receive(channel_name)` - Called before receiving a message
- `on_message_received(channel_name, from, content)` - Called after message received
- `on_channel_error(channel_name, error)` - Called on channel errors

### Trait Definition

```rust
#[async_trait]
pub trait ChannelHook: Hook {
    async fn on_message_send(&self, channel_name: &str, to: &str, content: &str) -> Result<(), String> {
        Ok(())
    }

    async fn on_message_sent(&self, channel_name: &str, to: &str, content: &str) -> Result<(), String> {
        Ok(())
    }

    async fn on_message_receive(&self, channel_name: &str) -> Result<(), String> {
        Ok(())
    }

    async fn on_message_received(&self, channel_name: &str, from: &str, content: &str) -> Result<(), String> {
        Ok(())
    }

    async fn on_channel_error(&self, channel_name: &str, error: &str) -> Result<(), String> {
        Ok(())
    }
}
```

### Implementation Example

```rust
use async_trait::async_trait;
use zero_core::hooks::{Hook, ChannelHook};

/// Logging hook for channel messages
#[derive(Debug, Clone)]
pub struct ChannelLoggingHook;

impl ChannelLoggingHook {
    pub fn new() -> Self {
        Self
    }
}

impl Hook for ChannelLoggingHook {
    fn name(&self) -> &str {
        "channel-logging"
    }

    fn priority(&self) -> i32 {
        5
    }
}

#[async_trait]
impl ChannelHook for ChannelLoggingHook {
    async fn on_message_send(
        &self,
        channel_name: &str,
        to: &str,
        content: &str,
    ) -> Result<(), String> {
        println!("[{}] Sending message to '{}': {}", channel_name, to, content);
        Ok(())
    }

    async fn on_message_sent(
        &self,
        channel_name: &str,
        to: &str,
        _content: &str,
    ) -> Result<(), String> {
        println!("[{}] Message successfully sent to '{}'", channel_name, to);
        Ok(())
    }

    async fn on_message_received(
        &self,
        channel_name: &str,
        from: &str,
        content: &str,
    ) -> Result<(), String> {
        println!("[{}] Received message from '{}': {}", channel_name, from, content);
        Ok(())
    }

    async fn on_channel_error(&self, channel_name: &str, error: &str) -> Result<(), String> {
        eprintln!("[{}] Channel error: {}", channel_name, error);
        Ok(())
    }
}
```

### Common Patterns

- **Message Logging**: Record all messages sent and received
- **Message Transformation**: Encrypt/decrypt or transform message content
- **Message Filtering**: Block or redirect certain messages
- **Metrics**: Track message volume and latency

## ProviderHook

ProviderHook allows you to intercept LLM provider calls, enabling caching, rate limiting, and monitoring.

### Hook Points

- `on_provider_call(provider_name, request)` - Called before making a provider request
- `on_provider_response(provider_name, request, response)` - Called after receiving response
- `on_provider_error(provider_name, request, error)` - Called on provider errors

### Trait Definition

```rust
#[async_trait]
pub trait ProviderHook: Hook {
    async fn on_provider_call(&self, provider_name: &str, request: &str) -> Result<(), ProviderError> {
        Ok(())
    }

    async fn on_provider_response(
        &self,
        provider_name: &str,
        request: &str,
        response: &str,
    ) -> Result<(), ProviderError> {
        Ok(())
    }

    async fn on_provider_error(
        &self,
        provider_name: &str,
        request: &str,
        error: &str,
    ) -> Result<(), ProviderError> {
        Ok(())
    }
}
```

### Implementation Example

```rust
use async_trait::async_trait;
use zero_core::hooks::{Hook, ProviderHook};
use zero_core::error::ProviderError;
use std::sync::Arc;
use std::sync::Mutex;

/// Token counting hook for LLM providers
#[derive(Debug, Clone)]
pub struct TokenCountingHook {
    total_tokens: Arc<Mutex<u64>>,
}

impl TokenCountingHook {
    pub fn new() -> Self {
        Self {
            total_tokens: Arc::new(Mutex::new(0)),
        }
    }

    pub fn get_token_count(&self) -> u64 {
        *self.total_tokens.lock().unwrap()
    }

    fn estimate_tokens(text: &str) -> u64 {
        // Simple estimation: ~4 characters per token
        (text.len() as u64 + 3) / 4
    }
}

impl Hook for TokenCountingHook {
    fn name(&self) -> &str {
        "token-counting"
    }

    fn priority(&self) -> i32 {
        0
    }
}

#[async_trait]
impl ProviderHook for TokenCountingHook {
    async fn on_provider_call(&self, _provider_name: &str, request: &str) -> Result<(), ProviderError> {
        let tokens = Self::estimate_tokens(request);
        let mut total = self.total_tokens.lock().unwrap();
        *total += tokens;
        println!("Estimated tokens in request: {}", tokens);
        Ok(())
    }

    async fn on_provider_response(
        &self,
        _provider_name: &str,
        _request: &str,
        response: &str,
    ) -> Result<(), ProviderError> {
        let tokens = Self::estimate_tokens(response);
        let mut total = self.total_tokens.lock().unwrap();
        *total += tokens;
        println!("Estimated tokens in response: {}", tokens);
        Ok(())
    }
}
```

### Common Patterns

- **Token Counting**: Track API usage for billing
- **Response Caching**: Cache responses to avoid duplicate API calls
- **Rate Limiting**: Enforce rate limits on provider calls
- **Retry Logic**: Implement exponential backoff for failures
- **Latency Monitoring**: Track provider response times

## MemoryHook

MemoryHook enables monitoring and controlling access to the shared memory system.

### Hook Points

- `on_memory_get(memory_name, key)` - Called before retrieving a value
- `on_memory_get_done(memory_name, key, value)` - Called after successful retrieval
- `on_memory_set(memory_name, key, value)` - Called before storing a value
- `on_memory_set_done(memory_name, key, value)` - Called after successful storage
- `on_memory_delete(memory_name, key)` - Called before deleting a value
- `on_memory_delete_done(memory_name, key, result)` - Called after deletion
- `on_memory_error(memory_name, key, error)` - Called on memory errors

### Trait Definition

```rust
#[async_trait]
pub trait MemoryHook: Hook {
    async fn on_memory_get(&self, memory_name: &str, key: &str) -> Result<(), MemoryError> {
        Ok(())
    }

    async fn on_memory_get_done(&self, memory_name: &str, key: &str, value: &str) -> Result<(), MemoryError> {
        Ok(())
    }

    async fn on_memory_set(&self, memory_name: &str, key: &str, value: &str) -> Result<(), MemoryError> {
        Ok(())
    }

    async fn on_memory_set_done(&self, memory_name: &str, key: &str, value: &str) -> Result<(), MemoryError> {
        Ok(())
    }

    async fn on_memory_delete(&self, memory_name: &str, key: &str) -> Result<(), MemoryError> {
        Ok(())
    }

    async fn on_memory_delete_done(&self, memory_name: &str, key: &str, result: &str) -> Result<(), MemoryError> {
        Ok(())
    }

    async fn on_memory_error(&self, memory_name: &str, key: &str, error: &str) -> Result<(), MemoryError> {
        Ok(())
    }
}
```

### Implementation Example

```rust
use async_trait::async_trait;
use zero_core::hooks::{Hook, MemoryHook};
use zero_core::error::MemoryError;

/// Audit hook for memory access
#[derive(Debug, Clone)]
pub struct MemoryAuditHook;

impl MemoryAuditHook {
    pub fn new() -> Self {
        Self
    }
}

impl Hook for MemoryAuditHook {
    fn name(&self) -> &str {
        "memory-audit"
    }

    fn priority(&self) -> i32 {
        0
    }
}

#[async_trait]
impl MemoryHook for MemoryAuditHook {
    async fn on_memory_get(&self, memory_name: &str, key: &str) -> Result<(), MemoryError> {
        println!("[AUDIT] Memory GET: store='{}', key='{}'", memory_name, key);
        Ok(())
    }

    async fn on_memory_set(&self, memory_name: &str, key: &str, _value: &str) -> Result<(), MemoryError> {
        println!("[AUDIT] Memory SET: store='{}', key='{}'", memory_name, key);
        Ok(())
    }

    async fn on_memory_delete(&self, memory_name: &str, key: &str) -> Result<(), MemoryError> {
        println!("[AUDIT] Memory DELETE: store='{}', key='{}'", memory_name, key);
        Ok(())
    }

    async fn on_memory_error(&self, memory_name: &str, key: &str, error: &str) -> Result<(), MemoryError> {
        eprintln!("[AUDIT] Memory ERROR: store='{}', key='{}', error='{}'", memory_name, key, error);
        Ok(())
    }
}
```

### Common Patterns

- **Access Logging**: Audit who accessed what data
- **Encryption/Decryption**: Encrypt sensitive values before storage
- **Indexing**: Maintain search indexes on memory operations
- **Validation**: Enforce constraints on stored values
- **Deduplication**: Avoid storing duplicate values

## ConfigHook

ConfigHook provides lifecycle hooks for configuration loading and saving, useful for validation and migration.

### Hook Points

- `before_load()` - Called before loading configuration
- `after_load(value)` - Called after configuration is loaded
- `before_save()` - Called before saving configuration
- `after_save(value)` - Called after configuration is saved

### Trait Definition

```rust
pub trait ConfigHook: Send + Sync {
    fn before_load(&self) -> ConfigResult<()>;

    fn after_load(&self, value: &Value) -> ConfigResult<()>;

    fn before_save(&self) -> ConfigResult<()>;

    fn after_save(&self, value: &Value) -> ConfigResult<()>;
}
```

### Implementation Example

```rust
use zero_core::config::hooks::ConfigHook;
use zero_core::config::ConfigResult;
use serde_json::Value;

/// Validation hook for configuration
#[derive(Debug, Clone)]
pub struct ConfigValidationHook;

impl ConfigValidationHook {
    pub fn new() -> Self {
        Self
    }

    fn validate_config(&self, config: &Value) -> ConfigResult<()> {
        // Example: check required fields
        if !config.is_object() {
            return Err("Configuration must be a JSON object".into());
        }

        // Add your validation logic here
        Ok(())
    }
}

impl ConfigHook for ConfigValidationHook {
    fn before_load(&self) -> ConfigResult<()> {
        println!("Preparing to load configuration...");
        Ok(())
    }

    fn after_load(&self, value: &Value) -> ConfigResult<()> {
        println!("Configuration loaded, validating...");
        self.validate_config(value)
    }

    fn before_save(&self) -> ConfigResult<()> {
        println!("Preparing to save configuration...");
        Ok(())
    }

    fn after_save(&self, value: &Value) -> ConfigResult<()> {
        println!("Configuration saved successfully");
        self.validate_config(value)
    }
}
```

### Common Patterns

- **Format Validation**: Ensure configuration matches expected schema
- **Migration**: Transform configuration from old to new formats
- **Encryption**: Encrypt sensitive configuration values at rest
- **Default Values**: Set defaults for missing configuration keys
- **Environment Substitution**: Replace placeholders with environment variables

## Hook Lifecycle

The following diagram illustrates when hooks fire during typical execution:

```
Agent Execution Flow:
┌─────────────────────────────────────────────────────────────┐
│                                                             │
│  on_agent_init() ────→ Agent Setup                          │
│        │                                                    │
│        └──→ on_agent_init_done()                            │
│                                                             │
│  on_agent_run() ────→ Agent Processing                      │
│        │                                                    │
│        ├─→ on_tool_execute() ───→ Tool Execution           │
│        │        │                                           │
│        │        └──→ on_tool_execute_done()                │
│        │                                                    │
│        ├─→ on_message_send() ───→ Channel Operation        │
│        │        │                                           │
│        │        └──→ on_message_sent()                     │
│        │                                                    │
│        ├─→ on_provider_call() ───→ LLM Provider Call       │
│        │        │                                           │
│        │        └──→ on_provider_response()                │
│        │                                                    │
│        ├─→ on_memory_set() ───→ Memory Storage             │
│        │        │                                           │
│        │        └──→ on_memory_set_done()                  │
│        │                                                    │
│        └──→ on_agent_run_done(result)                      │
│                                                             │
│  OR: on_agent_error(error) ─→ Error Handling               │
│                                                             │
└─────────────────────────────────────────────────────────────┘

Hook Execution Order:
1. Hooks execute in priority order (lower = earlier)
2. Multiple hooks of same type execute sequentially
3. If a hook returns an error, execution stops
4. Exceptions in hooks don't crash the system (caught internally)
```

## Best Practices

### 1. Keep Hooks Performant

Hooks are called frequently in the execution path. Minimize work in hooks:

```rust
// GOOD: Fast, non-blocking operation
impl Hook for MyHook {
    async fn on_tool_execute(&self, tool_name: &str, _input: &str) -> Result<(), String> {
        // Just increment a counter
        self.counter.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        Ok(())
    }
}

// BAD: Expensive I/O in hook
impl Hook for BadHook {
    async fn on_tool_execute(&self, tool_name: &str, _input: &str) -> Result<(), String> {
        // Don't make network requests in hooks!
        let _ = reqwest::get("https://example.com").await;
        Ok(())
    }
}
```

### 2. Handle Errors Gracefully

Always return appropriate error types and provide context:

```rust
impl Hook for MyHook {
    async fn on_memory_set(&self, name: &str, key: &str, value: &str) -> Result<(), MemoryError> {
        if value.is_empty() {
            // Return a proper error with context
            return Err(MemoryError::ValidationFailed(
                format!("Cannot set empty value for key '{}'", key)
            ));
        }
        Ok(())
    }
}
```

### 3. Use Appropriate Priority

Set hook priority based on execution order needs:

```rust
impl Hook for MyHook {
    fn priority(&self) -> i32 {
        // Validation hooks should run first (higher priority = lower number)
        0
    }
}

impl Hook for AuditHook {
    fn priority(&self) -> i32 {
        // Audit should run last (lower priority = higher number)
        100
    }
}
```

### 4. Make Hooks Composable

Design hooks to work well with other hooks:

```rust
// Each hook focuses on a single responsibility
pub struct ValidationHook; // Validates inputs
pub struct LoggingHook;    // Logs operations
pub struct MetricsHook;    // Collects metrics

// They can be registered together
hook_manager.register_agent_hook(Box::new(ValidationHook));
hook_manager.register_agent_hook(Box::new(LoggingHook));
hook_manager.register_agent_hook(Box::new(MetricsHook));
```

### 5. Test Hooks Thoroughly

Write tests for hook behavior:

```rust
#[tokio::test]
async fn test_counter_increments() {
    let hook = CounterHook::new();
    hook.on_tool_execute("test_tool", "input").await.unwrap();
    hook.on_tool_execute("test_tool", "input").await.unwrap();
    assert_eq!(hook.count(), 2);
}

#[tokio::test]
async fn test_error_on_invalid_input() {
    let hook = ValidationHook::new();
    let result = hook.on_tool_validate("test", "").await;
    assert!(result.is_err());
}
```

### 6. Document Hook Behavior

Always document what your hooks do:

```rust
/// Validates all tool inputs for maximum length
///
/// This hook enforces a maximum input length of 1024 characters.
/// It will return an error if an input exceeds this limit.
///
/// Priority: 0 (executes early in the chain)
#[derive(Debug, Clone)]
pub struct InputLengthValidationHook;
```

## Complete Example

Here's a complete example of building a comprehensive monitoring system using hooks:

```rust
use async_trait::async_trait;
use zero_core::hooks::{Hook, AgentHook, ToolHook, ProviderHook};
use zero_core::error::ProviderError;
use std::sync::{Arc, Mutex};
use std::time::Instant;
use std::collections::HashMap;

/// Comprehensive monitoring system using multiple hooks
#[derive(Debug, Clone)]
pub struct MonitoringSystem {
    metrics: Arc<Mutex<MetricsData>>,
}

#[derive(Debug, Clone, Default)]
struct MetricsData {
    agent_executions: u64,
    agent_errors: u64,
    agent_total_time_ms: u64,
    tool_calls: HashMap<String, u64>,
    tool_errors: HashMap<String, u64>,
    provider_calls: u64,
    last_agent_start: Option<Instant>,
}

impl MonitoringSystem {
    pub fn new() -> Self {
        Self {
            metrics: Arc::new(Mutex::new(MetricsData::default())),
        }
    }

    pub fn report(&self) {
        let metrics = self.metrics.lock().unwrap();
        println!("\n=== Monitoring Report ===");
        println!("Agent Executions: {}", metrics.agent_executions);
        println!("Agent Errors: {}", metrics.agent_errors);
        println!("Agent Total Time: {}ms", metrics.agent_total_time_ms);
        println!("Provider Calls: {}", metrics.provider_calls);
        println!("Tool Calls: {:?}", metrics.tool_calls);
        println!("Tool Errors: {:?}", metrics.tool_errors);
    }
}

impl Hook for MonitoringSystem {
    fn name(&self) -> &str {
        "monitoring-system"
    }

    fn priority(&self) -> i32 {
        50  // Run after other hooks
    }
}

#[async_trait]
impl AgentHook for MonitoringSystem {
    async fn on_agent_run(&self, _agent_name: &str) -> Result<(), String> {
        let mut metrics = self.metrics.lock().unwrap();
        metrics.agent_executions += 1;
        metrics.last_agent_start = Some(Instant::now());
        Ok(())
    }

    async fn on_agent_run_done(&self, _agent_name: &str, _result: &str) -> Result<(), String> {
        let mut metrics = self.metrics.lock().unwrap();
        if let Some(start) = metrics.last_agent_start {
            metrics.agent_total_time_ms += start.elapsed().as_millis() as u64;
        }
        Ok(())
    }

    async fn on_agent_error(&self, _agent_name: &str, _error: &str) -> Result<(), String> {
        let mut metrics = self.metrics.lock().unwrap();
        metrics.agent_errors += 1;
        Ok(())
    }
}

#[async_trait]
impl ToolHook for MonitoringSystem {
    async fn on_tool_execute(&self, tool_name: &str, _input: &str) -> Result<(), String> {
        let mut metrics = self.metrics.lock().unwrap();
        *metrics.tool_calls.entry(tool_name.to_string()).or_insert(0) += 1;
        Ok(())
    }

    async fn on_tool_error(&self, tool_name: &str, _input: &str, _error: &str) -> Result<(), String> {
        let mut metrics = self.metrics.lock().unwrap();
        *metrics.tool_errors.entry(tool_name.to_string()).or_insert(0) += 1;
        Ok(())
    }
}

#[async_trait]
impl ProviderHook for MonitoringSystem {
    async fn on_provider_call(&self, _provider_name: &str, _request: &str) -> Result<(), ProviderError> {
        let mut metrics = self.metrics.lock().unwrap();
        metrics.provider_calls += 1;
        Ok(())
    }
}

// Usage:
// let monitoring = MonitoringSystem::new();
// hook_manager.register_agent_hook(Box::new(monitoring.clone()));
// hook_manager.register_tool_hook(Box::new(monitoring.clone()));
// hook_manager.register_provider_hook(Box::new(monitoring.clone()));
// ... run agents ...
// monitoring.report();
```

## Next Steps

- Review the [API Reference](./05-api-reference.md) for detailed hook API documentation
- Check out the [Examples](./04-examples.md) section for more hook patterns
- See [Contributing Guide](./07-contributing.md) for guidelines on extending the hooks system
