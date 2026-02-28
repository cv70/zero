// Config hooks module
use super::ConfigResult;
use serde_json::Value;

/// Config hook trait
pub trait ConfigHook: Send + Sync {
    /// Called before configuration load
    fn before_load(&self) -> ConfigResult<()>;

    /// Called after configuration load
    fn after_load(&self, value: &Value) -> ConfigResult<()>;

    /// Called before configuration save
    fn before_save(&self) -> ConfigResult<()>;

    /// Called after configuration save
    fn after_save(&self, value: &Value) -> ConfigResult<()>;
}

/// Configuration hooks manager
#[allow(dead_code)]
pub struct ConfigHooks {
    hooks: Vec<Box<dyn ConfigHook>>,
}

impl ConfigHooks {
    pub fn new() -> Self {
        Self { hooks: Vec::new() }
    }

    pub fn add_hook(&mut self, hook: Box<dyn ConfigHook>) {
        self.hooks.push(hook);
    }

    pub fn run_before_load(&mut self) -> ConfigResult<()> {
        for hook in &self.hooks {
            hook.before_load()?;
        }
        Ok(())
    }

    pub fn run_after_load(&mut self, value: &Value) -> ConfigResult<()> {
        for hook in &self.hooks {
            hook.after_load(value)?;
        }
        Ok(())
    }

    pub fn run_before_save(&mut self) -> ConfigResult<()> {
        for hook in &self.hooks {
            hook.before_save()?;
        }
        Ok(())
    }

    pub fn run_after_save(&mut self, value: &Value) -> ConfigResult<()> {
        for hook in &self.hooks {
            hook.after_save(value)?;
        }
        Ok(())
    }
}
