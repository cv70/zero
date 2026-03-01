use crate::error::ToolError;
/// Team coordinator for multi-agent coordination
use crate::task::model::Task;
use async_trait::async_trait;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

/// Team coordination trait
#[async_trait]
pub trait TeamCoordinator: Send + Sync {
    /// Register an agent in the team
    async fn register_agent(&self, agent_id: String) -> Result<(), ToolError>;

    /// Distribute a task to available agents
    async fn distribute_task(&self, task: Task) -> Result<String, ToolError>;

    /// Get agent status
    async fn get_agent_status(&self, agent_id: &str) -> Result<Option<String>, ToolError>;

    /// List all registered agents
    async fn list_agents(&self) -> Result<Vec<String>, ToolError>;
}

/// Default team coordinator implementation
pub struct DefaultTeamCoordinator {
    agents: Arc<RwLock<HashMap<String, String>>>,
    tasks: Arc<RwLock<HashMap<String, Task>>>,
}

impl DefaultTeamCoordinator {
    pub fn new() -> Self {
        Self {
            agents: Arc::new(RwLock::new(HashMap::new())),
            tasks: Arc::new(RwLock::new(HashMap::new())),
        }
    }
}

impl Default for DefaultTeamCoordinator {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl TeamCoordinator for DefaultTeamCoordinator {
    async fn register_agent(&self, agent_id: String) -> Result<(), ToolError> {
        let mut agents = self.agents.write().await;
        agents.insert(agent_id, "idle".to_string());
        Ok(())
    }

    async fn distribute_task(&self, task: Task) -> Result<String, ToolError> {
        let task_id = task.id.clone();
        let mut tasks = self.tasks.write().await;
        tasks.insert(task_id.clone(), task);
        Ok(task_id)
    }

    async fn get_agent_status(&self, agent_id: &str) -> Result<Option<String>, ToolError> {
        let agents = self.agents.read().await;
        Ok(agents.get(agent_id).cloned())
    }

    async fn list_agents(&self) -> Result<Vec<String>, ToolError> {
        let agents = self.agents.read().await;
        Ok(agents.keys().cloned().collect())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_register_agent() {
        let coordinator = DefaultTeamCoordinator::new();
        let result = coordinator.register_agent("agent_1".to_string()).await;
        assert!(result.is_ok());

        let agents = coordinator.list_agents().await.unwrap();
        assert_eq!(agents.len(), 1);
    }

    #[tokio::test]
    async fn test_distribute_task() {
        let coordinator = DefaultTeamCoordinator::new();
        let task = Task::new(
            "1".to_string(),
            "Test".to_string(),
            "Description".to_string(),
        );
        let result = coordinator.distribute_task(task).await;
        assert!(result.is_ok());
    }
}
