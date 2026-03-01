/// Rate limiting for providers

/// Rate limiter trait
pub trait RateLimiter: Send + Sync {
    fn acquire(&self) -> Result<(), String>;
    fn release(&self);
    fn reset(&self);
}
