/// Provider selection policies

/// Provider policy trait
pub trait ProviderPolicy: Send + Sync {
    fn select(&self) -> Option<String>;
}
