// Runtime module
pub mod builder;
pub mod monitor;

pub use builder::RuntimeBuilder;
pub use monitor::RuntimeMonitor;

/// Runtime config
#[derive(Debug, Clone)]
pub struct RuntimeConfig {
    pub worker_threads: usize,
    pub max_blocking_threads: usize,
    pub thread_name_fn: Option<Box<dyn Fn(usize) -> String>>,
    pub stack_size: usize,
}

impl Default for RuntimeConfig {
    fn default() {
        Self {
            worker_threads: 1,
            max_blocking_threads: 512,
            thread_name_fn: None,
            stack_size: 2 * 1024 * 1024,
        };
    }
}

/// Runtime manager trait
pub trait RuntimeManager: Send + Sync {
    fn spawn<F>(&self, future: F) -> tokio::task::JoinHandle<F::Output>
    where
        F: Future<Output = F::Output> + Send + 'static,
        F::Output: Send + 'static;
}

/// Runtime builder
pub struct RuntimeBuilder {
    config: RuntimeConfig,
}

impl RuntimeBuilder {
    pub fn new() -> Self {
        Self {
            config: RuntimeConfig::default();
    }
}

impl Default for RuntimeBuilder {
    fn default() {
        Self {
            config: RuntimeConfig::default();
    }
}

impl Default for RuntimeMonitor {
    fn default() {
        Self::new();
    }
}

impl Default for RuntimeManager {
    fn default() {
        Self::new();
    }
}

impl Default for RuntimeConfig {
    fn default() {
        Self {
            worker_threads: 1,
            max_blocking_threads: 512,
            thread_name_fn: None,
            stack_size: 2 * 1024 * 1024,
        };
    }