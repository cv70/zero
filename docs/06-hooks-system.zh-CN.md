# 钩子系统

## 目录

- [概述](#概述)
- [钩子类型对比](#钩子类型对比)
- [AgentHook](#agenthook)
- [ToolHook](#toolhook)
- [ChannelHook](#channelhook)
- [ProviderHook](#providerhook)
- [MemoryHook](#memoryhook)
- [ConfigHook](#confighook)
- [钩子生命周期](#钩子生命周期)
- [最佳实践](#最佳实践)
- [完整示例](#完整示例)

## 概述

Zero Core 中的钩子系统提供了一个强大的基于插件的扩展机制，允许你在执行管道的关键点拦截和自定义行为。与其修改核心代码，钩子能够让你非侵入式地将自定义逻辑注入到 Agent 执行、Tool 操作、Channel 消息、LLM Provider 调用、Memory 访问和配置管理中。

### 为什么使用钩子？

钩子在各种场景中都很有用：

- **可观测性**: 跟踪执行流程、监控性能、收集指标
- **安全性**: 验证输入、执行策略、审计操作
- **转换**: 在操作前后修改数据
- **集成**: 连接外部系统、记录到监控平台
- **速率限制**: 控制资源消耗、实现背压
- **缓存**: 去重操作、提高性能
- **验证**: 执行业务规则、检查约束

### 钩子特征

Zero Core 中的所有钩子都共享以下特征：

- **异步优先**: 使用 `async-trait` 实现，非阻塞
- **非侵入式**: 可以添加而无需修改核心代码
- **可组合**: 可以注册和按顺序执行多个钩子
- **优先级**: 钩子按优先级顺序执行（优先级值越低，执行越早）
- **类型安全**: 利用 Rust 的类型系统确保安全性

## 钩子类型对比

| 钩子类型 | 触发点 | 使用场景 | 错误处理 |
|---------|-------|--------|--------|
| **AgentHook** | Agent 初始化、运行、完成、错误 | 记录执行、收集指标、性能分析 | 返回 `String` 错误 |
| **ToolHook** | Tool 验证、执行、完成、错误 | 输入验证、执行跟踪、性能分析 | 返回 `String` 错误 |
| **ChannelHook** | 消息发送/接收、连接、错误 | 消息转换、过滤、记录 | 返回 `String` 错误 |
| **ProviderHook** | Provider 调用、响应、错误 | Token 计数、缓存、速率限制、记录 | 返回 `ProviderError` |
| **MemoryHook** | Memory 获取/设置/删除、错误 | 访问记录、索引、验证 | 返回 `MemoryError` |
| **ConfigHook** | 配置加载/保存 | 验证、加密、迁移 | 返回 `ConfigResult` |

## AgentHook

AgentHook 允许你监控和扩展 Agent 生命周期事件，包括初始化、执行和错误处理。

### 钩子点

- `on_agent_init(agent_name)` - Agent 初始化前调用
- `on_agent_init_done(agent_name)` - Agent 初始化后调用
- `on_agent_run(agent_name)` - Agent 执行开始前调用
- `on_agent_run_done(agent_name, result)` - Agent 完成并返回结果后调用
- `on_agent_error(agent_name, error)` - 执行期间发生错误时调用

### Trait 定义

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

### 实现示例

```rust
use async_trait::async_trait;
use zero_core::hooks::{Hook, AgentHook};
use std::time::Instant;

/// 监控钩子，跟踪 Agent 执行时间
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
        println!("Agent '{}' 初始化中...", agent_name);
        Ok(())
    }

    async fn on_agent_init_done(&self, agent_name: &str) -> Result<(), String> {
        println!("Agent '{}' 初始化成功", agent_name);
        Ok(())
    }

    async fn on_agent_run(&self, agent_name: &str) -> Result<(), String> {
        println!("Agent '{}' 开始执行", agent_name);
        *self.start_time.lock().unwrap() = Some(Instant::now());
        Ok(())
    }

    async fn on_agent_run_done(&self, agent_name: &str, result: &str) -> Result<(), String> {
        if let Some(start) = *self.start_time.lock().unwrap() {
            let elapsed = start.elapsed();
            println!("Agent '{}' 在 {:?} 内完成", agent_name, elapsed);
            println!("结果: {}", result);
        }
        Ok(())
    }

    async fn on_agent_error(&self, agent_name: &str, error: &str) -> Result<(), String> {
        eprintln!("Agent '{}' 出错: {}", agent_name, error);
        Ok(())
    }
}
```

### 常见模式

- **执行计时**: 测量 Agent 执行花费的时间
- **请求/响应记录**: 记录 Agent 接收和返回的内容
- **指标收集**: 跟踪成功率、错误频率
- **分布式追踪**: 与追踪框架集成

## ToolHook

ToolHook 提供对 Tool 执行的细粒度控制，包括验证和执行跟踪。

### 钩子点

- `on_tool_validate(tool_name, input)` - Tool 输入验证前调用
- `on_tool_validate_done(tool_name, input)` - 验证完成后调用
- `on_tool_execute(tool_name, input)` - Tool 执行前调用
- `on_tool_execute_done(tool_name, input, result)` - 执行完成后调用
- `on_tool_error(tool_name, input, error)` - 出错时调用

### Trait 定义

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

### 实现示例

```rust
use async_trait::async_trait;
use zero_core::hooks::{Hook, ToolHook};
use std::collections::HashMap;
use std::sync::Mutex;

/// Tool 执行指标收集器
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

### 常见模式

- **调用计数**: 跟踪每个 Tool 被调用的次数
- **输入验证**: 在执行前对 Tool 输入强制约束
- **执行分析**: 测量 Tool 性能和延迟
- **错误跟踪**: 监控失败率和错误模式

## ChannelHook

ChannelHook 使你能够拦截和监控消息通道操作，从发送到接收消息。

### 钩子点

- `on_message_send(channel_name, to, content)` - 发送消息前调用
- `on_message_sent(channel_name, to, content)` - 消息发送后调用
- `on_message_receive(channel_name)` - 接收消息前调用
- `on_message_received(channel_name, from, content)` - 接收消息后调用
- `on_channel_error(channel_name, error)` - 通道错误时调用

### Trait 定义

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

### 实现示例

```rust
use async_trait::async_trait;
use zero_core::hooks::{Hook, ChannelHook};

/// Channel 消息日志钩子
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
        println!("[{}] 向 '{}' 发送消息: {}", channel_name, to, content);
        Ok(())
    }

    async fn on_message_sent(
        &self,
        channel_name: &str,
        to: &str,
        _content: &str,
    ) -> Result<(), String> {
        println!("[{}] 消息成功发送至 '{}'", channel_name, to);
        Ok(())
    }

    async fn on_message_received(
        &self,
        channel_name: &str,
        from: &str,
        content: &str,
    ) -> Result<(), String> {
        println!("[{}] 从 '{}' 收到消息: {}", channel_name, from, content);
        Ok(())
    }

    async fn on_channel_error(&self, channel_name: &str, error: &str) -> Result<(), String> {
        eprintln!("[{}] 通道错误: {}", channel_name, error);
        Ok(())
    }
}
```

### 常见模式

- **消息记录**: 记录所有发送和接收的消息
- **消息转换**: 加密/解密或转换消息内容
- **消息过滤**: 阻止或重定向某些消息
- **指标**: 跟踪消息量和延迟

## ProviderHook

ProviderHook 允许你拦截 LLM Provider 调用，实现缓存、速率限制和监控。

### 钩子点

- `on_provider_call(provider_name, request)` - 发出 Provider 请求前调用
- `on_provider_response(provider_name, request, response)` - 接收响应后调用
- `on_provider_error(provider_name, request, error)` - Provider 错误时调用

### Trait 定义

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

### 实现示例

```rust
use async_trait::async_trait;
use zero_core::hooks::{Hook, ProviderHook};
use zero_core::error::ProviderError;
use std::sync::Arc;
use std::sync::Mutex;

/// Token 计数钩子，用于 LLM Provider
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
        // 简单估计: ~4 字符每个 token
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
        println!("请求中估计 token 数: {}", tokens);
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
        println!("响应中估计 token 数: {}", tokens);
        Ok(())
    }
}
```

### 常见模式

- **Token 计数**: 追踪 API 使用情况以便计费
- **响应缓存**: 缓存响应以避免重复 API 调用
- **速率限制**: 对 Provider 调用实施速率限制
- **重试逻辑**: 为失败实现指数退避
- **延迟监控**: 跟踪 Provider 响应时间

## MemoryHook

MemoryHook 使你能够监控和控制对共享内存系统的访问。

### 钩子点

- `on_memory_get(memory_name, key)` - 检索值前调用
- `on_memory_get_done(memory_name, key, value)` - 检索成功后调用
- `on_memory_set(memory_name, key, value)` - 存储值前调用
- `on_memory_set_done(memory_name, key, value)` - 存储成功后调用
- `on_memory_delete(memory_name, key)` - 删除值前调用
- `on_memory_delete_done(memory_name, key, result)` - 删除后调用
- `on_memory_error(memory_name, key, error)` - Memory 错误时调用

### Trait 定义

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

### 实现示例

```rust
use async_trait::async_trait;
use zero_core::hooks::{Hook, MemoryHook};
use zero_core::error::MemoryError;

/// Memory 访问审计钩子
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
        println!("[审计] Memory GET: store='{}', key='{}'", memory_name, key);
        Ok(())
    }

    async fn on_memory_set(&self, memory_name: &str, key: &str, _value: &str) -> Result<(), MemoryError> {
        println!("[审计] Memory SET: store='{}', key='{}'", memory_name, key);
        Ok(())
    }

    async fn on_memory_delete(&self, memory_name: &str, key: &str) -> Result<(), MemoryError> {
        println!("[审计] Memory DELETE: store='{}', key='{}'", memory_name, key);
        Ok(())
    }

    async fn on_memory_error(&self, memory_name: &str, key: &str, error: &str) -> Result<(), MemoryError> {
        eprintln!("[审计] Memory 错误: store='{}', key='{}', error='{}'", memory_name, key, error);
        Ok(())
    }
}
```

### 常见模式

- **访问记录**: 审计谁访问了什么数据
- **加密/解密**: 在存储前加密敏感值
- **索引**: 在 Memory 操作时维护搜索索引
- **验证**: 对存储的值强制约束
- **去重**: 避免存储重复值

## ConfigHook

ConfigHook 为配置加载和保存提供生命周期钩子，对验证和迁移很有用。

### 钩子点

- `before_load()` - 加载配置前调用
- `after_load(value)` - 配置加载后调用
- `before_save()` - 保存配置前调用
- `after_save(value)` - 配置保存后调用

### Trait 定义

```rust
pub trait ConfigHook: Send + Sync {
    fn before_load(&self) -> ConfigResult<()>;

    fn after_load(&self, value: &Value) -> ConfigResult<()>;

    fn before_save(&self) -> ConfigResult<()>;

    fn after_save(&self, value: &Value) -> ConfigResult<()>;
}
```

### 实现示例

```rust
use zero_core::config::hooks::ConfigHook;
use zero_core::config::ConfigResult;
use serde_json::Value;

/// 配置验证钩子
#[derive(Debug, Clone)]
pub struct ConfigValidationHook;

impl ConfigValidationHook {
    pub fn new() -> Self {
        Self
    }

    fn validate_config(&self, config: &Value) -> ConfigResult<()> {
        // 示例: 检查必需字段
        if !config.is_object() {
            return Err("配置必须是 JSON 对象".into());
        }

        // 添加你的验证逻辑
        Ok(())
    }
}

impl ConfigHook for ConfigValidationHook {
    fn before_load(&self) -> ConfigResult<()> {
        println!("准备加载配置...");
        Ok(())
    }

    fn after_load(&self, value: &Value) -> ConfigResult<()> {
        println!("配置已加载，正在验证...");
        self.validate_config(value)
    }

    fn before_save(&self) -> ConfigResult<()> {
        println!("准备保存配置...");
        Ok(())
    }

    fn after_save(&self, value: &Value) -> ConfigResult<()> {
        println!("配置保存成功");
        self.validate_config(value)
    }
}
```

### 常见模式

- **格式验证**: 确保配置与预期模式匹配
- **迁移**: 将配置从旧格式转换为新格式
- **加密**: 加密敏感配置值用于静态存储
- **默认值**: 为缺失的配置键设置默认值
- **环境替换**: 用环境变量替换占位符

## 钩子生命周期

以下图表说明了在典型执行期间何时触发钩子：

```
Agent 执行流程:
┌─────────────────────────────────────────────────────────────┐
│                                                             │
│  on_agent_init() ────→ Agent 设置                           │
│        │                                                    │
│        └──→ on_agent_init_done()                            │
│                                                             │
│  on_agent_run() ────→ Agent 处理                            │
│        │                                                    │
│        ├─→ on_tool_execute() ───→ Tool 执行                │
│        │        │                                           │
│        │        └──→ on_tool_execute_done()                │
│        │                                                    │
│        ├─→ on_message_send() ───→ Channel 操作             │
│        │        │                                           │
│        │        └──→ on_message_sent()                     │
│        │                                                    │
│        ├─→ on_provider_call() ───→ LLM Provider 调用       │
│        │        │                                           │
│        │        └──→ on_provider_response()                │
│        │                                                    │
│        ├─→ on_memory_set() ───→ Memory 存储                │
│        │        │                                           │
│        │        └──→ on_memory_set_done()                  │
│        │                                                    │
│        └──→ on_agent_run_done(result)                      │
│                                                             │
│  或: on_agent_error(error) ─→ 错误处理                     │
│                                                             │
└─────────────────────────────────────────────────────────────┘

钩子执行顺序:
1. 钩子按优先级顺序执行（低值 = 先执行）
2. 同类型的多个钩子按序执行
3. 如果钩子返回错误，执行停止
4. 钩子中的异常不会导致系统崩溃（内部捕获）
```

## 最佳实践

### 1. 保持钩子性能高效

钩子在执行路径中被频繁调用。最小化钩子中的工作量：

```rust
// 好: 快速、非阻塞操作
impl Hook for MyHook {
    async fn on_tool_execute(&self, tool_name: &str, _input: &str) -> Result<(), String> {
        // 只增加计数器
        self.counter.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        Ok(())
    }
}

// 不好: 在钩子中进行昂贵的 I/O
impl Hook for BadHook {
    async fn on_tool_execute(&self, tool_name: &str, _input: &str) -> Result<(), String> {
        // 不要在钩子中进行网络请求!
        let _ = reqwest::get("https://example.com").await;
        Ok(())
    }
}
```

### 2. 优雅地处理错误

始终返回适当的错误类型并提供上下文：

```rust
impl Hook for MyHook {
    async fn on_memory_set(&self, name: &str, key: &str, value: &str) -> Result<(), MemoryError> {
        if value.is_empty() {
            // 返回包含上下文的适当错误
            return Err(MemoryError::ValidationFailed(
                format!("无法为键 '{}' 设置空值", key)
            ));
        }
        Ok(())
    }
}
```

### 3. 使用适当的优先级

根据执行顺序需求设置钩子优先级：

```rust
impl Hook for MyHook {
    fn priority(&self) -> i32 {
        // 验证钩子应该首先运行（优先级值越低越早执行）
        0
    }
}

impl Hook for AuditHook {
    fn priority(&self) -> i32 {
        // 审计应该最后运行（优先级值越高越晚执行）
        100
    }
}
```

### 4. 使钩子可组合

设计钩子能够很好地与其他钩子配合工作：

```rust
// 每个钩子关注单一职责
pub struct ValidationHook; // 验证输入
pub struct LoggingHook;    // 记录操作
pub struct MetricsHook;    // 收集指标

// 它们可以一起注册
hook_manager.register_agent_hook(Box::new(ValidationHook));
hook_manager.register_agent_hook(Box::new(LoggingHook));
hook_manager.register_agent_hook(Box::new(MetricsHook));
```

### 5. 彻底测试钩子

为钩子行为编写测试：

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

### 6. 记录钩子行为

始终记录你的钩子做什么：

```rust
/// 验证所有 Tool 输入的最大长度
///
/// 该钩子强制最大输入长度为 1024 字符。
/// 如果输入超过此限制，它将返回错误。
///
/// 优先级: 0（在链的早期执行）
#[derive(Debug, Clone)]
pub struct InputLengthValidationHook;
```

## 完整示例

这是一个使用多个钩子构建综合监控系统的完整示例：

```rust
use async_trait::async_trait;
use zero_core::hooks::{Hook, AgentHook, ToolHook, ProviderHook};
use zero_core::error::ProviderError;
use std::sync::{Arc, Mutex};
use std::time::Instant;
use std::collections::HashMap;

/// 使用多个钩子的综合监控系统
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
        println!("\n=== 监控报告 ===");
        println!("Agent 执行数: {}", metrics.agent_executions);
        println!("Agent 错误数: {}", metrics.agent_errors);
        println!("Agent 总时间: {}ms", metrics.agent_total_time_ms);
        println!("Provider 调用数: {}", metrics.provider_calls);
        println!("Tool 调用: {:?}", metrics.tool_calls);
        println!("Tool 错误: {:?}", metrics.tool_errors);
    }
}

impl Hook for MonitoringSystem {
    fn name(&self) -> &str {
        "monitoring-system"
    }

    fn priority(&self) -> i32 {
        50  // 在其他钩子之后运行
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

// 使用方法:
// let monitoring = MonitoringSystem::new();
// hook_manager.register_agent_hook(Box::new(monitoring.clone()));
// hook_manager.register_tool_hook(Box::new(monitoring.clone()));
// hook_manager.register_provider_hook(Box::new(monitoring.clone()));
// ... 运行 agents ...
// monitoring.report();
```

## 下一步

- 查阅 [API 参考](./05-api-reference.zh-CN.md) 了解详细的钩子 API 文档
- 查看 [示例](./04-examples.zh-CN.md) 部分了解更多钩子模式
- 参阅 [贡献指南](./07-contributing.zh-CN.md) 了解如何扩展钩子系统
