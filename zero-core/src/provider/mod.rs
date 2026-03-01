pub mod adapter;
pub mod anthropic;
pub mod health;
pub mod hook;
pub mod loop_provider;
pub mod ollama;
pub mod openai;
pub mod router;
pub mod routing;
pub mod r#trait;

pub use anthropic::AnthropicLoopProvider;
pub use loop_provider::{LoopProvider, ProviderResponse, StreamEvent, StreamingLoopProvider};
pub use ollama::OllamaLoopProvider;
pub use openai::OpenAILoopProvider;
pub use r#trait::{CompleteOpts, LLMProvider, MediaInput, ModelCapability};
