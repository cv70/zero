/// Channel module for message channels
pub mod registry;

pub use crate::channel::registry::{ChannelRegistry, ChannelState, DefaultChannelRegistry};
pub use async_trait::async_trait;
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
    pub attachments: Vec<MediaInput>,
}

impl Message {
    pub fn new(from: impl Into<String>, content: impl Into<String>) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
        }
    }
}

/// Channel Trait
#[async_trait]
pub trait Channel: Send + Sync {
    /// Channel 名称
    fn name(&self) -> &str;

    /// 发送消息
    async fn send(&self, msg: &Message) -> Result<(), ChannelError>;

    /// 接收消息（可选）
    async fn receive(&self) -> Result<Option<Message>, ChannelError>;

    /// 连接
    async fn connect(&self) -> Result<(), ChannelError>;

    /// 断开
    async fn disconnect(&self) -> Result<(), ChannelError>;
}
