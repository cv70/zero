// Priority module
pub use crate::scheduler::TaskPriority;

/// Priority queue wrapper
pub struct PriorityQueue {
    inner: std::collections::BinaryHeap<crate::scheduler::TaskPriority>,
}

impl PriorityQueue {
    pub fn new() -> Self {
        Self {
            inner: std::collections::BinaryHeap::new(),
        }
    }

    pub fn push(&mut self, priority: TaskPriority) {
        self.inner.push(priority);
    }

    pub fn pop(&mut self) -> Option<TaskPriority> {
        self.inner.pop()
    }

    pub fn peek(&self) -> Option<&TaskPriority> {
        self.inner.peek()
    }

    pub fn len(&self) -> usize {
        self.inner.len()
    }

    pub fn is_empty(&self) -> bool {
        self.inner.is_empty()
    }
}

impl Default for PriorityQueue {
    fn default() -> Self {
        Self::new()
    }
}

/// Priority-based scheduler
pub struct PriorityScheduler {
    queue: PriorityQueue,
}

impl PriorityScheduler {
    pub fn new() -> Self {
        Self {
            queue: PriorityQueue::new(),
        }
    }

    pub fn schedule(&mut self, priority: TaskPriority) {
        self.queue.push(priority);
    }

    pub fn get_next(&mut self) -> Option<TaskPriority> {
        self.queue.pop()
    }
}

impl Default for PriorityScheduler {
    fn default() -> Self {
        Self::new()
    }
}

/// Priority manager
pub struct PriorityManager {
    priorities: std::collections::HashMap<String, TaskPriority>,
}

impl PriorityManager {
    pub fn new() -> Self {
        Self {
            priorities: std::collections::HashMap::new(),
        }
    }

    pub fn set_priority(&mut self, id: &str, priority: TaskPriority) {
        self.priorities.insert(id.to_string(), priority);
    }

    pub fn get_priority(&self, id: &str) -> Option<&TaskPriority> {
        self.priorities.get(id)
    }

    pub fn remove_priority(&mut self, id: &str) {
        self.priorities.remove(id);
    }

    pub fn list_priorities(&self) -> Vec<(String, TaskPriority)> {
        self.priorities.clone().into_iter().collect()
    }
}

impl Default for PriorityManager {
    fn default() -> Self {
        Self::new()
    }
}
