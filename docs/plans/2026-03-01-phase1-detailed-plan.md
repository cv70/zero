# Phase 1: 核心循环与工具调度 - 详细实现计划

## 概述

Phase 1 包含 S1 和 S2，目标是实现一个完整的 Agent 循环，支持多工具调度和执行。

**时间估计**: 2 周
**关键输出**: 可以执行多种工具的独立 Agent

---

## S1: Agent Loop Core

### 1.1 核心数据结构

#### Message 系统

```rust
// zero-core/src/message.rs (新文件)

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Message {
    /// 用户消息
    User { content: String },

    /// Assistant 消息
    Assistant {
        content: Vec<ContentBlock>,
    },

    /// 工具结果
    ToolResult {
        tool_use_id: String,
        content: String,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum ContentBlock {
    #[serde(rename = "text")]
    Text { text: String },

    #[serde(rename = "tool_use")]
    ToolUse {
        id: String,
        name: String,
        input: serde_json::Value,
    },
}

impl Message {
    pub fn user(content: impl Into<String>) -> Self {
        Message::User {
            content: content.into(),
        }
    }

    pub fn assistant(blocks: Vec<ContentBlock>) -> Self {
        Message::Assistant { content: blocks }
    }

    pub fn tool_result(id: String, content: impl Into<String>) -> Self {
        Message::ToolResult {
            tool_use_id: id,
            content: content.into(),
        }
    }
}
```

#### AgentLoopConfig

```rust
// zero-core/src/agent/loop_config.rs (新文件)

#[derive(Debug, Clone)]
pub struct AgentLoopConfig {
    /// 最大迭代次数
    pub max_iterations: usize,

    /// 单个 LLM 调用超时（秒）
    pub provider_timeout: u64,

    /// 工具执行超时（秒）
    pub tool_timeout: u64,

    /// 是否启用钩子
    pub enable_hooks: bool,

    /// 是否保存消息历史
    pub save_history: bool,
}

impl Default for AgentLoopConfig {
    fn default() -> Self {
        Self {
            max_iterations: 100,
            provider_timeout: 300,
            tool_timeout: 120,
            enable_hooks: true,
            save_history: true,
        }
    }
}
```

### 1.2 AgentLoop Trait 扩展

```rust
// zero-core/src/agent/trait.rs (修改)

#[async_trait]
pub trait Agent: Send + Sync {
    // ... 现有方法 ...

    /// 执行 Agent，返回最终响应
    async fn execute(
        &self,
        messages: &mut Vec<Message>,
        config: AgentLoopConfig,
    ) -> Result<AgentResponse, AgentError>;
}

#[derive(Debug, Clone)]
pub struct AgentResponse {
    pub content: String,
    pub tool_calls: Vec<ToolCall>,
    pub stop_reason: String,
    pub metadata: HashMap<String, String>,
}
```

### 1.3 AgentLoop 实现

