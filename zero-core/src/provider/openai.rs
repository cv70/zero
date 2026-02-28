use std::sync::Arc;

use crate::provider::{adapter::{OpenAIAdapter, LLMAdapter}, LLMProvider, ProviderError};
use crate::provider::health::{ProviderHealth, ProviderHealthError};
use async_trait::async_trait;
use reqwest::Client;
use crate::provider::adapter::RateLimiter;

// OpenAI specific provider implementing LLMProvider
pub struct OpenAIProvider {
    pub adapter: Arc<OpenAIAdapter>,
}

impl OpenAIProvider {
    pub fn new(api_key: String) -> Self {
        let adapter = OpenAIAdapter {
            client: Client::new(),
            api_key,
            base_url: "https://api.openai.com/v1".to_string(),
            rate_limiter: RateLimiter::new(60, 60),
        };
        OpenAIProvider { adapter: Arc::new(adapter) }
    }
}

#[async_trait]
impl crate::provider::LLMProvider for OpenAIProvider {
    fn id(&self) -> &'static str {
        "openai"
    }

    fn kind(&self) -> crate::provider::ProviderKind {
        crate::provider::ProviderKind::OpenAI
    }

    async fn generate(&self, prompt: &str, model: &str) -> Result<String, ProviderError> {
        let res = self.adapter.generate(prompt, model).await.map_err(ProviderError::Adapter)?;
        Ok(res)
    }

    async fn health(&self) -> Result<ProviderHealth, ProviderHealthError> {
        Ok(ProviderHealth { healthy: true, details: "OPENAI_OK".to_string() })
    }
}
