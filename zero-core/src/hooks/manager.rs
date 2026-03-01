use super::{AgentHook, Hook, MemoryHook, ProviderHook};
use std::collections::HashMap;

/// Hook manager
pub struct HookManager {
    hooks: HashMap<String, Vec<Box<dyn Hook>>>,
    provider_hooks: Vec<Box<dyn ProviderHook>>,
    memory_hooks: Vec<Box<dyn MemoryHook>>,
    agent_hooks: Vec<Box<dyn AgentHook>>,
}

impl HookManager {
    /// Create a new hook manager
    pub fn new() -> Self {
        Self {
            hooks: HashMap::new(),
            provider_hooks: Vec::new(),
            memory_hooks: Vec::new(),
            agent_hooks: Vec::new(),
        }
    }

    /// Register a hook
    pub fn register_hook<H: Hook + 'static>(&mut self, hook_type: &str, hook: Box<H>) {
        let hooks = self
            .hooks
            .entry(hook_type.to_string())
            .or_insert_with(Vec::new);
        hooks.push(hook);
    }

    /// Register a provider hook
    pub fn register_provider_hook<H: ProviderHook + 'static>(&mut self, hook: Box<H>) {
        self.provider_hooks.push(hook);
    }

    /// Register an agent hook
    pub fn register_agent_hook<H: AgentHook + 'static>(&mut self, hook: Box<H>) {
        self.agent_hooks.push(hook);
    }

    /// Get hooks by type
    pub fn get_hooks(&self, hook_type: &str) -> Vec<&dyn Hook> {
        self.hooks
            .get(hook_type)
            .map(|h| h.iter().map(|x| x.as_ref()).collect::<Vec<_>>())
            .unwrap_or_default()
    }

    /// Run agent hooks
    pub async fn run_agent_hooks(&self) -> Result<(), String> {
        for _hook in &self.agent_hooks {
            // hooks would be called here with .await if needed
        }
        Ok(())
    }
}

impl Default for HookManager {
    fn default() -> Self {
        Self::new()
    }
}
