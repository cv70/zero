use crate::error::ToolError;
/// Task storage abstraction
use crate::task::model::Task;

/// Task storage trait
pub trait TaskStore: Send + Sync {
    fn save(&self, task: &Task) -> Result<(), ToolError>;
    fn load(&self, id: &str) -> Result<Option<Task>, ToolError>;
    fn list(&self) -> Result<Vec<Task>, ToolError>;
    fn delete(&self, id: &str) -> Result<(), ToolError>;
}
