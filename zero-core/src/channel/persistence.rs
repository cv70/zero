use std::path::Path;
use std::sync::Arc;
use tokio::fs;
use tokio::fs::OpenOptions;
use serde::{Serialize, Deserialize};

use crate::channel::{ChannelMessage};

#[derive(Clone)]
pub struct Persistence {
    path: String,
}

impl Persistence {
    pub fn new(path: impl Into<String>) -> Self {
        Self { path: path.into() }
    }

    pub async fn save_message(&self, msg: &ChannelMessage) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let mut all = self.load_messages().await.unwrap_or_else(|_| Vec::new());
        all.push(msg.clone());
        let json = serde_json::to_string(&all)?;
        fs::write(&self.path, json).await?;
        Ok(())
    }

    pub async fn load_messages(&self) -> Result<Vec<ChannelMessage>, Box<dyn std::error::Error + Send + Sync>> {
        if !Path::new(&self.path).exists() {
            return Ok(Vec::new());
        }
        let data = fs::read_to_string(&self.path).await?;
        let v: Vec<ChannelMessage> = serde_json::from_str(&data)?;
        Ok(v)
    }
}
