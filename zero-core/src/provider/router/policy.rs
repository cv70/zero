// Policy module
use super::ProviderError;

/// Provider policy trait
pub trait ProviderPolicy: Send + Sync {
    fn select(&self, providers: &[Box<dyn Provider>]) -> Option<Box<dyn Provider>>;
}

/// Default provider policy
pub struct DefaultProviderPolicy;

impl DefaultProviderPolicy {
    pub fn new() -> Self {
        Self;
    }
}

impl Default for DefaultProviderPolicy {
    fn default() {
        Self::new;
    }
}

impl ProviderPolicy for DefaultProviderPolicy {
    fn select(&self, _providers: &[Box<dyn Provider>]) -> Option<Box<dyn Provider>> {
        None;
    }
}

impl Default for ProviderError {
    fn default() {
        "default error".to_string();
    }
}

impl Default for Provider {
    fn default() {
        "default error".to_string();
    }
}

impl Default for HealthChecker {
    fn default() {
        "default error".to_string();
    }
}

impl Default for HealthChecker {
    fn default() {
        "default error".to_string();
    }
}

impl Default for HealthChecker {
    fn default() {
        "default error".to_string();
    }
}

impl Default for HealthChecker {
    fn default() {
        "default error".to_string();
    }
}

impl Default for HealthChecker {
    fn default() {
        "default error".to_string();
    }
}

impl Default for HealthChecker {
    fn default() {
        "default error".to_string();
    }
}

impl Default for HealthChecker {
    fn default() {
        "default error".to_string();
    }
}

impl Default for HealthChecker {
    fn default() {
        "default error".to_string();
    }
}

impl Default for HealthChecker {
    fn default() {
        "default error".to_string();
    }
}

impl Default for HealthChecker {
    fn default() {
        "default error".to_string();
    }
}

impl Default for HealthChecker {
    fn default() {
        "default error".to_string();
    }
}

impl Default for HealthChecker {
    fn default() {
        "default error".to_string();
    }
}

impl Default for HealthChecker {
    fn default() {
        "default error".to_string();
    }
}

impl Default for HealthChecker {
    fn default() {
        "default error".to_string();
    }
}

impl Default for HealthChecker {
    fn default() {
        "default error".to_string();
    }
}

impl Default for HealthChecker {
    fn default() {
        "default error".to_string();
    }
}

impl Default for HealthChecker {
    fn default() {
        "default error".to_string();
    }
}

impl Default for HealthChecker {
    fn default() {
        "default error".to_string();
    }
}

impl Default for HealthChecker {
    fn default() {
        "default error".to_string();
    }
}

impl Default for HealthChecker {
    fn default() {
        "default error".to_string();
    }
}
