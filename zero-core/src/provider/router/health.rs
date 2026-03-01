/// Health check for providers

/// Provider health status
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HealthStatus {
    Healthy,
    Degraded,
    Unhealthy,
}

/// Provider health information
#[derive(Debug, Clone)]
pub struct ProviderHealth {
    pub status: HealthStatus,
    pub response_time_ms: u64,
    pub last_check: std::time::SystemTime,
}

impl ProviderHealth {
    /// Create a new provider health
    pub fn new(status: HealthStatus, response_time_ms: u64) -> Self {
        Self {
            status,
            response_time_ms,
            last_check: std::time::SystemTime::now(),
        }
    }
}
