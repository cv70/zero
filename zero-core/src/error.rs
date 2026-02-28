// Main error module
pub mod types;
pub mod handlers;
pub mod context;

pub use types::{ZeroError, AgentError, ToolError, MemoryError, ProviderError, ChannelError};
pub use handlers::{ErrorHandler, ErrorReporter};

/// Error type definitions
pub mod types {
    use thiserror::Error;

    /// Zero Platform errors
    #[derive(Error, Debug)]
    pub enum ZeroError {
        #[error("Agent error: {0}")]
        Agent(#[from] AgentError),
        #[error("Tool error: {0}")]
        Tool(#[from] ToolError),
        #[error("Memory error: {0}")]
        Memory(#[from] MemoryError),
        #[error("Provider error: {0}")]
        Provider(#[from] ProviderError),
        #[error("Channel error: {0}")]
        Channel(#[from] ChannelError),
        #[error("Config error: {0}")]
        Config(#[from] ConfigError),
        #[error("Not found: {0}")]
        NotFound(String),
        #[error("Invalid input: {0}")]
        InvalidInput(String),
    }

    /// Agent errors
    #[derive(Error, Debug)]
    pub enum AgentError {
        #[error("Execution failed: {0}")]
        ExecutionFailed(String),
        #[error("Context error: {0}")]
        ContextError(String),
        #[error("Initialization error: {0}")]
        InitializationError(String),
    }

    /// Tool errors
    #[derive(Error, Debug)]
    pub enum ToolError {
        #[error("Execution failed: {0}")]
        ExecutionFailed(String),
        #[error("Invalid input: {0}")]
        InvalidInput(String),
        #[error("Validation failed: {0}")]
        ValidationFailed(String),
    }

    /// Memory errors
    #[derive(Error, Debug)]
    pub enum MemoryError {
        #[error("Store failed: {0}")]
        StoreFailed(String),
        #[error("Retrieve failed: {0}")]
        RetrieveFailed(String),
        #[error("Search failed: {0}")]
        SearchFailed(String),
        #[error("Delete failed: {0}")]
        DeleteFailed(String),
    }

    /// Provider errors
    #[derive(Error, Debug)]
    pub enum ProviderError {
        #[error("Request failed: {0}")]
        RequestFailed(String),
        #[error("Rate limited: {0}")]
        RateLimited(String),
        #[error("Invalid response: {0}")]
        InvalidResponse(String),
    }

    /// Channel errors
    #[derive(Error, Debug)]
    pub enum ChannelError {
        #[error("Send failed: {0}")]
        SendFailed(String),
        #[error("Receive failed: {0}")]
        ReceiveFailed(String),
        #[error("Connection failed: {0}")]
        ConnectionFailed(String),
    }

    /// Config errors
    #[derive(Error, Debug)]
    pub enum ConfigError {
        #[error("Load failed: {0}")]
        LoadFailed(String),
        #[error("Save failed: {0}")]
        SaveFailed(String),
        #[error("Validate failed: {0}")]
        ValidateFailed(String),
    }
}

/// Error handling utilities
pub mod handlers {
    use super::ZeroError;
    use std::collections::HashMap;

    /// Error handler trait
    pub trait ErrorHandler: Send + Sync {
        /// Handle error
        fn handle_error(&self, error: ZeroError) -> ZeroError;
        
        /// Report error
        fn report_error(&self, error: &ZeroError, context: &HashMap<String, String>);
    }

    /// Error reporter trait
    pub trait ErrorReporter: Send + Sync {
        /// Report error
        fn report(&self, error: &ZeroError);
        
        /// Get error context
        fn get_context(&self) -> HashMap<String, String>;
    }

    /// Default error handler
    pub struct DefaultErrorHandler;

    impl ErrorHandler for DefaultErrorHandler {
        fn handle_error(&self, error: ZeroError) -> ZeroError {
            error
        }
        
        fn report_error(&self, _error: &ZeroError, _context: &HashMap<String, String>) {
            // Default logging
        }
    }

    /// No-op error handler
    pub struct NoopErrorHandler;

    impl ErrorHandler for NoopErrorHandler {
        fn handle_error(&self, error: ZeroError) -> ZeroError {
            error
        }
        
        fn report_error(&self, _error: &ZeroError, _context: &HashMap<String, String>) {
            // No-op
        }
    }

    /// Default error reporter
    pub struct DefaultErrorReporter;

    impl ErrorReporter for DefaultErrorReporter {
        fn report(&self, _error: &ZeroError) {
            // Default logging
        }
        
        fn get_context(&self) -> HashMap<String, String> {
            HashMap::new()
        }
    }
}

/// Error context module
pub mod context {
    use std::collections::HashMap;

    /// Error context
    pub struct ErrorContext {
        pub id: String,
        pub timestamp: i64,
        pub module: String,
        pub operation: String,
        pub additional_data: HashMap<String, String>,
    }

    impl ErrorContext {
        pub fn new(module: &str, operation: &str) -> Self {
            Self {
                id: uuid::Uuid::new_v4().to_string(),
                timestamp: std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap()
                    .as_secs() as i64,
                module: module.to_string(),
                operation: operation.to_string(),
                additional_data: HashMap::new(),
            }
        }

        pub fn with_additional_data(mut self, additional_data: HashMap<String, String>) -> Self {
            self.additional_data = additional_data;
            self
        }
    }

    /// Error context provider
    pub trait ErrorContextProvider: Send + Sync {
        /// Get error context
        fn get_error_context(&self) -> ErrorContext;
    }

    /// Default error context provider
    pub struct DefaultErrorContextProvider;

    impl ErrorContextProvider for DefaultErrorContextProvider {
        fn get_error_context(&self) -> ErrorContext {
            ErrorContext::new("unknown", "unknown");
        }
    }

    /// Thread-local error context provider
    pub struct ThreadLocalErrorContextProvider {
        base_context: ErrorContext,
    }

    impl ThreadLocalErrorContextProvider {
        pub fn new(base_context: ErrorContext) -> Self {
            Self { base_context }
    }}

    impl ErrorContextProvider for ThreadLocalErrorContextProvider {
        fn get_error_context(&self) -> ErrorContext {
            self.base_context.clone()
        }
    }

    /// Error context manager
    pub struct ErrorContextManager {
        context: std::sync::Mutex<Option<ErrorContext>>,
    }

    impl ErrorContextManager {
        pub fn new() -> Self {
            Self {
                context: std::sync::Mutex::new(None),
            }
        }

        pub fn set_context(&self, context: ErrorContext) {
            let mut guard = self.context.lock().unwrap();
            *guard = Some(context);
        }

        pub fn get_context(&self) -> Option<ErrorContext> {
            let guard = self.context.lock().unwrap();
            guard.clone()
        }
    }

    impl Drop for ErrorContextManager {
        fn drop(&mut self) {
            let _ = self.context.lock();
        }
    }
}
