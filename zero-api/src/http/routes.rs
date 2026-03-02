use axum::{
    Router,
    routing::{delete, get, post},
};

use crate::state::app_state::AppState;

use super::handlers::{
    attach_task_to_session, complete_step, create_session, create_task, delete_session,
    delete_task, detach_task_from_session, dispatch_next_step, fail_step, get_plan,
    get_runtime_metrics, get_session, get_task, get_task_state, healthz, list_sessions, list_tasks,
    patch_session, patch_task, recover_decide, submit_plan, timeout_step, verify_outcome,
};

pub fn app(state: AppState) -> Router {
    Router::new()
        .route("/healthz", get(healthz))
        .route("/metrics/runtime", get(get_runtime_metrics))
        .route("/tasks", post(create_task).get(list_tasks))
        .route(
            "/tasks/{id}",
            get(get_task).patch(patch_task).delete(delete_task),
        )
        .route("/tasks/{id}/plan", post(submit_plan).get(get_plan))
        .route("/tasks/{id}/dispatch", post(dispatch_next_step))
        .route("/tasks/{id}/steps/{step_id}/completed", post(complete_step))
        .route("/tasks/{id}/steps/{step_id}/failed", post(fail_step))
        .route("/tasks/{id}/steps/{step_id}/timeout", post(timeout_step))
        .route("/tasks/{id}/state", get(get_task_state))
        .route("/tasks/{id}/recover/decide", post(recover_decide))
        .route("/tasks/{id}/verify/outcome", post(verify_outcome))
        .route("/sessions", post(create_session).get(list_sessions))
        .route(
            "/sessions/{id}",
            get(get_session).patch(patch_session).delete(delete_session),
        )
        .route("/sessions/{id}/tasks", post(attach_task_to_session))
        .route(
            "/sessions/{id}/tasks/{task_id}",
            delete(detach_task_from_session),
        )
        .with_state(state)
}
