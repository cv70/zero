use async_trait::async_trait;

// Re-export submodules for easier usage from the crate root
pub mod adapter;
pub mod openai;
pub mod anthropic;
pub mod ollama;
pub mod health;
pub mod routing;

use crate::provider::health::ProviderHealth;

// Publicly exposed types
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProviderKind {
    OpenAI,
    Anthropic,
    Ollama,
    Unknown,
}

#[derive(Debug, thiserror::Error)]
pub enum ProviderError {
    #[error("adapter error: {0}")]
    Adapter(String),
    #[error("not available: {0}")]
    NotAvailable(String),
    #[error("bad response: {0}")]
    BadResponse(String),
}

impl From<String> for ProviderError {
    fn from(s: String) -> Self {
        ProviderError::Adapter(s)
    }
}

// Trait that all LLM providers must implement
#[async_trait]
pub trait LLMProvider: Send + Sync {
    // Identifier used in logs/metrics
    fn id(&self) -> &'static str;
    // Kind for routing/selection
    fn kind(&self) -> crate::provider::ProviderKind;

    // Generate a response given a prompt and model name
    async fn generate(&self, prompt: &str, model: &str) -> Result<String, ProviderError>;

    // Health check for the provider (internal readiness)
    async fn health(&self) -> Result<ProviderHealth, ProviderHealthError>;
}

// Health error type wrapper
#[derive(Debug, thiserror::Error)]
pub enum ProviderHealthError {
    #[error("health check failed: {0}")]
    Error(String),
}
