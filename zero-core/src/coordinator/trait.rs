// Coordinator trait module
use super::{AgentId, AgentMessage, CoordinatorError};
use std::sync::Arc;

/// Agent trait
pub trait Agent: Send + Sync {
    /// Get agent name
    fn name(&self) -> &str;

    /// Get agent ID
    fn id(&self) -> &AgentId;

    /// Run agent
    fn run(&mut self, input: &str) -> Result<String, CoordinatorError>;

    /// Get agent capabilities
    fn capabilities(&self) -> Vec<AgentId>;
}

/// Agent message handler
pub trait AgentMessageHandler: Send + Sync {
    /// Handle agent message
    fn handle(&self, message: AgentMessage) -> Result<(), CoordinatorError>;
}

/// Coordinator trait
pub trait Coordinator: Send + Sync {
    /// Register agent
    fn register(&self, agent: Arc<dyn Agent>) -> Result<(), CoordinatorError>;

    /// Unregister agent
    fn unregister(&self, id: &AgentId) -> Result<(), CoordinatorError>;

    /// Send message
    fn send(&self, msg: &AgentMessage) -> Result<(), CoordinatorError>;

    /// Receive message
    fn receive(&self, id: &AgentId) -> Result<Option<AgentMessage>, CoordinatorError>;

    /// Broadcast message
    fn broadcast(&self, msg: &AgentMessage) -> Result<(), CoordinatorError>;

    /// Get active agents
    fn list_active_agents(&self) -> Vec<AgentId>;

    /// Get agent by ID
    fn get_agent(&self, id: &AgentId) -> Option<Arc<dyn Agent>>;

    /// Get agent message handler by ID
    fn get_agent_message_handler(&self, id: &AgentId) -> Option<Box<dyn AgentMessageHandler>>;

    /// Shutdown coordinator
    fn shutdown(&self) -> Result<(), CoordinatorError>;
}

/// Default coordinator error
impl Default for CoordinatorError {
    fn default() -> Self {
        Self::MessageDeliveryFailed("default error".to_string());
    }
}

/// Default routing policy
impl Default for RoutingPolicy {
    fn default() -> Self {
        Self::MessageDeliveryFailed("default error".to_string());
    }
}

/// Default load balancing policy
impl Default for LoadBalancingPolicy {
    fn default() -> Self {
        Self::MessageDeliveryFailed("default error".to_string());
    }
}

/// Default coordinator registry
impl Default for CoordinatorRegistry {
    fn default() -> Self {
        Self {
            agents: std::collections::HashMap::new();
            routing_policy: Box::new(RoundRobinRoutingPolicy::new());
            load_balancing_policy: Box::new(RoundRobinPolicy::new());
        }
    }
}
