// Scheduler module
pub mod trait;
pub mod priority;
pub mod queue;
pub mod manager;

pub use trait::Scheduler;
pub use priority::TaskPriority;
pub use queue::TaskQueue;
pub use manager::TaskManager;

/// Task priority
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum TaskPriority {
   Lowest = 0,
   Lower = 1,
   Low = 2,
   Normal = 3,
   High = 4,
   Higher = 5,
   Highest = 6,
   Critical = 7,
}

impl Default for TaskPriority {
    fn default() -> Self {
        TaskPriority::Normal
    }
}

/// Task status
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TaskStatus {
    Pending,
    Running,
    Completed,
    Failed,
    Cancelled,
}

/// Task trait
pub trait Task: Send + Sync {
    fn id(&self) -> &str;
    fn priority(&self) -> TaskPriority;
    fn execute(&mut self) -> Result<(), String>;
    fn status(&self) -> TaskStatus;
    fn timeout(&self) -> Option<std::time::Duration> {
        None
    }
    fn retries(&self) -> u32 {
        0
    }
}

/// Task queue trait
pub trait TaskQueue: Send + Sync {
    fn push(&mut self, task: Box<dyn Task>);
    fn pop(&mut self) -> Option<Box<dyn Task>>;
    fn peek(&self) -> Option<&dyn Task>;
    fn len(&self) -> usize;
    fn is_empty(&self) -> bool;
}

/// Task manager trait
pub trait TaskManager: Send + Sync {
    fn submit(&self, task: Box<dyn Task>) -> Result<String, String>;
    fn cancel(&self, id: &str) -> Result<(), String>;
    fn status(&self, id: &str) -> Result<TaskStatus, String>;
    fn list_tasks(&self) -> Vec<Box<dyn Task>>;
    fn shutdown(&self) -> Result<(), String>;
}

/// Priority-based task queue
pub struct PriorityTaskQueue {
    tasks: Vec<(TaskPriority, Box<dyn Task>)>,
    priority_queue: std::collections::BinaryHeap<(std::cmp::Reverse<TaskPriority>, Box<dyn Task>)>,
}

impl PriorityTaskQueue {
    pub fn new() -> Self {
        Self {
            tasks: Vec::new(),
            priority_queue: std::collections::BinaryHeap::new(),
        }
    }
}

impl Default for PriorityTaskQueue {
    fn default() -> Self {
        Self::new()
    }
}

impl TaskQueue for PriorityTaskQueue {
    fn push(&mut self, task: Box<dyn Task>) {
        let priority = task.priority();
        self.priority_queue.push((std::cmp::Reverse(priority), task));
    }

    fn pop(&mut self) -> Option<Box<dyn Task>> {
        self.priority_queue.pop().map(|(_, task)| task)
    }

    fn peek(&self) -> Option<&dyn Task> {
        self.priority_queue.peek().map(|(_, task)| task.as_ref())
    }

    fn len(&self) -> usize {
        self.priority_queue.len()
    }

    fn is_empty(&self) -> bool {
        self.priority_queue.is_empty()
    }
}

/// Task manager implementation
pub struct DefaultTaskManager {
    task_queue: PriorityTaskQueue,
}

impl DefaultTaskManager {
    pub fn new() -> Self {
        Self {
            task_queue: PriorityTaskQueue::new(),
        }
    }
}

impl Default for DefaultTaskManager {
    fn default() -> Self {
        Self::new()
    }
}

impl TaskManager for DefaultTaskManager {
    fn submit(&self, task: Box<dyn Task>) -> Result<String, String> {
        self.task_queue.push(task);
        Ok("submitted".to_string()
            .map_err(|e| e.to_string())?)
    }

    fn cancel(&self, _id: &str) -> Result<(), String> {
        Ok(())
    }

    fn status(&self, _id: &str) -> Result<TaskStatus, String> {
        Ok(TaskStatus::Pending)
    }

    fn list_tasks(&self) -> Vec<Box<dyn Task>> {
        Vec::new()
    }

    fn shutdown(&self) -> Result<(), String> {
        Ok(())
    }
}
