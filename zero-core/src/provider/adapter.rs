use std::sync::Arc;
use std::time::Duration;
use std::time::Instant;

use async_trait::async_trait;
use reqwest::Client;

use crate::provider::health::ProviderHealth;

/// Token bucket rate limiter (per-provider)
#[derive(Debug, Clone)]
pub struct RateLimiter {
    inner: Arc<RateLimiterState>,
}

#[derive(Debug, Default, Clone)]
struct RateLimiterState {
    limit: u32,
    interval: Duration,
    window_start: Instant,
    hits: u32,
}

impl RateLimiter {
    /// Creates a new rate limiter
    pub fn new(limit: u32, interval_secs: u64) -> Self {
        RateLimiter { inner: Arc::new(RateLimiterState {
            limit,
            interval: Duration::from_secs(interval_secs),
            window_start: Instant::now(),
            hits: 0,
        }) }
    }

    /// Acquires permission to make a request, waiting if necessary
    pub async fn acquire(&self) {
        loop {
            let now = Instant::now();
            let mut state = self.inner.as_ref().clone();
            
            // Reset window if expired
            if now.duration_since(state.window_start) > state.interval {
                state.window_start = now;
                state.hits = 0;
            }
            
            // If we've hit the limit, we need to wait
            if state.hits >= state.limit {
                let wait_duration = state.window_start + state.interval - now;
                tokio::time::sleep(wait_duration).await;
                state.window_start = Instant::now();
                state.hits = 0;
            }
            
            // Record this hit
            state.hits += 1;
            
            // Try to update the state
            let mut new_state = self.inner.as_ref().clone();
            if now.duration_since(new_state.window_start) > new_state.interval {
                new_state.window_start = now;
                new_state.hits = 0;
            }
            
            if state.hits >= state.limit {
                let wait_duration = state.window_start + state.interval - now;
                tokio::time::sleep(wait_duration).await;
            }
            
            break;
        }
    }
}

#[derive(Debug, Clone)]
pub struct OpenAIAdapter {
    pub client: Client,
    pub api_key: String,
    pub base_url: String,
    pub rate_limiter: RateLimiter,
}

#[derive(Debug, Clone)]
pub struct AnthropicAdapter {
    pub client: Client,
    pub api_key: String,
    pub base_url: String,
}

#[derive(Debug, Clone)]
pub struct OllamaAdapter {
    pub client: Client,
    pub endpoint: String,
}

#[derive(Debug, thiserror::Error)]
pub enum AdapterError {
    #[error("http error: {0}")]
    Http(#[from] reqwest::Error),
    #[error("invalid response: {0}")]
    BadResponse(String),
}

#[async_trait]
pub trait LLMAdapter: Send + Sync {
    async fn generate(&self, prompt: &str, model: &str) -> Result<String, AdapterError>;
    async fn health(&self) -> bool;
}
