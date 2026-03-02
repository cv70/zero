use std::collections::HashMap;

use axum::{
    Json,
    extract::{Path, Query, State},
    http::StatusCode,
    response::{IntoResponse, Response},
};
use serde_json::Value;
use uuid::Uuid;
use zero_core::runtime::{DispatchEvent, ExecutionPlan, TaskState};
use zero_core::task::model::{Task, TaskStatus};

use crate::domain::error::DomainError;
use crate::domain::models::{parse_failure_class, parse_task_status, parse_verify_outcome};
use crate::domain::service::{ensure_dispatchable, ensure_path_matches, ensure_plan_path_matches};
use crate::http::dto::{
    ApiError, ApiErrorBody, AttachTaskRequest, CreateSessionRequest, CreateTaskRequest,
    DispatchResponse, HealthResponse, IdResponse, ListTaskQuery, PatchSessionRequest,
    PatchTaskRequest, RecoverDecideRequest, RecoveryDecisionResponse, RuntimeMetricsResponse,
    TaskStateResponse, VerifyOutcomeRequest,
};
use crate::state::app_state::AppState;
use crate::state::session_store::Session;

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        (
            self.status,
            Json(ApiErrorBody {
                error: self.message,
                code: self.code,
                details: self.details,
            }),
        )
            .into_response()
    }
}

fn to_api_error(err: DomainError) -> ApiError {
    match err {
        DomainError::BadRequest(msg) => ApiError {
            status: StatusCode::BAD_REQUEST,
            code: "bad_request",
            message: msg,
            details: Value::Null,
        },
        DomainError::NotFound(msg) => ApiError {
            status: StatusCode::NOT_FOUND,
            code: "not_found",
            message: msg,
            details: Value::Null,
        },
        DomainError::Conflict(msg) => ApiError {
            status: StatusCode::CONFLICT,
            code: "conflict",
            message: msg,
            details: Value::Null,
        },
        DomainError::InvalidState(msg) => ApiError {
            status: StatusCode::CONFLICT,
            code: "invalid_state",
            message: msg,
            details: Value::Null,
        },
        DomainError::Internal(msg) => ApiError {
            status: StatusCode::INTERNAL_SERVER_ERROR,
            code: "internal",
            message: msg,
            details: Value::Null,
        },
    }
}

fn map_runtime_error(e: zero_core::ZeroError) -> ApiError {
    let msg = e.to_string();
    if msg.contains("not found") {
        to_api_error(DomainError::NotFound(msg))
    } else {
        to_api_error(DomainError::Internal(msg))
    }
}

async fn ensure_task_exists(state: &AppState, id: &str) -> Result<(), ApiError> {
    let task = state
        .task_manager
        .get(id)
        .await
        .map_err(|e| to_api_error(DomainError::Internal(e.to_string())))?;
    if task.is_none() {
        return Err(to_api_error(DomainError::NotFound(format!(
            "task not found: {id}"
        ))));
    }
    Ok(())
}

pub async fn healthz() -> impl IntoResponse {
    Json(HealthResponse { status: "ok" })
}

pub async fn create_task(
    State(state): State<AppState>,
    Json(req): Json<CreateTaskRequest>,
) -> Result<impl IntoResponse, ApiError> {
    if req.title.trim().is_empty() {
        return Err(to_api_error(DomainError::BadRequest(
            "title cannot be empty".to_string(),
        )));
    }

    let id = Uuid::new_v4().to_string();
    let task = req.dependencies.into_iter().fold(
        Task::new(id.clone(), req.title, req.description),
        |acc, dep| acc.with_dependency(dep),
    );

    state
        .task_manager
        .create(task)
        .await
        .map_err(|e| to_api_error(DomainError::Internal(e.to_string())))?;
    state.metrics.lock().await.record_task_started();

    Ok((StatusCode::CREATED, Json(IdResponse { id })))
}

pub async fn create_session(
    State(state): State<AppState>,
    Json(req): Json<CreateSessionRequest>,
) -> Result<impl IntoResponse, ApiError> {
    let session = state
        .session_store
        .create(req.title)
        .await
        .map_err(|e| to_api_error(DomainError::Internal(e.to_string())))?;
    Ok((StatusCode::CREATED, Json(session)))
}

