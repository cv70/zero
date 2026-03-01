// Config module
pub mod hooks;
pub mod loader;
pub mod validator;

pub use hooks::ConfigHooks;
pub use loader::ConfigLoader;
pub use validator::ConfigValidator;

/// Configuration error
#[derive(Debug, thiserror::Error)]
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
    FormatError(String),
}

/// Result type for config operations
pub type ConfigResult<T> = Result<T, ConfigError>;
