// Queue module
use super::{Task, TaskPriority, TaskStatus};
use crate::scheduler::PriorityTaskQueue;

/// Task queue trait
pub trait Queue: Send + Sync {
    fn push(&mut self, task: Box<dyn Task>);
    fn pop(&mut self) -> Option<Box<dyn Task>>;
    fn peek(&self) -> Option<&dyn Task>;
    fn len(&self) -> usize;
    fn is_empty(&self) -> bool;
}

/// Priority-based queue
pub struct PriorityQueue {
    inner: std::collections::BinaryHeap<(std::cmp::Reverse<TaskPriority>, Box<dyn Task>)>,
}

impl PriorityQueue {
    pub fn new() -> Self {
        Self {
            inner: std::collections::BinaryHeap::new(),
        }
    }

    pub fn push(&mut self, task: Box<dyn Task>) {
        self.inner.push((std::cmp::Reverse(task.priority()), task));
    }

    pub fn pop(&mut self) -> Option<Box<dyn Task>> {
        self.inner.pop().map(|(_, task)| task)
    }

    pub fn peek(&self) -> Option<&dyn Task> {
        self.inner.peek().map(|(_, task)| task.as_ref())
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

/// Priority-based task queue implementation
pub struct PriorityTaskQueue {
    inner: std::collections::BinaryHeap<(std::cmp::Reverse<TaskPriority>, Box<dyn Task>)>,
}

impl PriorityTaskQueue {
    pub fn new() -> Self {
        Self {
            inner: std::collections::BinaryHeap::new(),
        }
    }

    pub fn push(&mut self, task: Box<dyn Task>) {
        self.inner.push((std::cmp::Reverse(task.priority()), task));
    }

    pub fn pop(&mut self) -> Option<Box<dyn Task>> {
        self.inner.pop().map(|(_, task)| task)
    }

    pub fn peek(&self) -> Option<&dyn Task> {
        self.inner.peek().map(|(_, task)| task.as_ref())
    }

    pub fn len(&self) -> usize {
        self.inner.len()
    }

    pub fn is_empty(&self) -> bool {
        self.inner.is_empty()
    }
}

impl Default for PriorityTaskQueue {
    fn default() -> Self {
        Self::new()
    }
}
