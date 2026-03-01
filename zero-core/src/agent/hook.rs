/// Agent hooks for extensibility

/// Hook interface
pub trait Hook: Send + Sync {
    /// Hook name
    fn name(&self) -> &str;
}

impl Hook for () {
    fn name(&self) -> &str {
        "none"
    }
}

/// Hooked agent wrapper
pub struct HookedAgent;

impl HookedAgent {
    pub fn new() -> Self {
        Self
    }
}