```rust
// zero-core/src/agent/agent_loop.rs (新文件)

use async_trait::async_trait;
use std::sync::Arc;

#[async_trait]
pub trait AgentLoop: Send + Sync {
    /// 执行 Agent 循环
    async fn execute(
        &self,
        messages: &mut Vec<Message>,
        config: &AgentLoopConfig,
    ) -> Result<String, AgentError>;
}

pub struct DefaultAgentLoop {
    provider: Arc<dyn LLMProvider>,
    tool_dispatcher: Arc<dyn ToolDispatcher>,
    hooks: Option<Arc<HookManager>>,
}

impl DefaultAgentLoop {
    pub fn new(
        provider: Arc<dyn LLMProvider>,
        tool_dispatcher: Arc<dyn ToolDispatcher>,
    ) -> Self {
        Self {
            provider,
            tool_dispatcher,
            hooks: None,
        }
    }

    pub fn with_hooks(mut self, hooks: Arc<HookManager>) -> Self {
        self.hooks = Some(hooks);
        self
    }
}

#[async_trait]
impl AgentLoop for DefaultAgentLoop {
    async fn execute(
        &self,
        messages: &mut Vec<Message>,
        config: &AgentLoopConfig,
    ) -> Result<String, AgentError> {
        let mut iteration = 0;
        let mut final_response = String::new();

        loop {
            if iteration >= config.max_iterations {
                return Err(AgentError::MaxIterationsExceeded);
            }
            iteration += 1;

            // 钩子：循环开始前
            if let Some(ref hooks) = self.hooks {
                hooks.fire_before_provider_call(&messages)?;
            }

            // 调用 LLM
            let response = tokio::time::timeout(
                std::time::Duration::from_secs(config.provider_timeout),
                self.provider.complete(messages),
            )
            .await
            .map_err(|_| AgentError::ProviderTimeout)?
            .map_err(|e| AgentError::ProviderError(e.to_string()))?;

            // 追加 Assistant 消息
            messages.push(Message::Assistant {
                content: response.content.clone(),
            });

            // 提取文本响应
            for block in &response.content {
                if let ContentBlock::Text { text } = block {
                    final_response = text.clone();
                }
            }

            // 检查是否需要执行工具
            if response.stop_reason != "tool_use" {
                return Ok(final_response);
            }

            // 执行工具
            let mut results = Vec::new();
            for block in response.content {
                if let ContentBlock::ToolUse { id, name, input } = block {
                    // 钩子：工具执行前
                    if let Some(ref hooks) = self.hooks {
                        hooks.fire_before_tool_execution(&name, &input)?;
                    }

                    let tool_call = ToolCall {
                        id: id.clone(),
                        name: name.clone(),
                        arguments: serde_json::to_string(&input)
                            .map_err(|e| AgentError::SerializationError(e.to_string()))?,
                    };

                    let result = tokio::time::timeout(
                        std::time::Duration::from_secs(config.tool_timeout),
                        self.tool_dispatcher.execute(tool_call),
                    )
                    .await
                    .map_err(|_| AgentError::ToolTimeout)?
                    .unwrap_or_else(|e| format!("Tool error: {}", e));

                    // 钩子：工具执行后
                    if let Some(ref hooks) = self.hooks {
                        hooks.fire_after_tool_execution(&name, &result)?;
                    }

                    results.push(Message::ToolResult {
                        tool_use_id: id,
                        content: result,
                    });
                }
            }

            // 追加工具结果
            messages.extend(results);

            // 钩子：循环迭代完成
            if let Some(ref hooks) = self.hooks {
                hooks.fire_after_iteration(iteration)?;
            }
        }
    }
}
```

### 1.4 测试

```rust
// zero-core/src/agent/tests/loop_tests.rs (新文件)

#[cfg(test)]
mod tests {
    use super::*;
    use mockall::predicate::*;
    use mockall::mock;

    mock! {
        MockProvider {}

        #[async_trait]
        impl LLMProvider for MockProvider {
            async fn complete(&self, messages: &[Message]) -> Result<ProviderResponse, ProviderError>;
        }
    }

    #[tokio::test]
    async fn test_simple_loop_single_response() {
        let mut mock_provider = MockProvider::new();
        mock_provider
            .expect_complete()
            .times(1)
            .returning(|_| {
                Ok(ProviderResponse {
                    content: vec![ContentBlock::Text {
                        text: "Hello, world!".to_string(),
                    }],
                    stop_reason: "end_turn".to_string(),
                })
            });

        let dispatcher = Arc::new(MockToolDispatcher::new());
        let loop_impl = DefaultAgentLoop::new(
            Arc::new(mock_provider),
            dispatcher,
        );

        let mut messages = vec![Message::User {
            content: "Say hello".to_string(),
        }];

        let result = loop_impl
            .execute(&mut messages, &AgentLoopConfig::default())
            .await;

        assert!(result.is_ok());
        assert_eq!(messages.len(), 2); // user + assistant
    }

    #[tokio::test]
    async fn test_loop_with_tool_execution() {
        // 省略详细实现...
    }
}
```

---

## S2: Tool Dispatcher

### 2.1 扩展 Tool Trait

```rust
// zero-core/src/tool/trait.rs (修改)

#[derive(Debug, Clone)]
pub struct ToolCall {
    pub id: String,
    pub name: String,
    pub arguments: String,  // JSON 字符串
}

impl ToolCall {
    pub fn parse_arguments<T: serde::de::DeserializeOwned>(
        &self,
    ) -> Result<T, serde_json::Error> {
        serde_json::from_str(&self.arguments)
    }
}

#[async_trait]
pub trait ToolDispatcher: Send + Sync {
    /// 执行工具调用
    async fn execute(&self, call: ToolCall) -> Result<String, ToolError>;

    /// 获取可用工具列表
    fn get_tools(&self) -> Vec<ToolSchema>;
}
```

