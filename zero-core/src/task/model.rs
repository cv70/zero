use crate::runtime::TaskState;
/// Task data model
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, time::SystemTime};

/// Task status enumeration
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TaskStatus {
    #[serde(rename = "pending")]
    Pending,
    #[serde(rename = "running")]
    Running,
    #[serde(rename = "completed")]
    Completed,
    #[serde(rename = "failed")]
    Failed,
}

/// Task result information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskResult {
    pub output: String,
    pub exit_code: i32,
}

/// Task success contract for post-execution verification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskSuccessContract {
    pub required_substrings: Vec<String>,
}

/// Task model
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Task {
    pub id: String,
    pub title: String,
    pub description: String,
    pub status: TaskStatus,
    pub dependencies: Vec<String>,
    pub created_at: String,
    pub updated_at: String,
    pub result: Option<TaskResult>,
    pub metadata: HashMap<String, String>,
}

impl Task {
    /// Create a new task
    pub fn new(id: String, title: String, description: String) -> Self {
        let now = SystemTime::now();
        let duration = now
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap_or(std::time::Duration::from_secs(0));
        let now = duration.as_secs().to_string();
        Self {
            id,
            title,
            description,
            status: TaskStatus::Pending,
            dependencies: Vec::new(),
            created_at: now.clone(),
            updated_at: now,
            result: None,
            metadata: HashMap::new(),
        }
    }

    /// Add a dependency
    pub fn with_dependency(mut self, dep_id: String) -> Self {
        self.dependencies.push(dep_id);
        self
    }

    /// Set status
    pub fn with_status(mut self, status: TaskStatus) -> Self {
        self.status = status;
        self
    }

    /// Check if task is blocked (has unmet dependencies)
    pub fn is_blocked(&self, completed_tasks: &[String]) -> bool {
        self.dependencies
            .iter()
            .any(|dep| !completed_tasks.contains(dep))
    }
}

impl From<TaskState> for TaskStatus {
    fn from(value: TaskState) -> Self {
        match value {
            TaskState::Pending | TaskState::Runnable | TaskState::Waiting => TaskStatus::Pending,
            TaskState::Running => TaskStatus::Running,
            TaskState::Succeeded => TaskStatus::Completed,
            TaskState::Failed | TaskState::Compensated => TaskStatus::Failed,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_task() {
        let task = Task::new(
            "1".to_string(),
            "Test".to_string(),
            "Description".to_string(),
        );
        assert_eq!(task.id, "1");
        assert_eq!(task.status, TaskStatus::Pending);
    }

    #[test]
    fn test_task_with_dependencies() {
        let task = Task::new("1".to_string(), "Test".to_string(), "Desc".to_string())
            .with_dependency("0".to_string());
        assert_eq!(task.dependencies.len(), 1);
    }

    #[test]
    fn test_task_is_blocked() {
        let task = Task::new("2".to_string(), "Test".to_string(), "Desc".to_string())
            .with_dependency("1".to_string());
        assert!(task.is_blocked(&vec![]));
        assert!(!task.is_blocked(&vec!["1".to_string()]));
    }
}
