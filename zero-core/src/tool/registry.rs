use std::collections::HashMap;
use std::default::Default;
use std::sync::Arc;
use tokio::sync::RwLock;
use crate::tool::Tool;

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
}

impl Default for ToolRegistry {
    fn default() -> Self {
        Self::new()
    }
}
