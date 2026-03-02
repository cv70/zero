use crate::error::ToolError;
/// Task manager for CRUD operations
use crate::task::model::{Task, TaskStatus};
use async_trait::async_trait;
use sqlx::sqlite::{SqliteConnectOptions, SqlitePoolOptions};
use sqlx::{Row, SqlitePool};
use std::collections::HashMap;
use std::path::Path;
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
        let duration = now
            .duration_since(SystemTime::UNIX_EPOCH)
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

/// SQLite-backed task manager
pub struct SqliteTaskManager {
    pool: SqlitePool,
}

impl SqliteTaskManager {
    /// Create a new sqlite task manager and initialize schema.
    pub fn new(path: impl AsRef<Path>) -> Result<Self, ToolError> {
        let options = SqliteConnectOptions::new()
            .filename(path.as_ref())
            .create_if_missing(true);
        let pool = SqlitePoolOptions::new()
            .max_connections(5)
            .connect_lazy_with(options);
        Ok(Self { pool })
    }

    async fn ensure_schema(&self) -> Result<(), ToolError> {
        sqlx::query(
            "CREATE TABLE IF NOT EXISTS tasks (
                id TEXT PRIMARY KEY,
                title TEXT NOT NULL,
                description TEXT NOT NULL,
                status TEXT NOT NULL,
                dependencies TEXT NOT NULL,
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL,
                result TEXT,
                metadata TEXT NOT NULL
            )",
        )
        .execute(&self.pool)
        .await
        .map_err(|e| ToolError::ExecutionFailed(format!("init sqlite schema failed: {e}")))?;
        Ok(())
    }
}