### 2.2 Tool Registry 改进

```rust
// zero-core/src/tool/registry.rs (修改)

pub struct ToolRegistry {
    tools: Arc<RwLock<HashMap<String, Arc<dyn Tool>>>>,
}

impl ToolRegistry {
    pub fn new() -> Self {
        Self {
            tools: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// 注册工具
    pub async fn register(&self, name: impl Into<String>, tool: Arc<dyn Tool>) {
        let mut tools = self.tools.write().await;
        tools.insert(name.into(), tool);
    }

    /// 获取工具
    pub async fn get(&self, name: &str) -> Option<Arc<dyn Tool>> {
        let tools = self.tools.read().await;
        tools.get(name).cloned()
    }

    /// 列出所有工具
    pub async fn list_tools(&self) -> Vec<ToolSchema> {
        let tools = self.tools.read().await;
        tools.values().map(|t| t.schema()).collect()
    }
}

#[async_trait]
impl ToolDispatcher for ToolRegistry {
    async fn execute(&self, call: ToolCall) -> Result<String, ToolError> {
        let tool = self
            .get(&call.name)
            .await
            .ok_or_else(|| ToolError::ToolNotFound(call.name.clone()))?;

        tool.execute(&call.arguments).await
    }

    fn get_tools(&self) -> Vec<ToolSchema> {
        // 需要阻塞版本或异步处理
        // 这是一个设计权衡
    }
}
```

### 2.3 内置工具实现

```rust
// zero-core/src/tool/builtins/bash.rs (新文件)

pub struct BashTool {
    allowed_commands: Vec<String>,
    blocked_patterns: Vec<String>,
}

#[async_trait]
impl Tool for BashTool {
    fn name(&self) -> &str {
        "bash"
    }

    fn schema(&self) -> ToolSchema {
        ToolSchema {
            name: "bash".to_string(),
            description: "Run shell commands".to_string(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "command": {
                        "type": "string",
                        "description": "Shell command to execute"
                    }
                },
                "required": ["command"]
            }),
        }
    }

    async fn execute(&self, arguments: &str) -> Result<String, ToolError> {
        #[derive(Deserialize)]
        struct Args {
            command: String,
        }

        let args: Args = serde_json::from_str(arguments)
            .map_err(|e| ToolError::InvalidArguments(e.to_string()))?;

        // 安全检查
        for pattern in &self.blocked_patterns {
            if args.command.contains(pattern) {
                return Err(ToolError::ExecutionError(
                    "Dangerous command blocked".to_string()
                ));
            }
        }

        let output = std::process::Command::new("sh")
            .arg("-c")
            .arg(&args.command)
            .output()
            .map_err(|e| ToolError::ExecutionError(e.to_string()))?;

        let result = String::from_utf8_lossy(&output.stdout).to_string()
            + &String::from_utf8_lossy(&output.stderr);

        Ok(result.chars().take(50000).collect())
    }
}
```

```rust
// zero-core/src/tool/builtins/file.rs (新文件)

pub struct FileReadTool {
    root_dir: PathBuf,
}

#[async_trait]
impl Tool for FileReadTool {
    fn name(&self) -> &str {
        "read_file"
    }

    fn schema(&self) -> ToolSchema {
        // ... schema定义 ...
    }

    async fn execute(&self, arguments: &str) -> Result<String, ToolError> {
        #[derive(Deserialize)]
        struct Args {
            path: String,
            #[serde(default)]
            limit: Option<usize>,
        }

        let args: Args = serde_json::from_str(arguments)?;
        let file_path = (self.root_dir.clone() / &args.path).canonicalize()?;

        // 安全检查：确保路径在根目录内
        if !file_path.starts_with(&self.root_dir) {
            return Err(ToolError::ExecutionError(
                "Path escapes workspace".to_string()
            ));
        }

        let content = std::fs::read_to_string(&file_path)?;
        let lines: Vec<&str> = content.lines().collect();

        let limited = if let Some(limit) = args.limit {
            lines.iter().take(limit).map(|s| s.to_string()).collect::<Vec<_>>()
        } else {
            lines.iter().map(|s| s.to_string()).collect()
        };

        Ok(limited.join("\n"))
    }
}

pub struct FileWriteTool {
    root_dir: PathBuf,
}

// 类似实现...

pub struct FileEditTool {
    root_dir: PathBuf,
}

// 类似实现...
```

