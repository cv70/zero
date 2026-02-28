use crate::error::ToolError;
use async_trait::async_trait;
use serde_json::Value;

/// Tool 输出类型
#[derive(Debug, Clone)]
pub enum ToolOutput {
    Text(String),
    Image { data: Vec<u8>, mime_type: String },
    Video { data: Vec<u8>, mime_type: String },
    Audio { data: Vec<u8>, mime_type: String },
}

impl ToolOutput {
    pub fn text(s: impl Into<String>) -> Self {
        ToolOutput::Text(s.into())
    }
}

/// Tool 元数据
#[derive(Debug, Clone)]
pub struct ToolMetadata {
    pub name: String,
    pub description: String,
    pub input_schema: Value,
}

/// Tool 执行上下文
pub struct ToolContext {
    pub session_id: String,
    pub working_dir: Option<String>,
}

impl ToolContext {
    pub fn new(session_id: String) -> Self {
        Self {
            session_id,
            working_dir: None,
        }
    }
}

/// 统一 Tool Trait
#[async_trait]
pub trait Tool: Send + Sync {
    /// Tool 元数据
    fn metadata(&self) -> ToolMetadata;
    
    /// 执行 Tool
    async fn execute(&self, input: &str, ctx: &ToolContext) -> Result<ToolOutput, ToolError>;
    
    /// 可选：验证输入
    fn validate_input(&self, _input: &str) -> Result<(), ToolError> {
        Ok(())
    }
}
