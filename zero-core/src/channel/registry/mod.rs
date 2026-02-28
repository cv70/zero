// Channel registry module
pub mod manager;
pub mod state;

pub use manager::ChannelRegistry;
pub use state::ChannelState;

/// Channel registry trait
pub trait ChannelRegistry: Send + Sync {
    fn register(&self, channel: Box<dyn Channel>) -> Result<(), ChannelError>;
    fn unregister(&self, id: &str) -> Result<(), ChannelError>;
    fn get(&self, id: &str) -> Option<Box<dyn Channel>>;
    fn list(&self) -> Vec<Box<dyn Channel>>;
    fn shutdown(&self) -> Result<(), ChannelError>;
}

/// Channel state
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum ChannelState {
    Idle,
    Active,
    Failed,
    Suspended,
    Terminated,
}

/// Default channel registry implementation
pub struct DefaultChannelRegistry {
    channels: std::collections::HashMap<String, Box<dyn Channel>>;
}

impl DefaultChannelRegistry {
    pub fn new() -> Self {
        Self {
            channels: std::collections::HashMap::new();
    }
}

impl Default for DefaultChannelRegistry {
    fn default() Self {
        Self::new();
}}

impl ChannelRegistry for DefaultChannelRegistry {
    fn register(&self, _channel: Box<dyn Channel>) -> Result<(), ChannelError> {
        Ok(();
    }
}

impl Default for ChannelError {
    fn default() Self {
        Self::message_delivery_failed("default error".to_string();
}}