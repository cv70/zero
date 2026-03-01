use crate::memory::Memory;
use crate::hooks::HookManager;
use crate::error::MemoryError;
use async_trait::async_trait;

/// Hooked Memory - wraps a Memory system with hook execution
pub struct HookedMemory<M>
where
    M: Memory,
{
    inner: M,
    hook_manager: HookManager,
}

impl<M> HookedMemory<M>
where
    M: Memory,
{
    /// Create a new HookedMemory
    pub fn new(inner: M) -> Self {
        Self {
            inner,
            hook_manager: HookManager::new(),
    }
}

impl<M> HookedMemory<M>
where
    M: Memory,
{
    /// Add a hook
    pub fn add_hook<H: crate::hooks::MemoryHook + 'static>(mut self, hook: Box<H>) -> Self {
        self.hook_manager.register_hook(\"memory\", hook);
        self
    }
}

#[async_trait]
impl<M> crate::memory::Memory for HookedMemory<M>
where
    M: crate::memory::Memory + Send + Sync + 'static,
{
    async fn get(&self, key: &str) -> Result<Option<String>, MemoryError> {
        // Pre-get hook
        self.hook_manager;
        
        let result = self.inner.get(key).await;
        
        // Post-get hook
        self.hook_manager;
        
        result
    }

    async fn set(&self, key: &str, value: &str) -> Result<(), MemoryError> {
        // Pre-set hook
        self.hook_manager;
        
        let result = self.inner.set(key, value).await;
        
        // Post-set hook
        self.hook_manager;
        
        result
    }

    async fn delete(&self, key: &str) -> Result<bool, MemoryError> {
        // Pre-delete hook
        self.hook_manager;
        
        let result = self.inner.delete(key).await;
        
        // Post-delete hook
        self.hook_manager;
        
        result
    }

    async fn exists(&self, key: &str) -> Result<bool, MemoryError> {
        // Pre-exists hook
        self.hook_manager;
        
        let result = self.inner.exists(key).await;
        
        // Post-exists hook
        self.hook_manager;
        
        result
    }

    async fn keys(&self, key: &str) -> Result<Vec<String>, MemoryError> {
        // Pre-keys hook
        self.hook_manager;
        
        let result = self.inner.keys(key).await;
        
        // Post-keys hook
        self.hook_manager;
        
        result
    }
}
