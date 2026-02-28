use thiserror::Error;

#[derive(Debug, Clone, Copy)]
pub struct ProviderHealth {
    pub healthy: bool,
    pub details: String,
}

#[derive(Debug, Error)]
pub enum ProviderHealthError {
    #[error("health check failed: {0}")]
    Error(String),
}
