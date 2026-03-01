use crate::error::ToolError;
/// Task manager for CRUD operations
use crate::task::model::{Task, TaskStatus};
use async_trait::async_trait;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::SystemTime;
use tokio::sync::RwLock;

/// Task manager trait
#[async_trait]
pub trait TaskManager: Send + Sync {
    /// Create a new task
    async fn create(&self, task: Task) -> Result<String, ToolError>;

    /// Get a task by ID
    async fn get(&self, id: &str) -> Result<Option<Task>, ToolError>;

    /// List all tasks
    async fn list(&self) -> Result<Vec<Task>, ToolError>;

    /// List pending tasks
    async fn list_pending(&self) -> Result<Vec<Task>, ToolError>;

    /// Update task status
    async fn update_status(&self, id: &str, status: TaskStatus) -> Result<(), ToolError>;

    /// Mark task as completed
    async fn complete(&self, id: &str) -> Result<(), ToolError>;

    /// Mark task as failed
    async fn fail(&self, id: &str) -> Result<(), ToolError>;

    /// Delete a task
    async fn delete(&self, id: &str) -> Result<(), ToolError>;
}

/// In-memory task manager
pub struct InMemoryTaskManager {
    tasks: Arc<RwLock<HashMap<String, Task>>>,
}

impl InMemoryTaskManager {
    /// Create a new in-memory task manager
    pub fn new() -> Self {
        Self {
            tasks: Arc::new(RwLock::new(HashMap::new())),
        }
    }
}

impl Default for InMemoryTaskManager {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl TaskManager for InMemoryTaskManager {
    async fn create(&self, task: Task) -> Result<String, ToolError> {
        let id = task.id.clone();
        let mut tasks = self.tasks.write().await;
        tasks.insert(id.clone(), task);
        Ok(id)
    }

    async fn get(&self, id: &str) -> Result<Option<Task>, ToolError> {
        let tasks = self.tasks.read().await;
        Ok(tasks.get(id).cloned())
    }

    async fn list(&self) -> Result<Vec<Task>, ToolError> {
        let tasks = self.tasks.read().await;
        Ok(tasks.values().cloned().collect())
    }

    async fn list_pending(&self) -> Result<Vec<Task>, ToolError> {
        let tasks = self.tasks.read().await;
        Ok(tasks
            .values()
            .filter(|t| t.status == TaskStatus::Pending)
            .cloned()
            .collect())
    }

    async fn update_status(&self, id: &str, status: TaskStatus) -> Result<(), ToolError> {
        let now = SystemTime::now();
        let duration = now.duration_since(SystemTime::UNIX_EPOCH)
            .unwrap_or(std::time::Duration::from_secs(0));
        let mut tasks = self.tasks.write().await;
        if let Some(task) = tasks.get_mut(id) {
            task.status = status;
            task.updated_at = duration.as_secs().to_string();
            Ok(())
        } else {
            Err(ToolError::ExecutionFailed(format!(
                "Task not found: {}",
                id
            )))
        }
    }

    async fn complete(&self, id: &str) -> Result<(), ToolError> {
        self.update_status(id, TaskStatus::Completed).await
    }

    async fn fail(&self, id: &str) -> Result<(), ToolError> {
        self.update_status(id, TaskStatus::Failed).await
    }

    async fn delete(&self, id: &str) -> Result<(), ToolError> {
        let mut tasks = self.tasks.write().await;
        tasks.remove(id);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_create_task() {
        let manager = InMemoryTaskManager::new();
        let task = Task::new("1".to_string(), "Test".to_string(), "Desc".to_string());
        let result = manager.create(task).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_get_task() {
        let manager = InMemoryTaskManager::new();
        let task = Task::new("1".to_string(), "Test".to_string(), "Desc".to_string());
        manager.create(task.clone()).await.unwrap();
        let retrieved = manager.get("1").await.unwrap();
        assert!(retrieved.is_some());
    }

    #[tokio::test]
    async fn test_list_tasks() {
        let manager = InMemoryTaskManager::new();
        manager
            .create(Task::new(
                "1".to_string(),
                "T1".to_string(),
                "D1".to_string(),
            ))
            .await
            .unwrap();
        manager
            .create(Task::new(
                "2".to_string(),
                "T2".to_string(),
                "D2".to_string(),
            ))
            .await
            .unwrap();

        let tasks = manager.list().await.unwrap();
        assert_eq!(tasks.len(), 2);
    }

    #[tokio::test]
    async fn test_update_status() {
        let manager = InMemoryTaskManager::new();
        manager
            .create(Task::new(
                "1".to_string(),
                "Test".to_string(),
                "Desc".to_string(),
            ))
            .await
            .unwrap();

        manager.complete("1").await.unwrap();
        let task = manager.get("1").await.unwrap().unwrap();
        assert_eq!(task.status, TaskStatus::Completed);
    }
}