### 2.4 测试

```rust
// zero-core/src/tool/tests/dispatcher_tests.rs (新文件)

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_tool_dispatch() {
        let registry = ToolRegistry::new();

        // 注册工具
        registry.register(
            "bash",
            Arc::new(BashTool::new()),
        ).await;

        // 执行工具
        let call = ToolCall {
            id: "1".to_string(),
            name: "bash".to_string(),
            arguments: r#"{"command": "echo hello"}"#.to_string(),
        };

        let result = registry.execute(call).await;
        assert!(result.is_ok());
    }
}
```

---

## 1.5 mod.rs 更新

```rust
// zero-core/src/agent/mod.rs

pub mod trait;
pub mod context;
pub mod hooked_agent;
pub mod hook;
pub mod loop_config;      // 新
pub mod agent_loop;       // 新
pub mod message;          // 新

pub use trait::{Agent, AgentResponse, ToolCall};
pub use context::AgentContext;
pub use hooked_agent::HookedAgent;
pub use loop_config::AgentLoopConfig;
pub use agent_loop::{AgentLoop, DefaultAgentLoop};
pub use message::{Message, ContentBlock};
```

```rust
// zero-core/src/tool/mod.rs

pub mod trait;
pub mod registry;
pub mod metadata;
pub mod hook;
pub mod dispatcher;        // 新
pub mod builtins;          // 新

pub mod builtins {
    pub mod bash;
    pub mod file;
}

pub use trait::{Tool, ToolSchema, ToolDispatcher, ToolCall};
pub use registry::ToolRegistry;
pub use dispatcher::*;
```

---

## 集成点

### mod.rs 顶级更新

```rust
// zero-core/src/lib.rs

pub mod message;           // 新

pub use message::{Message, ContentBlock};
```

---

## 验收清单

### S1: Agent Loop
- [ ] Message 序列化/反序列化完整
- [ ] AgentLoop Trait 实现
- [ ] DefaultAgentLoop 正确处理循环和超时
- [ ] 钩子集成完整
- [ ] 单元测试覆盖 > 80%
- [ ] `cargo build` 通过
- [ ] `cargo test` 全部通过

### S2: Tool Dispatch
- [ ] ToolDispatcher Trait 实现
- [ ] ToolRegistry 完整功能
- [ ] 至少 4 个内置工具（bash, read, write, edit）
- [ ] 参数验证和安全检查
- [ ] 单元测试覆盖 > 80%
- [ ] `cargo build` 通过
- [ ] `cargo test` 全部通过

---

## 文件清单

### 新增文件

```
zero-core/src/
├── message.rs                        (新)
├── agent/
│   ├── loop_config.rs               (新)
│   ├── agent_loop.rs                (新)
│   └── tests/
│       └── loop_tests.rs            (新)
├── tool/
│   ├── dispatcher.rs                (新)
│   ├── builtins/
│   │   ├── mod.rs                   (新)
│   │   ├── bash.rs                  (新)
│   │   └── file.rs                  (新)
│   └── tests/
│       └── dispatcher_tests.rs       (新)
```

### 修改文件

```
zero-core/src/
├── lib.rs                           (添加 message 模块)
├── agent/
│   ├── mod.rs                       (导出新模块)
│   └── trait.rs                     (更新 AgentResponse)
└── tool/
    ├── mod.rs                       (导出新模块)
    └── trait.rs                     (添加 ToolDispatcher, ToolCall)
```

---

## 关键依赖

确保 `Cargo.toml` 包含：

```toml
[dependencies]
tokio = { version = "1", features = ["full"] }
serde = { version = "1", features = ["derive"] }
serde_json = "1"
async-trait = "0.1"
thiserror = "1"

[dev-dependencies]
mockall = "0.12"
```

---

## 下一步

完成 Phase 1 后：
1. 创建示例 Agent
2. 集成完整的端到端测试
3. 性能基准测试
4. 文档和 API 参考
5. 开始 Phase 2 (Planning System)

