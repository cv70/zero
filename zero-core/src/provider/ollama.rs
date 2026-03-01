/// Ollama LLM provider

use crate::provider::LLMProvider;
use crate::error::ProviderError;

/// Ollama provider
pub struct OllamaProvider {
    endpoint: String,
}

impl OllamaProvider {
    pub fn new(endpoint: String) -> Self {
        Self { endpoint }
    }
}

#[async_trait::async_trait]
impl LLMProvider for OllamaProvider {
    fn name(&self) -> &str {
        "ollama"
    }

    fn capabilities(&self) -> crate::provider::ModelCapability {
        crate::provider::ModelCapability::TextOnly
    }

    fn available_models(&self) -> Vec<String> {
        vec!["llama2".to_string()]
    }

    async fn complete(&self, prompt: &str, _opts: crate::provider::CompleteOpts) -> Result<String, ProviderError> {
        // Placeholder
        Ok(format!("Response to: {}", prompt))
    }
}
