use std::sync::Arc;

use crate::provider::{adapter::{OllamaAdapter, LLMAdapter}, LLMProvider, ProviderError, ProviderKind};
use crate::provider::health::{ProviderHealth, ProviderHealthError};
use crate::provider::LLMProvider;
use async_trait::async_trait;
use reqwest::Client;
use crate::provider::adapter::RateLimiter;

pub struct OllamaProvider {
    pub adapter: Arc<OllamaAdapter>,
}

impl OllamaProvider {
    pub fn new(endpoint: String) -> Self {
        let adapter = OllamaAdapter {
            client: Client::new(),
            endpoint,
        };
        OllamaProvider { adapter: Arc::new(adapter) }
    }
}

#[async_trait]
impl crate::provider::LLMProvider for OllamaProvider {
    fn id(&self) -> &'static str { "ollama" }
    fn kind(&self) -> ProviderKind { ProviderKind::Ollama }
    async fn generate(&self, prompt: &str, model: &str) -> Result<String, ProviderError> {
        let res = self.adapter.generate(prompt, model).await.map_err(ProviderError::Adapter)?;
        Ok(res)
    }
    async fn health(&self) -> Result<ProviderHealth, ProviderHealthError> {
        Ok(ProviderHealth { healthy: true, details: "OLLAMA_OK".to_string() })
    }
}
