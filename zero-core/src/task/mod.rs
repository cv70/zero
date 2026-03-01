pub mod manager;
/// Task management system
///
/// Provides persistent task storage, dependency tracking, and execution management
pub mod model;
pub mod store;

pub use manager::TaskManager;
pub use model::{Task, TaskResult, TaskStatus};
pub use store::TaskStore;
