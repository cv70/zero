# Zero API Domain-State Refactor Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Refactor `zero-api` to layered `domain + state` architecture while preserving existing API behavior.

**Architecture:** Move business rules to `domain`, `zero-core` integration and in-memory indexes to `state`, HTTP contracts and handlers to `http`, and keep `main.rs` as wiring-only entrypoint.

**Tech Stack:** Rust, axum, tokio, serde, zero-core

---

### Task 1: Baseline and failing safety net

**Files:**
- Modify: `zero-api/src/main.rs` (tests relocation prep)

1. Run current tests as baseline (`cargo test -p zero-api`).
2. Keep route-level tests intact as migration guard.

### Task 2: Introduce `state` layer

**Files:**
- Create: `zero-api/src/state/mod.rs`
- Create: `zero-api/src/state/app_state.rs`

1. Move `AppState` struct and initialization (`build_state`) into `state`.
2. Expose `SharedState` methods for task existence checks and zero-core interactions.

### Task 3: Introduce `domain` layer

**Files:**
- Create: `zero-api/src/domain/mod.rs`
- Create: `zero-api/src/domain/error.rs`
- Create: `zero-api/src/domain/models.rs`
- Create: `zero-api/src/domain/service.rs`

1. Define domain errors and parse helpers for status/failure/outcome.
2. Move business rule checks (id match, status filter, no dispatchable step).

### Task 4: Introduce `http` layer and wire router

**Files:**
- Create: `zero-api/src/http/mod.rs`
- Create: `zero-api/src/http/dto.rs`
- Create: `zero-api/src/http/handlers.rs`
- Create: `zero-api/src/http/routes.rs`
- Modify: `zero-api/src/main.rs`

1. Move DTOs/handlers out of `main.rs` into `http` modules.
2. Map domain errors to HTTP error contract.
3. Build router in `http::routes::app`.
4. Keep integration tests in `http` module.

### Task 5: Verify and stabilize

**Files:**
- Modify: touched files as needed

1. Run `cargo fmt`.
2. Run `cargo test -p zero-api`.
3. Run `cargo check -p zero-api`.
4. Fix any regressions and rerun until green.
