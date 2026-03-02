# Zero API Design (Task + Plan Hybrid)

## Goal
Implement `zero-api` as a task execution/scheduling HTTP service backed by `zero-core`, combining task metadata management and runtime step-state progression.

## Scope
- In scope: in-memory CRUD for tasks, plan submission, step completion callback, runtime state query.
- Out of scope: persistent storage, auth, distributed locks, retries/workers.

## API
- `POST /tasks`
- `GET /tasks`
- `GET /tasks/{id}`
- `DELETE /tasks/{id}`
- `POST /tasks/{id}/plan`
- `POST /tasks/{id}/steps/{step_id}/completed`
- `GET /tasks/{id}/state`

## Data Model Integration
- Task metadata: `zero_core::task::{Task, InMemoryTaskManager}`
- Runtime progression: `zero_core::runtime::{ExecutionPlan, ControlPlane, TaskState}`

## State Rules
- Task create => `TaskStatus::Pending`
- Plan accepted => runtime `TaskState::Running`
- Final step completed => runtime `TaskState::Succeeded`
- Missing task/step => `404`
- Duplicate plan submission => `409`

## Error Contract
JSON:
```json
{ "error": "...", "code": "..." }
```
Codes: `bad_request`, `not_found`, `conflict`, `internal`

## Testing Strategy
- Router-level tests with `tower::ServiceExt`
- Cover happy path: create task -> submit plan -> complete steps -> succeeded
- Cover errors: task not found, duplicate plan, task_id mismatch
