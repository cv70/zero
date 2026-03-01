/// Provider hooks for extensibility
///
/// This module provides hooks for intercepting provider operations
use crate::hooks::{Hook, HookManager};
use std::sync::Arc;

/// Hook that runs before a provider call
pub trait BeforeProviderCallHook: Hook + Send + Sync {
    /// Called before the provider is called
    fn before_call(&self, provider_name: &str) -> Result<(), crate::error::ProviderError>;
}

/// Hook that runs after a provider call
pub trait AfterProviderCallHook: Hook + Send + Sync {
    /// Called after the provider completes
    fn after_call(
        &self,
        provider_name: &str,
        success: bool,
    ) -> Result<(), crate::error::ProviderError>;
}

/// Provider with hook support
pub struct HookedProviderWrapper {
    hook_manager: Arc<HookManager>,
}

impl HookedProviderWrapper {
    /// Create a new hooked provider wrapper
    pub fn new(hook_manager: Arc<HookManager>) -> Self {
        Self { hook_manager }
    }

    /// Get the hook manager
    pub fn hook_manager(&self) -> &Arc<HookManager> {
        &self.hook_manager
    }
}
