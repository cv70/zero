use crate::error::AgentError;
use async_trait::async_trait;
use std::collections::HashMap;

// Forward declaration - actual definition in context.rs
pub struct AgentContext;

/// Agent 执行结果
#[derive(Debug, Clone)]
pub struct AgentResponse {
    pub content: String,
    pub tool_calls: Vec<ToolCall>,
    pub metadata: HashMap<String, String>,
}

/// Tool 调用请求
#[derive(Debug, Clone)]
pub struct ToolCall {
    pub id: String,
    pub name: String,
    pub arguments: String,
}

/// Agent  Trait - 定义 Agent 的核心接口
#[async_trait]
pub trait Agent: Send + Sync {
    /// Agent 名称
    fn name(&self) -> &str;
    
    /// Agent 系统提示词
    fn system_prompt(&self) -> &str;
    
    /// Agent 描述
    fn description(&self) -> &str;
    
    /// 执行 Agent
    async fn run(&mut self, ctx: &mut AgentContext) -> Result<AgentResponse, AgentError>;
}
