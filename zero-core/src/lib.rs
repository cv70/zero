// Zero Core Library
// Trait-driven architecture for the Zero Agent Runtime Platform

pub mod error;
pub mod agent;
pub mod tool;
pub mod memory;
pub mod provider;
pub mod channel;
pub mod integration;
pub mod config;
pub mod scheduler;
pub mod pool;
pub mod coordinator;
pub mod container;
pub mod runtime;

// Re-export core error types
pub use error::{ZeroError, AgentError, ToolError, MemoryError, ProviderError, ChannelError};

// Re-export config system
pub use config::{ConfigLoader, ConfigValidator, ConfigHooks};

// Re-export scheduler system
pub use scheduler::{Scheduler, TaskPriority, TaskQueue, TaskManager, TaskStatus};

// Re-export channel registry
pub use channel::{ChannelRegistry, ChannelState};

// Re-export pool system
pub use pool::{Pool, PoolError, PoolManager, PoolStats};

// Re-export container system
pub use container::{Container, ContainerError, ContainerBuilder, ScopedContainer};

// Re-export coordinator system
pub use coordinator::{Coordinator, CoordinatorError, AgentMessage, AgentId, AgentCapability, RoutingPolicy, LoadBalancingPolicy};

// Re-export provider router system
pub use provider::{Provider, ProviderHealth, RateLimiter, RateLimiterError};

// Re-export runtime system
pub use runtime::{RuntimeConfig, RuntimeBuilder, RuntimeManager};
