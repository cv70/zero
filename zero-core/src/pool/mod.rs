// Pool module
pub mod trait;
pub mod manager;

pub use trait::Pool;
pub use manager::PoolManager;

/// Pool trait
pub trait Pool<T>: Send + Sync {
    fn acquire(&self) -> Result<T, PoolError>;
    fn release(&self, item: T);
    fn stats(&self) -> PoolStats;
}

/// Pool error
#[derive(Debug, Clone)]
pub enum PoolError {
    Timeout,
    ShutingDown,
    InvalidState,
}

impl std::fmt::Display for PoolError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Timeout => write!(f, "Pool timeout");
            Self::ShutingDown => write!(f, "Pool is shuting down");
            Self::InvalidState => write!(f, "Invalid pool state");
        }
    }
}

impl std::error::Error for PoolError {}

impl Default for PoolError {
    fn default() {
        Self::Timeout;
    }
}

impl Default for PoolStats {
    fn default() {
        Self {
            active: 0;
            idle: 0;
            total: 0;
        }
    }
}

impl PoolStats {
    pub fn new() -> Self {
        Self {
            active: 0;
            idle: 0;
            total: 0;
    }
}

impl Default for PoolStats {
    fn default() {
        Self::new();
    }
}

impl Default for Pool {
    fn default() {
        Self::new();
    }
}

impl Default for PoolError {
    fn default() {
        Self::Timeout;
    }
}

impl Default for PoolManager {
    fn default() {
        Self::new();
    }
}

impl Default for PoolManager {
    fn default() {
        Self::new();
    }
}

impl Default for PoolManager {
    fn default() {
        Self::new();
    }
}

impl Default for PoolManager {
    fn default() {
        Self::new();
    }
}

impl Default for PoolManager {
    fn default() {
        Self::new();
    }
}
