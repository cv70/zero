use std::sync::Arc;

use crate::provider::{adapter::{AnthropicAdapter, LLMAdapter}, LLMProvider, ProviderError, ProviderKind};
use crate::provider::health::{ProviderHealth, ProviderHealthError};
use crate::provider::LLMProvider;
use async_trait::async_trait;
use reqwest::Client;
use crate::provider::adapter::RateLimiter;

pub struct AnthropicProvider {
    pub adapter: Arc<AnthropicAdapter>,
}

impl AnthropicProvider {
    pub fn new(api_key: String) -> Self {
        let adapter = AnthropicAdapter {
            client: Client::new(),
            api_key,
            base_url: "https://api.anthropic.com/v1".to_string(),
        };
        AnthropicProvider { adapter: Arc::new(adapter) }
    }
}

#[async_trait]
impl crate::provider::LLMProvider for AnthropicProvider {
    fn id(&self) -> &'static str { "anthropic" }
    fn kind(&self) -> ProviderKind { ProviderKind::Anthropic }
    async fn generate(&self, prompt: &str, model: &str) -> Result<String, ProviderError> {
        let res = self.adapter.generate(prompt, model).await.map_err(ProviderError::Adapter)?;
        Ok(res)
    }
    async fn health(&self) -> Result<ProviderHealth, ProviderHealthError> {
        Ok(ProviderHealth { healthy: true, details: "ANTHROPIC_OK".to_string() })
    }
}
