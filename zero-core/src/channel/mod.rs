// Channel module for message channels
pub mod queue;
pub mod registry;
pub mod r#trait;
// pub mod persistence;

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Channel message type
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChannelMessage {
    pub id: String,
    pub channel_id: String,
    pub from: String,
    pub to: String,
    pub content: String,
    pub timestamp: i64,
    pub metadata: HashMap<String, String>,
}

// Re-export types from trait module
pub use r#trait::{Channel, ChannelError, Message};
