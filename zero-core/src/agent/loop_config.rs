/// Configuration for Agent Loop execution
///
/// This structure controls the behavior of the Agent loop, including
/// timeouts, iteration limits, and feature flags.
use std::time::Duration;

/// Configuration for the Agent loop execution
///
/// # Example
///
/// ```ignore
/// let config = AgentLoopConfig {
///     max_iterations: 50,
///     provider_timeout: 300,
///     tool_timeout: 120,
///     enable_hooks: true,
///     save_history: true,
/// };
/// ```
#[derive(Debug, Clone)]
pub struct AgentLoopConfig {
    /// Maximum number of iterations before stopping
    /// Default: 100
    pub max_iterations: usize,

    /// Timeout for a single LLM provider call (in seconds)
    /// Default: 300 (5 minutes)
    pub provider_timeout: u64,

    /// Timeout for a single tool execution (in seconds)
    /// Default: 120 (2 minutes)
    pub tool_timeout: u64,

    /// Whether to enable hook system
    /// Default: true
    pub enable_hooks: bool,

    /// Whether to save message history to storage
    /// Default: true
    pub save_history: bool,

    /// Maximum number of concurrent tool executions
    /// Default: 4
    pub max_concurrent_tools: usize,

    /// Whether to log verbose debug information
    /// Default: false
    pub verbose_logging: bool,

    /// Maximum context tokens before compaction (0 = disabled)
    /// Default: 0 (disabled)
    pub max_context_tokens: usize,
}

impl AgentLoopConfig {
    /// Create a new AgentLoopConfig with default values
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the maximum number of iterations
    pub fn with_max_iterations(mut self, max_iterations: usize) -> Self {
        self.max_iterations = max_iterations;
        self
    }

    /// Set the provider timeout (in seconds)
    pub fn with_provider_timeout(mut self, timeout: u64) -> Self {
        self.provider_timeout = timeout;
        self
    }

    /// Set the tool timeout (in seconds)
    pub fn with_tool_timeout(mut self, timeout: u64) -> Self {
        self.tool_timeout = timeout;
        self
    }

    /// Enable or disable hooks
    pub fn with_hooks(mut self, enable_hooks: bool) -> Self {
        self.enable_hooks = enable_hooks;
        self
    }

    /// Enable or disable history saving
    pub fn with_history(mut self, save_history: bool) -> Self {
        self.save_history = save_history;
        self
    }

    /// Set the maximum concurrent tool executions
    pub fn with_max_concurrent_tools(mut self, max: usize) -> Self {
        self.max_concurrent_tools = max;
        self
    }

    /// Enable or disable verbose logging
    pub fn with_verbose_logging(mut self, verbose: bool) -> Self {
        self.verbose_logging = verbose;
        self
    }

    /// Set the maximum context tokens before compaction triggers (0 = disabled)
    pub fn with_max_context_tokens(mut self, max: usize) -> Self {
        self.max_context_tokens = max;
        self
    }

    /// Convert provider timeout to Duration
    pub fn provider_timeout_duration(&self) -> Duration {
        Duration::from_secs(self.provider_timeout)
    }

    /// Convert tool timeout to Duration
    pub fn tool_timeout_duration(&self) -> Duration {
        Duration::from_secs(self.tool_timeout)
    }
}

impl Default for AgentLoopConfig {
    fn default() -> Self {
        Self {
            max_iterations: 100,
            provider_timeout: 300,
            tool_timeout: 120,
            enable_hooks: true,
            save_history: true,
            max_concurrent_tools: 4,
            verbose_logging: false,
            max_context_tokens: 0,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = AgentLoopConfig::default();
        assert_eq!(config.max_iterations, 100);
        assert_eq!(config.provider_timeout, 300);
        assert_eq!(config.tool_timeout, 120);
        assert!(config.enable_hooks);
        assert!(config.save_history);
    }

    #[test]
    fn test_builder_pattern() {
        let config = AgentLoopConfig::new()
            .with_max_iterations(50)
            .with_provider_timeout(120)
            .with_tool_timeout(60)
            .with_hooks(false)
            .with_history(false);

        assert_eq!(config.max_iterations, 50);
        assert_eq!(config.provider_timeout, 120);
        assert_eq!(config.tool_timeout, 60);
        assert!(!config.enable_hooks);
        assert!(!config.save_history);
    }

    #[test]
    fn test_timeout_to_duration() {
        let config = AgentLoopConfig::default();
        let provider_duration = config.provider_timeout_duration();
        let tool_duration = config.tool_timeout_duration();

        assert_eq!(provider_duration.as_secs(), 300);
        assert_eq!(tool_duration.as_secs(), 120);
    }

    #[test]
    fn test_max_concurrent_tools() {
        let config = AgentLoopConfig::new().with_max_concurrent_tools(8);
        assert_eq!(config.max_concurrent_tools, 8);
    }

    #[test]
    fn test_verbose_logging() {
        let config = AgentLoopConfig::new().with_verbose_logging(true);
        assert!(config.verbose_logging);
    }
}
