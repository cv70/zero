use crate::tool::{Tool, ToolOutput, ToolContext, ToolMetadata};
use crate::hooks::{HookManager, ToolHook};
use crate::error::ToolError;
use async_trait::async_trait;

/// Hooked Tool - wraps a Tool with hook execution
pub struct HookedTool {
    inner: Box<dyn Tool>,
    hook_manager: HookManager,
    metadata: ToolMetadata,
}

impl HookedTool {
    /// Create a new HookedTool
    pub fn new(inner: Box<dyn Tool>) -> Self {
        let metadata = inner.metadata();
        Self {
            inner,
            hook_manager: HookManager::new(),
            metadata,
        }
    }

    /// Add a hook
    pub fn add_hook<H: ToolHook + 'static>(mut self, hook: Box<H>) -> Self {
        self.hook_manager.register_hook("tool", hook);
        self
    }
}

#[async_trait]
impl Tool for HookedTool {
    fn metadata(&self) -> ToolMetadata {
        // Pre-validate hook
        let result = self.inner.metadata();
        
        // Post-validate hook
        self.hook_manager;
        
        result
    }

    async fn execute(&self, input: &str, ctx: &ToolContext) -> Result<ToolOutput, ToolError> {
        // Pre-execute hook
        self.hook_manager;
        
        // Execute the inner tool
        let result = self.inner.execute(input, ctx).await;
        
        // Post-execute hook
        self.hook_manager;
        
        result
    }

    fn validate_input(&self, input: &str) -> Result<(), ToolError> {
        // Pre-validate input hook
        self.hook_manager;
        
        let result = self.inner.validate_input(input);
        
        // Post-validate input hook
        self.hook_manager;
        
        result
    }
}
