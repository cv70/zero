use std::collections::{HashMap, HashSet};
use std::sync::Arc;

use tokio::sync::Mutex;
use zero_core::runtime::{
    ControlPlane, DataPlane, ExecutionPlan, RuntimeMetricsCollector, TaskState,
};
use zero_core::task::manager::{InMemoryTaskManager, SqliteTaskManager, TaskManager};

use super::session_store::{SessionStore, SqliteSessionStore};

#[derive(Clone)]
pub struct AppState {
    pub task_manager: Arc<dyn TaskManager>,
    pub session_store: Arc<dyn SessionStore>,
    pub control_plane: Arc<Mutex<ControlPlane>>,
    pub data_plane: Arc<DataPlane>,
    pub planned_tasks: Arc<Mutex<HashSet<String>>>,
    pub plans: Arc<Mutex<HashMap<String, ExecutionPlan>>>,
    pub step_progress: Arc<Mutex<HashMap<String, usize>>>,
    pub state_overrides: Arc<Mutex<HashMap<String, TaskState>>>,
    pub metrics: Arc<Mutex<RuntimeMetricsCollector>>,
}

pub fn build_state() -> AppState {
    let sqlite_path = std::env::var("ZERO_API_SQLITE_PATH")
        .ok()
        .filter(|v| !v.trim().is_empty())
        .map(|v| v.trim().to_string())
        .unwrap_or_else(|| "zero_api.db".to_string());

    let task_manager: Arc<dyn TaskManager> = match SqliteTaskManager::new(&sqlite_path) {
        Ok(mgr) => Arc::new(mgr),
        Err(err) => {
            eprintln!("sqlite init failed, fallback to in-memory task manager: {err}");
            Arc::new(InMemoryTaskManager::new())
        }
    };

    let session_store: Arc<dyn SessionStore> = Arc::new(SqliteSessionStore::new(&sqlite_path));

    AppState {
        task_manager,
        session_store,
        control_plane: Arc::new(Mutex::new(ControlPlane::new_in_memory())),
        data_plane: Arc::new(DataPlane::new_for_test()),
        planned_tasks: Arc::new(Mutex::new(HashSet::new())),
        plans: Arc::new(Mutex::new(HashMap::new())),
        step_progress: Arc::new(Mutex::new(HashMap::new())),
        state_overrides: Arc::new(Mutex::new(HashMap::new())),
        metrics: Arc::new(Mutex::new(RuntimeMetricsCollector::default())),
    }
}
