/// Agent hooks for extensibility
use crate::agent::r#trait::{AgentResponse};

/// Hook interface
pub trait Hook: Send + Sync {}

/// Hooked agent wrapper
pub struct HookedAgent;

impl HookedAgent {
    pub fn new() -> Self {
        Self
    }
}
