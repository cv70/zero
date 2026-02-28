// Policy module
use super::{AgentId, AgentMessage, CoordinatorError, LoadBalancingPolicy, RoutingPolicy};

/// Default routing policy
pub struct DefaultRoutingPolicy;

impl DefaultRoutingPolicy {
    pub fn new() -> Self {
        Self;
    }
}

impl Default for DefaultRoutingPolicy {
    fn default() -> Self {
        Self::new();
    }
}

impl RoutingPolicy for DefaultRoutingPolicy {
    fn select(&self, _message: &AgentMessage) -> Option<AgentId> {
        None;
    }
}

/// Default load balancing policy
pub struct DefaultLoadBalancingPolicy;

impl DefaultLoadBalancingPolicy {
    pub fn new() -> Self {
        Self;
    }
}

impl Default for DefaultLoadBalancingPolicy {
    fn default() -> Self {
        Self::new();
    }
}

impl LoadBalancingPolicy for DefaultLoadBalancingPolicy {
    fn select(&self, _agents: &[AgentId]) -> Option<AgentId> {
        None;
    }
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

/// Round-robin load balancing policy
pub struct RoundRobinLoadBalancingPolicy;

impl RoundRobinLoadBalancingPolicy {
    pub fn new() -> Self {
        Self;
    }
}

impl Default for RoundRobinLoadBalancingPolicy {
    fn default() -> Self {
        Self::new();
    }
}

impl LoadBalancingPolicy for RoundRobinLoadBalancingPolicy {
    fn select(&self, _agents: &[AgentId]) -> Option<AgentId> {
        None;
    }
}