#[async_trait]
impl TaskManager for SqliteTaskManager {
    async fn create(&self, task: Task) -> Result<String, ToolError> {
        self.ensure_schema().await?;
        let id = task.id.clone();
        let deps = serde_json::to_string(&task.dependencies).map_err(|e| {
            ToolError::ExecutionFailed(format!("serialize dependencies failed: {e}"))
        })?;
        let result = serde_json::to_string(&task.result)
            .map_err(|e| ToolError::ExecutionFailed(format!("serialize result failed: {e}")))?;
        let metadata = serde_json::to_string(&task.metadata)
            .map_err(|e| ToolError::ExecutionFailed(format!("serialize metadata failed: {e}")))?;
        let status = match task.status {
            TaskStatus::Pending => "pending",
            TaskStatus::Running => "running",
            TaskStatus::Completed => "completed",
            TaskStatus::Failed => "failed",
        };

        sqlx::query(
            "INSERT INTO tasks (id, title, description, status, dependencies, created_at, updated_at, result, metadata)
             VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)
             ON CONFLICT(id) DO UPDATE SET
             title=excluded.title,
             description=excluded.description,
             status=excluded.status,
             dependencies=excluded.dependencies,
             created_at=excluded.created_at,
             updated_at=excluded.updated_at,
             result=excluded.result,
             metadata=excluded.metadata",
        )
        .bind(task.id)
        .bind(task.title)
        .bind(task.description)
        .bind(status)
        .bind(deps)
        .bind(task.created_at)
        .bind(task.updated_at)
        .bind(result)
        .bind(metadata)
        .execute(&self.pool)
        .await
        .map_err(|e| ToolError::ExecutionFailed(format!("upsert task failed: {e}")))?;
        Ok(id)
    }

    async fn get(&self, id: &str) -> Result<Option<Task>, ToolError> {
        self.ensure_schema().await?;
        let row = sqlx::query(
            "SELECT id, title, description, status, dependencies, created_at, updated_at, result, metadata
             FROM tasks WHERE id = ?",
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| ToolError::ExecutionFailed(format!("query task failed: {e}")))?;

        let Some(row) = row else {
            return Ok(None);
        };

        let raw_status: String = row.get("status");
        let status = match raw_status.as_str() {
            "pending" => TaskStatus::Pending,
            "running" => TaskStatus::Running,
            "completed" => TaskStatus::Completed,
            "failed" => TaskStatus::Failed,
            _ => TaskStatus::Pending,
        };
        let deps_json: String = row.get("dependencies");
        let result_json: Option<String> = row.get("result");
        let metadata_json: String = row.get("metadata");

        Ok(Some(Task {
            id: row.get("id"),
            title: row.get("title"),
            description: row.get("description"),
            status,
            dependencies: serde_json::from_str(&deps_json).unwrap_or_default(),
            created_at: row.get("created_at"),
            updated_at: row.get("updated_at"),
            result: result_json
                .as_deref()
                .and_then(|v| serde_json::from_str(v).ok())
                .unwrap_or(None),
            metadata: serde_json::from_str(&metadata_json).unwrap_or_default(),
        }))
    }

    async fn list(&self) -> Result<Vec<Task>, ToolError> {
        self.ensure_schema().await?;
        let rows = sqlx::query(
            "SELECT id, title, description, status, dependencies, created_at, updated_at, result, metadata
             FROM tasks ORDER BY updated_at DESC",
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| ToolError::ExecutionFailed(format!("query list failed: {e}")))?;

        let mut tasks = Vec::with_capacity(rows.len());
        for row in rows {
            let raw_status: String = row.get("status");
            let status = match raw_status.as_str() {
                "pending" => TaskStatus::Pending,
                "running" => TaskStatus::Running,
                "completed" => TaskStatus::Completed,
                "failed" => TaskStatus::Failed,
                _ => TaskStatus::Pending,
            };
            let deps_json: String = row.get("dependencies");
            let result_json: Option<String> = row.get("result");
            let metadata_json: String = row.get("metadata");
            tasks.push(Task {
                id: row.get("id"),
                title: row.get("title"),
                description: row.get("description"),
                status,
                dependencies: serde_json::from_str(&deps_json).unwrap_or_default(),
                created_at: row.get("created_at"),
                updated_at: row.get("updated_at"),
                result: result_json
                    .as_deref()
                    .and_then(|v| serde_json::from_str(v).ok())
                    .unwrap_or(None),
                metadata: serde_json::from_str(&metadata_json).unwrap_or_default(),
            });
        }
        Ok(tasks)
    }

    async fn list_pending(&self) -> Result<Vec<Task>, ToolError> {
        let tasks = self.list().await?;
        Ok(tasks
            .into_iter()
            .filter(|t| t.status == TaskStatus::Pending)
            .collect())
    }

    async fn update_status(&self, id: &str, status: TaskStatus) -> Result<(), ToolError> {
        self.ensure_schema().await?;
        let now = SystemTime::now();
        let duration = now
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap_or(std::time::Duration::from_secs(0));
        let updated_at = duration.as_secs().to_string();
        let status = match status {
            TaskStatus::Pending => "pending",
            TaskStatus::Running => "running",
            TaskStatus::Completed => "completed",
            TaskStatus::Failed => "failed",
        };
        let changed = sqlx::query("UPDATE tasks SET status = ?, updated_at = ? WHERE id = ?")
            .bind(status)
            .bind(updated_at)
            .bind(id)
            .execute(&self.pool)
            .await
            .map_err(|e| ToolError::ExecutionFailed(format!("update task status failed: {e}")))?
            .rows_affected();
        if changed == 0 {
            return Err(ToolError::ExecutionFailed(format!(
                "Task not found: {}",
                id
            )));
        }
        Ok(())
    }

    async fn complete(&self, id: &str) -> Result<(), ToolError> {
        self.update_status(id, TaskStatus::Completed).await
    }

    async fn fail(&self, id: &str) -> Result<(), ToolError> {
        self.update_status(id, TaskStatus::Failed).await
    }

    async fn delete(&self, id: &str) -> Result<(), ToolError> {
        self.ensure_schema().await?;
        sqlx::query("DELETE FROM tasks WHERE id = ?")
            .bind(id)
            .execute(&self.pool)
            .await
            .map_err(|e| ToolError::ExecutionFailed(format!("delete task failed: {e}")))?;
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
