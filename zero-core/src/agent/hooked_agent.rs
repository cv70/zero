use crate::agent::{Agent, AgentContext, AgentResponse};
use crate::error::AgentError;
use crate::hooks::HookManager;
use async_trait::async_trait;
use std::collections::HashMap;

/// Hooked Agent - wraps an Agent with hook execution
pub struct HookedAgent {
    inner: Box<dyn Agent>,
    hook_manager: HookManager,
}

impl HookedAgent {
    /// Create a new HookedAgent
    pub fn new(inner: Box<dyn Agent>) -> Self {
        Self {
            inner,
            hook_manager: HookManager::new(),
        }
    }

    /// Get a reference to the inner Agent
    pub fn inner(&self) -> &dyn Agent {
        self.inner.as_ref()
    }

    /// Execute the agent with hooks
    pub async fn run_with_hooks(&mut self, ctx: &mut AgentContext) -> Result<AgentResponse, AgentError> {
        // Run pre-run hooks
        self.hook_manager.run_agent_hooks().await?;
        
        // Run the inner agent
        let result = self.inner.run(ctx).await?;
        
        // Run post-run hooks
        self.hook_manager.run_agent_hooks().await?;
        
        Ok(result)
    }
}

#[async_trait]
impl Agent for HookedAgent {
    fn name(&self) -> &str {
        self.inner.name()
    }

    fn system_prompt(&self) -> &str {
        self.inner.system_prompt()
    }

    fn description(&self) -> &str {
        self.inner.description()
    }

    async fn run(&mut self, ctx: &mut AgentContext) -> Result<AgentResponse, AgentError> {
        self.run_with_hooks(ctx).await
    }
}
