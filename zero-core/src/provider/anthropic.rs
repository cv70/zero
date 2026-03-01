/// Anthropic LLM provider

use crate::provider::LLMProvider;
use crate::error::ProviderError;

/// Anthropic provider
pub struct AnthropicProvider {
    api_key: String,
}

impl AnthropicProvider {
    pub fn new(api_key: String) -> Self {
        Self { api_key }
    }
}

#[async_trait::async_trait]
impl LLMProvider for AnthropicProvider {
    fn name(&self) -> &str {
        "anthropic"
    }

    fn capabilities(&self) -> crate::provider::ModelCapability {
        crate::provider::ModelCapability::Multimodal
    }

    fn available_models(&self) -> Vec<String> {
        vec!["claude-3-opus".to_string()]
    }

    async fn complete(&self, prompt: &str, _opts: crate::provider::CompleteOpts) -> Result<String, ProviderError> {
        // Placeholder
        Ok(format!("Response to: {}", prompt))
    }
}
