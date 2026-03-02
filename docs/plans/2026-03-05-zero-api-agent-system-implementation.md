# Zero API Agent System Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Deliver a complete in-memory agent orchestration API in `zero-api` backed by `zero-core` runtime/task primitives.

**Architecture:** Extend single-file `zero-api/src/main.rs` with additional DTOs, routes, and orchestration state in `AppState` while delegating lifecycle and recovery semantics to `zero-core` `TaskManager`, `ControlPlane`, and `DataPlane`.

**Tech Stack:** Rust, tokio, axum, tower, serde, zero-core

---

### Task 1: Add failing tests for complete agent API

**Files:**
- Modify: `zero-api/src/main.rs` (test module)

1. Add tests for new endpoints/behaviors:
- `GET /healthz`
- `GET /metrics/runtime`
- `PATCH /tasks/{id}`
- `GET /tasks?status=...`
- `GET /tasks/{id}/plan`
- `POST /tasks/{id}/dispatch`
- `POST /tasks/{id}/steps/{step_id}/failed`
- `POST /tasks/{id}/steps/{step_id}/timeout`
- `POST /tasks/{id}/recover/decide`
- `POST /tasks/{id}/verify/outcome`

2. Run failing tests:
- Run: `cargo test -p zero-api`
- Expected: failures due to missing routes/handlers.

### Task 2: Implement route contracts and state plumbing

**Files:**
- Modify: `zero-api/src/main.rs`

1. Expand `AppState` indexes (plans, step order/progress, metrics collector).
2. Add request/response DTOs for patch, dispatch, recovery, verification, health/metrics.
3. Register all missing routes.
4. Implement handlers with validation and standardized error mapping.

### Task 3: Wire runtime semantics through zero-core

**Files:**
- Modify: `zero-api/src/main.rs`

1. Plan submission stores plan and initializes progress.
2. Dispatch returns next step dispatch event and executes via `DataPlane`.
3. Completed/failed/timeout callbacks update `ControlPlane` and task status.
4. Recovery and verification endpoints call `ControlPlane` decision methods.
5. Runtime metrics endpoint returns snapshot + task state counters.

### Task 4: Verify and stabilize

**Files:**
- Modify: `zero-api/src/main.rs` (if needed)

1. Run `cargo fmt`.
2. Run `cargo test -p zero-api`.
3. Run `cargo check -p zero-api`.
4. Fix any regressions and rerun until green.
