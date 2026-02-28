// Config module
pub mod loader;
pub mod validator;
pub mod hooks;

pub use loader::ConfigLoader;
pub use validator::ConfigValidator;
pub use hooks::ConfigHooks;

/// Configuration trait for modules
pub trait Configurable: Send + Sync {
    fn config(&self) -> &Self::Config;
    fn config_mut(&mut self) -> &mut Self::Config;
}

/// Configurable trait with type configuration
pub trait ConfigurableWith<T: Configurable> {
    fn with_config(config: T) -> Self;
}

/// Configuration error
#[derive(Debug, Clone, thiserror::Error, PartialEq, Eq)]
pub enum ConfigError {
    #[error("Config not found: {0}")]
    NotFound(String),
    #[error("Config invalid: {0}")]
    Invalid(String),
    #[error("Config load failed: {0}")]
    LoadFailed(String),
    #[error("Config save failed: {0}")]
    SaveFailed(String),
    #[error("Config format error: {0}")]
    FormatError(#[from] serde_json::Error),
}

/// Result type for config operations
pub type ConfigResult<T> = Result<T, ConfigError>;
