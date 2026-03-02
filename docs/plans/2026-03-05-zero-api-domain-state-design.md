# Zero API Domain-State Layering Design

## Goal
Refactor `zero-api` into `domain + state` architecture (with `state` directly integrated with `zero-core`) while preserving API behavior.

## Layering
- `domain`: business rules and parsing logic (status filter, failure class/outcome parsing, path/payload consistency).
- `state`: all runtime state and `zero-core` integration (`TaskManager`, `ControlPlane`, `DataPlane`, in-memory indexes, metrics).
- `http`: DTOs, handlers, routes; translates domain errors into HTTP error contract.
- `main`: process bootstrap and dependency wiring only.

## Dependency Direction
- `http -> domain + state`
- `domain` independent from `axum` and `zero-core`
- `state` depends on `zero-core`

## Migration Strategy
1. Extract `state` from monolithic `main.rs`.
2. Extract domain business rules into dedicated module.
3. Move DTO/handlers/routes into `http` module.
4. Keep route-level integration tests as regression guard.

## Validation
- `cargo test -p zero-api`
- `cargo check -p zero-api`
