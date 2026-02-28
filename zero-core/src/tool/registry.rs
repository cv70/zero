use crate::error::ToolError;
use std::collections::HashMap;
use std::default::Default;
use std::sync::Arc;
use tokio::sync::RwLock;
use crate::tool::{Tool, ToolContext};

pub struct ToolRegistry {
    tools: Arc<RwLock<HashMap<String, Box<dyn Tool>>>>,
}

impl ToolRegistry {
    pub fn new() -> Self {
        Self {
            tools: Arc::new(RwLock::new(HashMap::new()))
        }
    }
}