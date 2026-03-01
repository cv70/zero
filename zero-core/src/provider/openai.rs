/// OpenAI LLM provider

use crate::provider::LLMProvider;
use crate::error::ProviderError;

/// OpenAI provider
pub struct OpenAIProvider {
    api_key: String,
}

impl OpenAIProvider {
    pub fn new(api_key: String) -> Self {
        Self { api_key }
    }
}

#[async_trait::async_trait]
impl LLMProvider for OpenAIProvider {
    fn name(&self) -> &str {
        "openai"
    }

    fn capabilities(&self) -> crate::provider::ModelCapability {
        crate::provider::ModelCapability::Multimodal
    }

    fn available_models(&self) -> Vec<String> {
        vec!["gpt-4".to_string()]
    }

    async fn complete(&self, prompt: &str, _opts: crate::provider::CompleteOpts) -> Result<String, ProviderError> {
        // Placeholder
        Ok(format!("Response to: {}", prompt))
    }
}
