// Channel module for message channels
// pub mod registry;
// pub mod r#trait;
// pub mod queue;
// pub mod persistence;

use async_trait::async_trait;
use thiserror::Error;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// 消息结构
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    pub id: String,
    pub from: String,
    pub to: String,
    pub content: String,
    pub timestamp: i64,
    pub metadata: HashMap<String, String>,
}

impl Message {
    pub fn new(from: impl Into<String>, to: impl Into<String>, content: impl Into<String>) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            from: from.into(),
            to: to.into(),
            content: content.into(),
            timestamp: 0,
            metadata: HashMap::new(),
        }
    }
}

/// Channel Trait
#[async_trait]
pub trait Channel: Send + Sync {
    fn name(&self) -> &str;
    async fn send(&self, msg: &Message) -> Result<(), ChannelError>;
    async fn receive(&self) -> Result<Option<Message>, ChannelError>;
    async fn connect(&self) -> Result<(), ChannelError>;
    async fn disconnect(&self) -> Result<(), ChannelError>;
}

/// Channel error
#[derive(Error, Debug)]
pub enum ChannelError {
    #[error("Channel not found: {0}")]
    ChannelNotFound(String),
    #[error("Channel already exists: {0}")]
    ChannelAlreadyExists(String),
    #[error("Send failed: {0}")]
    SendFailed(String),
    #[error("Receive failed: {0}")]
    ReceiveFailed(String),
}

impl Default for ChannelError {
    fn default() -> Self {
        ChannelError::SendFailed("default error".to_string())
    }
}