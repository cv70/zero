/// Channel module for message channels
// pub mod registry;
// pub mod trait;
// pub mod queue;
// pub mod persistence;
// pub use crate::channel::registry::{ChannelRegistry, ChannelState, DefaultChannelRegistry};
pub use async_trait::async_trait;
use thiserror::Error;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// 消息结构