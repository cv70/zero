use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use serde_json::Value;
use zero_core::runtime::{DispatchEvent, TaskState};

#[derive(Debug, Deserialize)]
pub struct CreateTaskRequest {
    pub title: String,
    pub description: String,
    #[serde(default)]
    pub dependencies: Vec<String>,
}

#[derive(Debug, Deserialize)]
pub struct CreateSessionRequest {
    pub title: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct PatchSessionRequest {
    pub title: String,
}

#[derive(Debug, Deserialize)]
pub struct AttachTaskRequest {
    pub task_id: String,
}

#[derive(Debug, Deserialize)]
pub struct PatchTaskRequest {
    pub title: Option<String>,
    pub description: Option<String>,
    pub metadata: Option<HashMap<String, String>>,
}

#[derive(Debug, Deserialize)]
pub struct ListTaskQuery {
    pub status: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct RecoverDecideRequest {
    pub task_id: String,
    pub failure_class: String,
    pub attempt: u8,
}

#[derive(Debug, Deserialize)]
pub struct VerifyOutcomeRequest {
    pub task_id: String,
    pub outcome: String,
}

#[derive(Debug, Serialize)]
pub struct IdResponse {
    pub id: String,
}

#[derive(Debug, Serialize)]
pub struct HealthResponse {
    pub status: &'static str,
}

#[derive(Debug, Serialize)]
pub struct TaskStateResponse {
    pub task_id: String,
    pub state: TaskState,
}

#[derive(Debug, Serialize)]
pub struct RecoveryDecisionResponse {
    pub task_id: String,
    pub decision: String,
}

#[derive(Debug, Serialize)]
pub struct DispatchResponse {
    pub task_id: String,
    pub step_id: String,
    pub output: String,
    pub from_cache: bool,
    pub event: DispatchEvent,
}

#[derive(Debug, Serialize)]
pub struct RuntimeMetricsResponse {
    pub tasks_per_min: f64,
    pub task_success_rate: f64,
    pub token_per_task: f64,
    pub p50_latency_ms: f64,
    pub p95_latency_ms: f64,
    pub p99_latency_ms: f64,
    pub total_tasks: usize,
    pub planned_tasks: usize,
    pub state_counts: HashMap<String, usize>,
}

#[derive(Debug, Serialize)]
pub struct ApiErrorBody {
    pub error: String,
    pub code: &'static str,
    pub details: Value,
}

#[derive(Debug)]
pub struct ApiError {
    pub status: axum::http::StatusCode,
    pub code: &'static str,
    pub message: String,
    pub details: Value,
}
