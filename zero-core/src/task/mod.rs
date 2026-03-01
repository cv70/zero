/// Task management system
///
/// Provides persistent task storage, dependency tracking, and execution management

pub mod model;
pub mod manager;
pub mod store;

pub use model::{Task, TaskStatus, TaskResult};
pub use manager::TaskManager;
pub use store::TaskStore;