pub async fn list_sessions(State(state): State<AppState>) -> Result<impl IntoResponse, ApiError> {
    let sessions = state
        .session_store
        .list()
        .await
        .map_err(|e| to_api_error(DomainError::Internal(e.to_string())))?;
    Ok(Json(sessions))
}

pub async fn get_session(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<impl IntoResponse, ApiError> {
    let session = state
        .session_store
        .get(&id)
        .await
        .map_err(|e| to_api_error(DomainError::Internal(e.to_string())))?
        .ok_or_else(|| to_api_error(DomainError::NotFound(format!("session not found: {id}"))))?;
    Ok(Json(session))
}

pub async fn patch_session(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Json(req): Json<PatchSessionRequest>,
) -> Result<impl IntoResponse, ApiError> {
    let title = req.title.trim().to_string();
    if title.is_empty() {
        return Err(to_api_error(DomainError::BadRequest(
            "title cannot be empty".to_string(),
        )));
    }
    let session = state
        .session_store
        .update_title(&id, title)
        .await
        .map_err(|e| to_api_error(DomainError::Internal(e.to_string())))?
        .ok_or_else(|| to_api_error(DomainError::NotFound(format!("session not found: {id}"))))?;
    Ok(Json(session))
}

pub async fn delete_session(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<impl IntoResponse, ApiError> {
    state
        .session_store
        .delete(&id)
        .await
        .map_err(|e| to_api_error(DomainError::Internal(e.to_string())))?;
    Ok(StatusCode::NO_CONTENT)
}

pub async fn attach_task_to_session(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Json(req): Json<AttachTaskRequest>,
) -> Result<impl IntoResponse, ApiError> {
    ensure_task_exists(&state, &req.task_id).await?;
    let session = state
        .session_store
        .add_task(&id, req.task_id)
        .await
        .map_err(|e| to_api_error(DomainError::Internal(e.to_string())))?
        .ok_or_else(|| to_api_error(DomainError::NotFound(format!("session not found: {id}"))))?;
    Ok(Json(session))
}

pub async fn detach_task_from_session(
    State(state): State<AppState>,
    Path((id, task_id)): Path<(String, String)>,
) -> Result<impl IntoResponse, ApiError> {
    let session: Session = state
        .session_store
        .remove_task(&id, &task_id)
        .await
        .map_err(|e| to_api_error(DomainError::Internal(e.to_string())))?
        .ok_or_else(|| to_api_error(DomainError::NotFound(format!("session not found: {id}"))))?;
    Ok(Json(session))
}

pub async fn list_tasks(
    State(state): State<AppState>,
    Query(query): Query<ListTaskQuery>,
) -> Result<impl IntoResponse, ApiError> {
    let mut tasks = state
        .task_manager
        .list()
        .await
        .map_err(|e| to_api_error(DomainError::Internal(e.to_string())))?;

    if let Some(filter) = query.status {
        let status = parse_task_status(&filter).map_err(to_api_error)?;
        tasks.retain(|t| t.status == status);
    }

    Ok(Json(tasks))
}

pub async fn get_task(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<impl IntoResponse, ApiError> {
    let task = state
        .task_manager
        .get(&id)
        .await
        .map_err(|e| to_api_error(DomainError::Internal(e.to_string())))?;

    match task {
        Some(task) => Ok(Json(task).into_response()),
        None => Err(to_api_error(DomainError::NotFound(format!(
            "task not found: {id}"
        )))),
    }
}

pub async fn patch_task(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Json(req): Json<PatchTaskRequest>,
) -> Result<impl IntoResponse, ApiError> {
    let mut task = state
        .task_manager
        .get(&id)
        .await
        .map_err(|e| to_api_error(DomainError::Internal(e.to_string())))?
        .ok_or_else(|| to_api_error(DomainError::NotFound(format!("task not found: {id}"))))?;

    if let Some(title) = req.title {
        if title.trim().is_empty() {
            return Err(to_api_error(DomainError::BadRequest(
                "title cannot be empty".to_string(),
            )));
        }
        task.title = title;
    }
    if let Some(description) = req.description {
        task.description = description;
    }
    if let Some(metadata) = req.metadata {
        task.metadata = metadata;
    }

    state
        .task_manager
        .create(task.clone())
        .await
        .map_err(|e| to_api_error(DomainError::Internal(e.to_string())))?;

    Ok(Json(task))
}

pub async fn delete_task(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<impl IntoResponse, ApiError> {
    state
        .task_manager
        .delete(&id)
        .await
        .map_err(|e| to_api_error(DomainError::Internal(e.to_string())))?;
    state.planned_tasks.lock().await.remove(&id);
    state.plans.lock().await.remove(&id);
    state.step_progress.lock().await.remove(&id);
    state.state_overrides.lock().await.remove(&id);
    Ok(StatusCode::NO_CONTENT)
}

pub async fn submit_plan(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Json(plan): Json<ExecutionPlan>,
) -> Result<impl IntoResponse, ApiError> {
    ensure_plan_path_matches(&id, &plan.task_id).map_err(to_api_error)?;
    ensure_task_exists(&state, &id).await?;

    let mut planned = state.planned_tasks.lock().await;
    if planned.contains(&id) {
        return Err(to_api_error(DomainError::Conflict(format!(
            "plan already exists for task: {id}"
        ))));
    }

    state
        .control_plane
        .lock()
        .await
        .accept_plan(plan.clone())
        .await
        .map_err(map_runtime_error)?;

    state.plans.lock().await.insert(id.clone(), plan);
    state.step_progress.lock().await.insert(id.clone(), 0);
    planned.insert(id.clone());

    state
        .task_manager
        .update_status(&id, TaskStatus::Running)
        .await
        .map_err(|e| to_api_error(DomainError::Internal(e.to_string())))?;

    Ok((StatusCode::CREATED, Json(IdResponse { id })))
}

pub async fn get_plan(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<impl IntoResponse, ApiError> {
    let plan = state.plans.lock().await.get(&id).cloned().ok_or_else(|| {
        to_api_error(DomainError::NotFound(format!(
            "plan not found for task: {id}"
        )))
    })?;
    Ok(Json(plan))
}

pub async fn dispatch_next_step(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<impl IntoResponse, ApiError> {
    let plan = state.plans.lock().await.get(&id).cloned().ok_or_else(|| {
        to_api_error(DomainError::NotFound(format!(
            "plan not found for task: {id}"
        )))
    })?;

    let mut progress = state.step_progress.lock().await;
    let idx = *progress.get(&id).unwrap_or(&0);
    ensure_dispatchable(idx, plan.steps.len(), &id).map_err(to_api_error)?;

    let step = plan.steps[idx].clone();
    let event =
        DispatchEvent::step_dispatched(step.task_id.clone(), step.step_id.clone(), step.op.clone());
    let result = state
        .data_plane
        .execute_step(step)
        .await
        .map_err(|e| to_api_error(DomainError::Internal(e.to_string())))?;

    progress.insert(id.clone(), idx + 1);

    Ok(Json(DispatchResponse {
        task_id: id,
        step_id: result.step_id,
        output: result.output,
        from_cache: result.from_cache,
        event,
    }))
}

pub async fn complete_step(
    State(state): State<AppState>,
    Path((id, step_id)): Path<(String, String)>,
) -> Result<impl IntoResponse, ApiError> {
    ensure_task_exists(&state, &id).await?;

    state
        .control_plane
        .lock()
        .await
        .on_step_completed(&id, &step_id)
        .await
        .map_err(map_runtime_error)?;

    let state_value = state
        .control_plane
        .lock()
        .await
        .task_state(&id)
        .await
        .map_err(map_runtime_error)?;

    if state_value == TaskState::Succeeded {
        state
            .task_manager
            .complete(&id)
            .await
            .map_err(|e| to_api_error(DomainError::Internal(e.to_string())))?;
        state.metrics.lock().await.record_task_succeeded();
    }

    Ok(StatusCode::OK)
}

pub async fn fail_step(
    State(state): State<AppState>,
    Path((id, _step_id)): Path<(String, String)>,
) -> Result<impl IntoResponse, ApiError> {
    ensure_task_exists(&state, &id).await?;
    state
        .task_manager
        .fail(&id)
        .await
        .map_err(|e| to_api_error(DomainError::Internal(e.to_string())))?;
    state
        .state_overrides
        .lock()
        .await
        .insert(id, TaskState::Failed);
    Ok(StatusCode::OK)
}

pub async fn timeout_step(
    State(state): State<AppState>,
    Path((id, _step_id)): Path<(String, String)>,
) -> Result<impl IntoResponse, ApiError> {
    ensure_task_exists(&state, &id).await?;
    state
        .state_overrides
        .lock()
        .await
        .insert(id, TaskState::Waiting);
    Ok(StatusCode::OK)
}

pub async fn get_task_state(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<impl IntoResponse, ApiError> {
    if let Some(overridden) = state.state_overrides.lock().await.get(&id).cloned() {
        return Ok(Json(TaskStateResponse {
            task_id: id,
            state: overridden,
        }));
    }

    let state_value = state
        .control_plane
        .lock()
        .await
        .task_state(&id)
        .await
        .map_err(map_runtime_error)?;

    Ok(Json(TaskStateResponse {
        task_id: id,
        state: state_value,
    }))
}

pub async fn recover_decide(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Json(req): Json<RecoverDecideRequest>,
) -> Result<impl IntoResponse, ApiError> {
    ensure_path_matches(&id, &req.task_id).map_err(to_api_error)?;
    ensure_task_exists(&state, &id).await?;

    let class = parse_failure_class(&req.failure_class).map_err(to_api_error)?;
    let decision = state
        .control_plane
        .lock()
        .await
        .decide_recovery(class, req.attempt);

    Ok(Json(RecoveryDecisionResponse {
        task_id: id,
        decision: format!("{decision:?}"),
    }))
}

pub async fn verify_outcome(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Json(req): Json<VerifyOutcomeRequest>,
) -> Result<impl IntoResponse, ApiError> {
    ensure_path_matches(&id, &req.task_id).map_err(to_api_error)?;
    ensure_task_exists(&state, &id).await?;

    let outcome = parse_verify_outcome(&req.outcome).map_err(to_api_error)?;
    let decision = state
        .control_plane
        .lock()
        .await
        .handle_verification_outcome(outcome);

    Ok(Json(RecoveryDecisionResponse {
        task_id: id,
        decision: format!("{decision:?}"),
    }))
}

pub async fn get_runtime_metrics(
    State(state): State<AppState>,
) -> Result<impl IntoResponse, ApiError> {
    let runtime = state.metrics.lock().await.snapshot();
    let tasks = state
        .task_manager
        .list()
        .await
        .map_err(|e| to_api_error(DomainError::Internal(e.to_string())))?;
    let planned_tasks = state.planned_tasks.lock().await.len();

    let mut counts: HashMap<String, usize> = HashMap::new();
    for t in &tasks {
        let k = match t.status {
            TaskStatus::Pending => "pending",
            TaskStatus::Running => "running",
            TaskStatus::Completed => "completed",
            TaskStatus::Failed => "failed",
        }
        .to_string();
        *counts.entry(k).or_insert(0) += 1;
    }

    Ok(Json(RuntimeMetricsResponse {
        tasks_per_min: runtime.tasks_per_min,
        task_success_rate: runtime.task_success_rate,
        token_per_task: runtime.token_per_task,
        p50_latency_ms: runtime.p50_latency_ms,
        p95_latency_ms: runtime.p95_latency_ms,
        p99_latency_ms: runtime.p99_latency_ms,
        total_tasks: tasks.len(),
        planned_tasks,
        state_counts: counts,
    }))
}

#[cfg(test)]
mod tests {
    use axum::body::{Body, to_bytes};
    use axum::http::{Request, StatusCode};
    use serde_json::{Value, json};
    use tower::util::ServiceExt;

    use crate::http::routes::app;
    use crate::state::app_state::build_state;

    #[tokio::test]
    async fn healthz_and_metrics_should_work() {
        let app = app(build_state());

        let h = Request::builder()
            .method("GET")
            .uri("/healthz")
            .body(Body::empty())
            .unwrap();
        let h_resp = app.clone().oneshot(h).await.unwrap();
        assert_eq!(h_resp.status(), StatusCode::OK);

        let m = Request::builder()
            .method("GET")
            .uri("/metrics/runtime")
            .body(Body::empty())
            .unwrap();
        let m_resp = app.clone().oneshot(m).await.unwrap();
        assert_eq!(m_resp.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn patch_and_filter_tasks_should_work() {
        let app = app(build_state());
        let create = Request::builder()
            .method("POST")
            .uri("/tasks")
            .header("content-type", "application/json")
            .body(Body::from(
                json!({ "title":"old", "description":"d", "dependencies":[] }).to_string(),
            ))
            .unwrap();
        let create_resp = app.clone().oneshot(create).await.unwrap();
        assert_eq!(create_resp.status(), StatusCode::CREATED);
        let create_body = to_bytes(create_resp.into_body(), usize::MAX).await.unwrap();
        let created: Value = serde_json::from_slice(&create_body).unwrap();
        let task_id = created["id"].as_str().unwrap();

        let patch = Request::builder()
            .method("PATCH")
            .uri(format!("/tasks/{task_id}"))
            .header("content-type", "application/json")
            .body(Body::from(json!({"title":"new"}).to_string()))
            .unwrap();
        assert_eq!(
            app.clone().oneshot(patch).await.unwrap().status(),
            StatusCode::OK
        );

        let filtered = Request::builder()
            .method("GET")
            .uri("/tasks?status=pending")
            .body(Body::empty())
            .unwrap();
        let filtered_resp = app.clone().oneshot(filtered).await.unwrap();
        assert_eq!(filtered_resp.status(), StatusCode::OK);
        let body = to_bytes(filtered_resp.into_body(), usize::MAX)
            .await
            .unwrap();
        let v: Value = serde_json::from_slice(&body).unwrap();
        assert_eq!(v.as_array().unwrap().len(), 1);
    }

    #[tokio::test]
    async fn dispatch_and_callbacks_should_cover_full_flow() {
        let app = app(build_state());

        let create = Request::builder()
            .method("POST")
            .uri("/tasks")
            .header("content-type", "application/json")
            .body(Body::from(
                json!({ "title":"task-flow", "description":"desc", "dependencies":[] }).to_string(),
            ))
            .unwrap();
        let create_resp = app.clone().oneshot(create).await.unwrap();
        assert_eq!(create_resp.status(), StatusCode::CREATED);
        let create_body = to_bytes(create_resp.into_body(), usize::MAX).await.unwrap();
        let created: Value = serde_json::from_slice(&create_body).unwrap();
        let task_id = created["id"].as_str().unwrap();

        let plan = json!({
            "task_id": task_id,
            "steps": [
                { "task_id":task_id, "step_id":"s1", "op":"agent.execute", "idempotency_key":"i1" },
                { "task_id":task_id, "step_id":"s2", "op":"tool.execute", "idempotency_key":"i2" }
            ]
        });
        let submit = Request::builder()
            .method("POST")
            .uri(format!("/tasks/{task_id}/plan"))
            .header("content-type", "application/json")
            .body(Body::from(plan.to_string()))
            .unwrap();
        assert_eq!(
            app.clone().oneshot(submit).await.unwrap().status(),
            StatusCode::CREATED
        );

        let get_plan = Request::builder()
            .method("GET")
            .uri(format!("/tasks/{task_id}/plan"))
            .body(Body::empty())
            .unwrap();
        assert_eq!(
            app.clone().oneshot(get_plan).await.unwrap().status(),
            StatusCode::OK
        );

        let dispatch = Request::builder()
            .method("POST")
            .uri(format!("/tasks/{task_id}/dispatch"))
            .body(Body::empty())
            .unwrap();
        assert_eq!(
            app.clone().oneshot(dispatch).await.unwrap().status(),
            StatusCode::OK
        );

        let complete_s1 = Request::builder()
            .method("POST")
            .uri(format!("/tasks/{task_id}/steps/s1/completed"))
            .body(Body::empty())
            .unwrap();
        assert_eq!(
            app.clone().oneshot(complete_s1).await.unwrap().status(),
            StatusCode::OK
        );

        let complete_s2 = Request::builder()
            .method("POST")
            .uri(format!("/tasks/{task_id}/steps/s2/completed"))
            .body(Body::empty())
            .unwrap();
        assert_eq!(
            app.clone().oneshot(complete_s2).await.unwrap().status(),
            StatusCode::OK
        );

        let state_req = Request::builder()
            .method("GET")
            .uri(format!("/tasks/{task_id}/state"))
            .body(Body::empty())
            .unwrap();
        let state_resp = app.clone().oneshot(state_req).await.unwrap();
        assert_eq!(state_resp.status(), StatusCode::OK);
        let body = to_bytes(state_resp.into_body(), usize::MAX).await.unwrap();
        let v: Value = serde_json::from_slice(&body).unwrap();
        assert_eq!(v["state"], "Succeeded");
    }

    #[tokio::test]
    async fn failed_timeout_and_recovery_apis_should_work() {
        let app = app(build_state());

        let create = Request::builder()
            .method("POST")
            .uri("/tasks")
            .header("content-type", "application/json")
            .body(Body::from(
                json!({ "title":"task-err", "description":"desc", "dependencies":[] }).to_string(),
            ))
            .unwrap();
        let create_resp = app.clone().oneshot(create).await.unwrap();
        assert_eq!(create_resp.status(), StatusCode::CREATED);
        let create_body = to_bytes(create_resp.into_body(), usize::MAX).await.unwrap();
        let created: Value = serde_json::from_slice(&create_body).unwrap();
        let task_id = created["id"].as_str().unwrap();

        let failed = Request::builder()
            .method("POST")
            .uri(format!("/tasks/{task_id}/steps/s1/failed"))
            .body(Body::empty())
            .unwrap();
        assert_eq!(
            app.clone().oneshot(failed).await.unwrap().status(),
            StatusCode::OK
        );

        let timeout = Request::builder()
            .method("POST")
            .uri(format!("/tasks/{task_id}/steps/s2/timeout"))
            .body(Body::empty())
            .unwrap();
        assert_eq!(
            app.clone().oneshot(timeout).await.unwrap().status(),
            StatusCode::OK
        );

        let recover = Request::builder()
            .method("POST")
            .uri(format!("/tasks/{task_id}/recover/decide"))
            .header("content-type", "application/json")
            .body(Body::from(
                json!({"task_id":task_id,"failure_class":"provider_timeout","attempt":1})
                    .to_string(),
            ))
            .unwrap();
        let recover_resp = app.clone().oneshot(recover).await.unwrap();
        assert_eq!(recover_resp.status(), StatusCode::OK);

        let verify = Request::builder()
            .method("POST")
            .uri(format!("/tasks/{task_id}/verify/outcome"))
            .header("content-type", "application/json")
            .body(Body::from(
                json!({"task_id":task_id,"outcome":"hard_fail"}).to_string(),
            ))
            .unwrap();
        let verify_resp = app.clone().oneshot(verify).await.unwrap();
        assert_eq!(verify_resp.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn session_lifecycle_should_work() {
        let app = app(build_state());

        let create = Request::builder()
            .method("POST")
            .uri("/sessions")
            .header("content-type", "application/json")
            .body(Body::from(json!({"title":"s1"}).to_string()))
            .unwrap();
        let create_resp = app.clone().oneshot(create).await.unwrap();
        assert_eq!(create_resp.status(), StatusCode::CREATED);
        let create_body = to_bytes(create_resp.into_body(), usize::MAX).await.unwrap();
        let created: Value = serde_json::from_slice(&create_body).unwrap();
        let session_id = created["id"].as_str().unwrap();

        let task_create = Request::builder()
            .method("POST")
            .uri("/tasks")
            .header("content-type", "application/json")
            .body(Body::from(
                json!({ "title":"session-task", "description":"desc", "dependencies":[] })
                    .to_string(),
            ))
            .unwrap();
        let task_resp = app.clone().oneshot(task_create).await.unwrap();
        assert_eq!(task_resp.status(), StatusCode::CREATED);
        let task_body = to_bytes(task_resp.into_body(), usize::MAX).await.unwrap();
        let task_created: Value = serde_json::from_slice(&task_body).unwrap();
        let task_id = task_created["id"].as_str().unwrap();

        let attach = Request::builder()
            .method("POST")
            .uri(format!("/sessions/{session_id}/tasks"))
            .header("content-type", "application/json")
            .body(Body::from(json!({"task_id": task_id}).to_string()))
            .unwrap();
        assert_eq!(
            app.clone().oneshot(attach).await.unwrap().status(),
            StatusCode::OK
        );

        let list = Request::builder()
            .method("GET")
            .uri("/sessions")
            .body(Body::empty())
            .unwrap();
        let list_resp = app.clone().oneshot(list).await.unwrap();
        assert_eq!(list_resp.status(), StatusCode::OK);

        let detach = Request::builder()
            .method("DELETE")
            .uri(format!("/sessions/{session_id}/tasks/{task_id}"))
            .body(Body::empty())
            .unwrap();
        assert_eq!(
            app.clone().oneshot(detach).await.unwrap().status(),
            StatusCode::OK
        );

        let delete = Request::builder()
            .method("DELETE")
            .uri(format!("/sessions/{session_id}"))
            .body(Body::empty())
            .unwrap();
        assert_eq!(
            app.clone().oneshot(delete).await.unwrap().status(),
            StatusCode::NO_CONTENT
        );
    }
}
