# Zero API Agent System Design

## Goal
Build a complete single-node in-memory Agent orchestration API in `zero-api`, with all core task lifecycle and runtime orchestration endpoints backed by `zero-core`.

## Scope
- In scope: task CRUD/update/filter, plan submit/get, dispatch next steps, step result callbacks (completed/failed/timeout), runtime state query, recovery decision endpoints, health/runtime metrics endpoints.
- Out of scope: persistence, auth, distributed scheduling, worker pool, external queue.

## Architecture
`zero-api` exposes a single Axum router and stores runtime state in process:
- Task metadata via `zero_core::task::manager::InMemoryTaskManager`
- Plan/state transitions via `zero_core::runtime::ControlPlane`
- Step idempotent execution for dispatch simulation via `zero_core::runtime::DataPlane`
- Recovery/verification decisions via `ControlPlane::{decide_recovery, handle_verification_outcome}`
- Lightweight in-process indexes for plan existence and step progress

## API Surface
- `GET /healthz`
- `GET /metrics/runtime`
- `POST /tasks`
- `GET /tasks?status=<pending|running|completed|failed>`
- `GET /tasks/{id}`
- `PATCH /tasks/{id}`
- `DELETE /tasks/{id}`
- `POST /tasks/{id}/plan`
- `GET /tasks/{id}/plan`
- `POST /tasks/{id}/dispatch`
- `POST /tasks/{id}/steps/{step_id}/completed`
- `POST /tasks/{id}/steps/{step_id}/failed`
- `POST /tasks/{id}/steps/{step_id}/timeout`
- `GET /tasks/{id}/state`
- `POST /tasks/{id}/recover/decide`
- `POST /tasks/{id}/verify/outcome`

## Data Flow
1. Create task (`pending`)
2. Submit plan (runtime state -> `running`)
3. Dispatch executes next step via `DataPlane` and returns dispatch info
4. Step callbacks update runtime state and task status
5. Failure/verification endpoints provide recovery decision outputs

## Error Contract
JSON:
```json
{ "error": "...", "code": "...", "details": { ... } }
```
Codes: `bad_request`, `not_found`, `conflict`, `invalid_state`, `internal`

## Consistency Rules
- Path `id` must match payload `task_id` where applicable.
- Duplicate plan submission for same task -> `409`.
- Missing task/plan/step -> `404`.
- Deleting task clears task metadata + runtime indexes.

## Testing Strategy
Router-level tests (`tower::ServiceExt`) cover:
- Full happy path (create -> plan -> dispatch -> complete -> succeeded)
- Task patch/list filtering
- Plan get + duplicate submission conflict
- Failed/timeout callback and recovery decision APIs
- Validation and not-found error contracts
