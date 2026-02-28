use std::collections::HashMap;
use std::sync::Arc;

use async_trait::async_trait;
use serde::{Deserialize, Serialize};

use std::collections::HashMap as _HashMap;

// Core shared types for the Channel subsystem. Exposed for sibling modules.

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum ChannelKind {
    Telegram,
    Discord,
    Slack,
    Email,
    Matrix,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChannelMessage {
    pub id: String,
    pub channel: ChannelKind,
    pub content: String,
    pub timestamp: u64,
    pub metadata: std::collections::HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebhookPayload {
    pub channel: ChannelKind,
    pub content: String,
    pub headers: std::collections::HashMap<String, String>,
}

pub type ChannelResult<T> = Result<T, Box<dyn std::error::Error + Send + Sync>>;
pub type ChannelError = Box<dyn std::error::Error + Send + Sync>;

#[async_trait]
pub trait Channel: Send + Sync {
    async fn init(&self) -> ChannelResult<()>;
    fn kind(&self) -> ChannelKind;
    async fn send(&self, msg: ChannelMessage) -> ChannelResult<()>;
    async fn handle_webhook(&self, payload: WebhookPayload) -> ChannelResult<()>;
    // Default routing: simply forward to send
    async fn route_message(&self, msg: ChannelMessage) -> ChannelResult<()> {
        self.send(msg).await
    }
}

pub mod telegram;
pub mod discord;
pub mod slack;
pub mod email;
pub mod matrix;
pub mod webhook;
pub mod queue;
pub mod persistence;
pub mod analytics;

pub use self::queue::MessageQueue;
pub use self::persistence::Persistence;
pub use self::webhook::WebhookPayload;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum ChannelKind {
    Telegram,
    Discord,
    Slack,
    Email,
    Matrix,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChannelMessage {
    pub id: String,
    pub channel: ChannelKind,
    pub content: String,
    pub timestamp: u64,
    pub metadata: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebhookPayload {
    pub channel: ChannelKind,
    pub content: String,
    pub headers: HashMap<String, String>,
}

pub type ChannelResult<T> = Result<T, Box<dyn std::error::Error + Send + Sync>>;
pub type ChannelError = Box<dyn std::error::Error + Send + Sync>;

#[async_trait]
pub trait Channel: Send + Sync {
    async fn init(&self) -> ChannelResult<()>;
    fn kind(&self) -> ChannelKind;
    async fn send(&self, msg: ChannelMessage) -> ChannelResult<()>;
    async fn handle_webhook(&self, payload: WebhookPayload) -> ChannelResult<()>;
    // Default routing: simply forward to send
    async fn route_message(&self, msg: ChannelMessage) -> ChannelResult<()> {
        self.send(msg).await
    }
}
