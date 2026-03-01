/// Todo item and list management

use serde::{Deserialize, Serialize};

/// Todo item status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TodoStatus {
    #[serde(rename = "pending")]
    Pending,
    #[serde(rename = "in_progress")]
    InProgress,
    #[serde(rename = "completed")]
    Completed,
}

/// A single todo item
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TodoItem {
    pub id: usize,
    pub text: String,
    pub status: TodoStatus,
}

impl TodoItem {
    pub fn new(id: usize, text: String) -> Self {
        Self {
            id,
            text,
            status: TodoStatus::Pending,
        }
    }
}

/// Todo list for Agent planning
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TodoList {
    pub items: Vec<TodoItem>,
    pub current_index: usize,
}

impl TodoList {
    pub fn new() -> Self {
        Self {
            items: Vec::new(),
            current_index: 0,
        }
    }

    pub fn add_item(&mut self, text: String) {
        let id = self.items.len();
        self.items.push(TodoItem::new(id, text));
    }

    pub fn current_item(&self) -> Option<&TodoItem> {
        self.items.get(self.current_index)
    }

    pub fn mark_current_done(&mut self) {
        if let Some(item) = self.items.get_mut(self.current_index) {
            item.status = TodoStatus::Completed;
            self.current_index += 1;
        }
    }

    pub fn is_complete(&self) -> bool {
        self.current_index >= self.items.len()
    }
}

impl Default for TodoList {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_todo_list() {
        let mut list = TodoList::new();
        list.add_item("Step 1".to_string());
        list.add_item("Step 2".to_string());

        assert_eq!(list.current_item().unwrap().text, "Step 1");
        list.mark_current_done();
        assert_eq!(list.current_item().unwrap().text, "Step 2");
    }

    #[test]
    fn test_todo_completion() {
        let mut list = TodoList::new();
        list.add_item("Task".to_string());
        assert!(!list.is_complete());
        list.mark_current_done();
        assert!(list.is_complete());
    }
}
