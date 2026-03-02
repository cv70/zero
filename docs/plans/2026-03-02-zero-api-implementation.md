# Zero API Hybrid Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Build `zero-api` hybrid endpoints for task metadata + runtime scheduling progression.

**Architecture:** Use an `AppState` with `InMemoryTaskManager` and `tokio::sync::Mutex<ControlPlane>`. Expose axum routes for task CRUD + plan/step lifecycle. Track plan existence in-memory to prevent duplicate submission.

**Tech Stack:** Rust, tokio, axum, tower, serde, zero-core

---

### Task 1: Create failing API tests

**Files:**
- Modify: `zero-api/src/main.rs`

**Step 1: Write failing test**
- Add tests for:
  - create/get/list/delete task
  - submit plan and duplicate submission returns 409
  - complete step transitions task state to succeeded

**Step 2: Run tests to verify failure**
- Run: `cargo test -p zero-api`
- Expected: FAIL due to missing handlers/state/routes

### Task 2: Implement routes and handlers minimally

**Files:**
- Modify: `zero-api/src/main.rs`

**Step 1: Implement AppState and DTOs**

**Step 2: Implement routes/handlers**
- `/tasks` create/list/get/delete
- `/tasks/:id/plan`
- `/tasks/:id/steps/:step_id/completed`
- `/tasks/:id/state`

**Step 3: Run tests to verify pass**
- Run: `cargo test -p zero-api`
- Expected: PASS

### Task 3: Validate formatting/build stability

**Files:**
- Modify: `zero-api/src/main.rs` (if formatting needed)

**Step 1: Run `cargo fmt`**

**Step 2: Run `cargo test -p zero-api`**

**Step 3: Run `cargo check -p zero-api`**
