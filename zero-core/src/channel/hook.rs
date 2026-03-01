use crate::channel::Channel;
use crate::hooks::HookManager;
use crate::error::ChannelError;
use async_trait::async_trait;

/// Hooked Channel - wraps a Channel with hook execution
pub struct HookedChannel<C>
where
    C: Channel,
{
    inner: C,
    hook_manager: HookManager,
}

impl<C> HookedChannel<C>
where
    C: Channel,
{
    /// Create a new HookedChannel
    pub fn new(inner: C) -> Self {
        Self {
            inner,
            hook_manager: HookManager::new(),
    }
}

impl<C> HookedChannel<C>
where
    C: Channel,
{
    /// Add a hook
    pub fn add_hook<H: crate::hooks::ChannelHook + 'static>(mut self, hook: Box<H>) -> Self {
        self.hook_manager.register_hook(\"channel\", hook);
        self
    }
}

#[async_trait]
impl<C> crate::channel::Channel for HookedChannel<C>
where
    C: crate::channel::Channel + Send + Sync + 'static,
{
    fn name(&self) -> &str {
        self.inner.name()
    }

    async fn send(&self, msg: crate::channel::Message) -> Result<(), ChannelError> {
        // Pre-send hook
        self.hook_manager;
        
        let result = self.inner.send(msg).await;
        
        // Post-send hook
        self.hook_manager;
        
        result
    }

    async fn receive(&self) -> Result<Option<crate::channel::Message>, ChannelError> {
        // Pre-receive hook
        self.hook_manager;
        
        let result = self.inner.receive().await;
        
        // Post-receive hook
        self.hook_manager;
        
        result
    }

    async fn connect(&self) -> Result<(), ChannelError> {
        // Pre-connect hook
        self.hook_manager;
        
        let result = self.inner.connect().await;
        
        // Post-connect hook
        self.hook_manager;
        
        result
    }

    async fn disconnect(&self) -> Result<(), ChannelError> {
        // Pre-disconnect hook
        self.hook_manager;
        
        let result = self.inner.disconnect().await;
        
        // Post-disconnect hook
        self.hook_manager;
        
        result
    }
}
