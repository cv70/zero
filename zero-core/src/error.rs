use thiserror::Error;

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
    Config(String),
    #[error("Not found: {0}")]
    NotFound(String),
}

#[derive(Error, Debug)]
pub enum AgentError {
    #[error("Execution failed: {0}")]
    ExecutionFailed(String),
    #[error("Context error: {0}")]
    ContextError(String),
}

#[derive(Error, Debug)]
pub enum ToolError {
    #[error("Execution failed: {0}")]
    ExecutionFailed(String),
    #[error("Invalid input: {0}")]
    InvalidInput(String),
    #[error("Not supported: {0}")]
    NotSupported(String),
}

#[derive(Error, Debug)]
pub enum MemoryError {
    #[error("Store failed: {0}")]
    StoreFailed(String),
    #[error("Retrieve failed: {0}")]
    RetrieveFailed(String),
    #[error("Search failed: {0}")]
    SearchFailed(String),
}

#[derive(Error, Debug)]
pub enum ProviderError {
    #[error("Request failed: {0}")]
    RequestFailed(String),
    #[error("Rate limited: {0}")]
    RateLimited(String),
    #[error("Invalid response: {0}")]
    InvalidResponse(String),
}

#[derive(Error, Debug)]
pub enum ChannelError {
    #[error("Send failed: {0}")]
    SendFailed(String),
    #[error("Receive failed: {0}")]
    ReceiveFailed(String),
}
