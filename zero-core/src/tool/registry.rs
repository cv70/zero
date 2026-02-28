use crate::error::ToolError;
use crate::tool::{Tool, ToolOutput, ToolContext};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

pub struct ToolRegistry {
    tools: Arc<RwLock<HashMap<String, Box<dyn Tool>>>>,
}

impl ToolRegistry {
    pub fn new() -> Self {
        Self {
            tools: Arc::new(RwLock::new(HashMap::new())),
        }
    }
    
    pub async fn register(&self, tool: Box<dyn Tool>) {
        let name = tool.metadata().name.clone();
        self.tools.write().await.insert(name, tool);
    }
    
    pub async fn execute(&self, name: &str, input: &str, ctx: &ToolContext) -> Result<ToolOutput, ToolError> {
        let guard = self.tools.read().await;
        let tool = guard.get(name).ok_or_else(|| {
            ToolError::ExecutionFailed(format!("Tool not found: {}", name))
        })?;
        tool.execute(input, ctx).await
    }
    
    pub async fn list(&self) -> Vec<String> {
        self.tools.read().await.keys().cloned().collect()
    }
}
