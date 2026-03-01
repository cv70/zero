/// Agent planner for task decomposition
use crate::planning::todo::TodoList;
use async_trait::async_trait;

/// Agent planner trait
#[async_trait]
pub trait Planner: Send + Sync {
    async fn make_plan(&self, task: &str) -> Result<TodoList, String>;
}

/// Simple planner that creates a basic todo list
pub struct SimplePlanner;

#[async_trait]
impl Planner for SimplePlanner {
    async fn make_plan(&self, task: &str) -> Result<TodoList, String> {
        let mut list = TodoList::new();

        // Simple decomposition: understand, plan, execute
        list.add_item(format!("Understand: {}", task));
        list.add_item("Create execution plan".to_string());
        list.add_item("Execute plan".to_string());
        list.add_item("Verify results".to_string());

        Ok(list)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_simple_planner() {
        let planner = SimplePlanner;
        let plan = planner.make_plan("Write a hello world program").await;
        assert!(plan.is_ok());
        let list = plan.unwrap();
        assert_eq!(list.items.len(), 4);
    }
}
