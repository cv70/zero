pub mod r#trait;
pub mod loop_provider;
pub mod anthropic;
pub mod openai;
pub mod ollama;
pub mod adapter;
pub mod routing;
pub mod hook;
pub mod health;
pub mod router;

pub use r#trait::{LLMProvider, ModelCapability, MediaInput, CompleteOpts};
pub use loop_provider::{LoopProvider, ProviderResponse};
pub use anthropic::AnthropicLoopProvider;
pub use openai::OpenAILoopProvider;
pub use ollama::OllamaLoopProvider;
