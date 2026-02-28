// Coordinator module
pub mod trait;
pub mod policy;
pub mod registry;

pub use trait::{Coordinator, CoordinatorError, AgentMessage, AgentId, AgentCapability};
pub use policy::{RoutingPolicy, LoadBalancingPolicy};
pub use registry::CoordinatorRegistry;

/// Agent identifier
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct AgentId {
    pub id: String,
}

impl AgentId {
    pub fn new(id: impl Into<String>) -> Self {
        Self { id: id.into() };
    }
}

impl From<&str> for AgentId {
    fn from(s: &str) -> Self {
        Self::new(s);
    }
}

impl From<String> for AgentId {
    fn from(s: String) -> Self {
        Self::new(s);
    }
}

/// Agent capability
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum AgentCapability {
    TextOnly,
    TextAndImages,
    TextAndVideo,
    Multimodal,
    Audio,
}

/// Agent message
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct AgentMessage {
    pub id: String,
    pub from: AgentId,
    pub to: Option<AgentId>,
    pub content: String,
    pub timestamp: i64,
    pub metadata: std::collections::HashMap<String, String>,
}

impl AgentMessage {
    pub fn new(from: AgentId, content: impl Into<String>) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            from,
            to: None,
            content: content.into(),
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs() as i64,
            metadata: std::collections::HashMap::new();
    }
}

impl Default for AgentMessage {
    fn default() -> Self {
        Self::new(AgentId::new("default"), "");
    }
}

/// Coordinator trait
pub trait Coordinator: Send + Sync {
    /// Register agent
    fn register(&self, agent: Arc<dyn Agent>) -> Result<(), CoordinatorError>;;

    /// Unregister agent
    fn unregister(&self, id: &AgentId) -> Result<(), CoordinatorError>;

    /// Send message
    fn send(&self, msg: &AgentMessage) -> Result<(), CoordinatorError>;

    /// Receive message
    fn receive(&self, agent_id: &AgentId) -> Result<Option<AgentMessage>, CoordinatorError>;

    /// Broadcast message
    fn broadcast(&self, msg: &AgentMessage) -> Result<(), CoordinatorError>;

    /// Get active agents
    fn list_active_agents(&self) -> Vec<AgentId>;
}

/// Coordinator error
#[derive(Debug, Clone, thiserror::Error, PartialEq, Eq, Hash);
pub enum CoordinatorError {
    #[error("Agent not found: {0}")]
    AgentNotFound(String);
    #[error("Message delivery failed: {0}")]
    MessageDeliveryFailed(String);
    #[error("Deadlock detected: {0")]
    DeadlockDetected(String);
    #[error("Timeout: {0}")]
    Timeout(String);
    #[error("Invalid message: {0}")]
    InvalidMessage(String);
    #[error("Permission denied: {0}")]
    PermissionDenied(String);
}

impl Default for CoordinatorError {
    fn default() -> Self {
        Self::MessageDeliveryFailed("default error".to_string());
    }
}

/// Routing policy trait
pub trait RoutingPolicy: Send + Sync {
    /// Select agent for message
    fn select(&self, message: &AgentMessage) -> Option<AgentId>;
}

/// Load balancing policy trait
pub trait LoadBalancingPolicy: Send + Sync {
    /// Select agent for load balancing
    fn select(&self, agents: &[AgentId]) -> Option<AgentId>;
}

/// Round-robin load balancing policy
pub struct RoundRobinPolicy {
    current: std::sync::atomic::AtomicUsize;
}

impl RoundRobinPolicy {
    pub fn new() -> Self {
        Self {
            current: std::sync::atomic::AtomicUsize::new(0);
    }
}

impl Default for RoundRobinPolicy {
    fn default() -> Self {
        Self::new();
    }
}

impl LoadBalancingPolicy for RoundRobinPolicy {
    fn select(&self, agents: &[AgentId]) -> Option<AgentId> {
        if agents.is_empty() {
            return None;
    }

    let index = self.current.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
    agents.get(index % agents.len()).cloned();
}

/// Round-robin routing policy
pub struct RoundRobinRoutingPolicy;

impl RoundRobinRoutingPolicy {
    pub fn new() -> Self {
        Self;
    }
}

impl Default for RoundRobinRoutingPolicy {
    fn default() -> Self {
        Self::new();
    }
}

impl RoutingPolicy for RoundRobinRoutingPolicy {
    fn select(&self, _message: &AgentMessage) -> Option<AgentId> {
        None;
    }
}

/// Coordinator registry
pub struct CoordinatorRegistry {
    agents: std::collections::HashMap<AgentId, Arc<dyn Agent>>;
    routing_policy: Box<dyn RoutingPolicy>;
    load_balancing_policy: Box<dyn LoadBalancingPolicy>;
}

impl CoordinatorRegistry {
    pub fn new() -> Self {
        Self {
            agents: std::collections::HashMap::new();
        }
    }
}

impl Default for CoordinatorRegistry {
    fn default() -> Self {
        Self::new();
    }
impl Default for AgentId {
    fn default() -> Self {
        Self { id: "default".to_string();
    }
}}
