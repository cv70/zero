use std::collections::HashMap;
use std::default::Default;
use std::sync::Arc;
use tokio::sync::RwLock;
use crate::error::ToolError;
use crate::tool::{Tool, ToolOutput, ToolContext};

pub struct ToolRegistry {
    tools: Arc<RwLock<HashMap<String, Box<dyn Tool>>>>,
}

impl ToolRegistry {
    pub fn new() -> Self {
        Self {
            tools: Arc::new(RwLock::new(HashMap::new()))
        }
    }

    pub async fn register(&self, tool: Box<dyn Tool>) {
        let name = tool.metadata().name.clone();
        self.tools.write().await.insert(name, tool);
    }

    pub async fn list(&self) -> Vec<String> {
        self.tools.read().await.keys().cloned().collect()
    }

    /// Look up a tool by name and execute it with the given input and context.
    ///
    /// Returns `ToolError::NotFound` if no tool with the given name is registered.
    pub async fn execute_tool(
        &self,
        name: &str,
        input: &str,
        ctx: &ToolContext,
    ) -> Result<ToolOutput, ToolError> {
        let tools = self.tools.read().await;
        let tool = tools
            .get(name)
            .ok_or_else(|| ToolError::NotFound(name.to_string()))?;
        tool.execute(input, ctx).await
    }
}

impl Default for ToolRegistry {
    fn default() -> Self {
        Self::new()
    }
}
