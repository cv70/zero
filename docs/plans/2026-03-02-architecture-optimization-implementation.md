# Control/Data Plane Refactor Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Rebuild Zero runtime into control plane + data plane architecture to significantly improve throughput, success rate, and cost efficiency for long-running, concurrent, multi-agent tasks.

**Architecture:** Introduce explicit orchestration contracts (state machine, dispatch events, retries, checkpoints) in control plane, and move all execution primitives (agent/tool/provider/memory steps) into data plane workers. Connect both planes via typed event flow with backpressure and idempotent step execution.

**Tech Stack:** Rust, Tokio async runtime, async-trait, serde, tracing, existing `zero-core` modules.

---

### Task 1: Establish Runtime Contracts (Control/Data/Event)

**Files:**
- Create: `zero-core/src/runtime/contracts.rs`
- Modify: `zero-core/src/runtime/mod.rs`
- Modify: `zero-core/src/lib.rs`
- Test: `zero-core/src/runtime/contracts.rs` (inline unit tests)

**Step 1: Write the failing test**

```rust
#[test]
fn dispatch_event_roundtrip() {
    let evt = DispatchEvent::step_dispatched("task-1", "step-1", "agent.execute");
    let s = serde_json::to_string(&evt).unwrap();
    let back: DispatchEvent = serde_json::from_str(&s).unwrap();
    assert_eq!(back.task_id(), "task-1");
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test -p zero-core runtime::contracts::tests::dispatch_event_roundtrip -v`
Expected: FAIL with unresolved `DispatchEvent`/missing module

**Step 3: Write minimal implementation**

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DispatchEvent { StepDispatched { task_id: String, step_id: String, op: String } }
```

Also add:
- `ExecutionPlan`, `StepSpec`, `StepResultEvent`
- `TaskState` enum: `Pending|Runnable|Running|Waiting|Succeeded|Failed|Compensated`

**Step 4: Run test to verify it passes**

Run: `cargo test -p zero-core runtime::contracts::tests::dispatch_event_roundtrip -v`
Expected: PASS

**Step 5: Commit**

```bash
git add zero-core/src/runtime/contracts.rs zero-core/src/runtime/mod.rs zero-core/src/lib.rs
git commit -m "feat(runtime): add control/data plane runtime contracts"
```

### Task 2: Implement Control Plane State Machine And Orchestrator

**Files:**
- Create: `zero-core/src/runtime/control_plane.rs`
- Modify: `zero-core/src/runtime/mod.rs`
- Modify: `zero-core/src/task/model.rs`
- Test: `zero-core/src/runtime/control_plane.rs` (inline unit tests)

**Step 1: Write the failing test**

```rust
#[tokio::test]
async fn state_machine_advances_on_step_completed() {
    let mut cp = ControlPlane::new_in_memory();
    cp.accept_plan(sample_plan()).await.unwrap();
    cp.on_step_completed("task-1", "step-1").await.unwrap();
    assert_eq!(cp.task_state("task-1").await.unwrap(), TaskState::Running);
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test -p zero-core runtime::control_plane::tests::state_machine_advances_on_step_completed -v`
Expected: FAIL with missing `ControlPlane`

**Step 3: Write minimal implementation**

```rust
pub struct ControlPlane { /* in-memory state store + dependency index */ }
impl ControlPlane {
    pub async fn accept_plan(&mut self, plan: ExecutionPlan) -> Result<(), ZeroError> { /* ... */ }
    pub async fn on_step_completed(&mut self, task_id: &str, step_id: &str) -> Result<(), ZeroError> { /* ... */ }
}
```

Include:
- deterministic transition table
- transition reason codes
- no direct execution logic in this module

**Step 4: Run test to verify it passes**

Run: `cargo test -p zero-core runtime::control_plane::tests::state_machine_advances_on_step_completed -v`
Expected: PASS

**Step 5: Commit**

```bash
git add zero-core/src/runtime/control_plane.rs zero-core/src/runtime/mod.rs zero-core/src/task/model.rs
git commit -m "feat(control-plane): add orchestrator state machine and transition handling"
```

### Task 3: Implement Data Plane Worker Pool With Idempotent Step Execution

**Files:**
- Create: `zero-core/src/runtime/data_plane.rs`
- Create: `zero-core/src/runtime/idempotency.rs`
- Modify: `zero-core/src/runtime/mod.rs`
- Modify: `zero-core/src/tool/dispatcher.rs`
- Modify: `zero-core/src/provider/loop_provider.rs`
- Test: `zero-core/src/runtime/data_plane.rs` (inline async tests)

**Step 1: Write the failing test**

```rust
#[tokio::test]
async fn duplicate_step_is_not_executed_twice() {
    let dp = DataPlane::new_for_test();
    let spec = sample_step("task-1", "step-1", "idem-1");
    let a = dp.execute_step(spec.clone()).await.unwrap();
    let b = dp.execute_step(spec).await.unwrap();
    assert_eq!(a.step_id, b.step_id);
    assert!(b.from_cache);
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test -p zero-core runtime::data_plane::tests::duplicate_step_is_not_executed_twice -v`
Expected: FAIL with missing `DataPlane`

**Step 3: Write minimal implementation**

```rust
pub struct DataPlane { /* worker pool + bounded queue + idempotency store */ }
pub async fn execute_step(&self, step: StepSpec) -> Result<StepResultEvent, ZeroError> { /* ... */ }
```

Include:
- bounded channel for backpressure
- `idempotency_key` lookup/write-before-side-effect
- adapters to existing tool/provider entry points

**Step 4: Run test to verify it passes**

Run: `cargo test -p zero-core runtime::data_plane::tests::duplicate_step_is_not_executed_twice -v`
Expected: PASS

**Step 5: Commit**

```bash
git add zero-core/src/runtime/data_plane.rs zero-core/src/runtime/idempotency.rs zero-core/src/runtime/mod.rs zero-core/src/tool/dispatcher.rs zero-core/src/provider/loop_provider.rs
git commit -m "feat(data-plane): add worker pool with idempotent step execution and backpressure"
```

### Task 4: Add Retry/Compensation Policies And Budget Enforcement

**Files:**
- Create: `zero-core/src/runtime/recovery.rs`
- Modify: `zero-core/src/runtime/control_plane.rs`
- Modify: `zero-core/src/agent/loop_config.rs`
- Test: `zero-core/src/runtime/recovery.rs` (unit tests)

**Step 1: Write the failing test**

```rust
#[test]
fn provider_timeout_triggers_retry_then_downgrade() {
    let policy = RecoveryPolicy::default();
    let decision1 = policy.on_failure(FailureClass::ProviderTimeout, 1);
    let decision2 = policy.on_failure(FailureClass::ProviderTimeout, 2);
    assert_eq!(decision1, RecoveryDecision::RetryFast);
    assert_eq!(decision2, RecoveryDecision::DowngradeModel);
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test -p zero-core runtime::recovery::tests::provider_timeout_triggers_retry_then_downgrade -v`
Expected: FAIL with unresolved `RecoveryPolicy`

**Step 3: Write minimal implementation**

```rust
pub enum FailureClass { ProviderTimeout, ToolInvalidArgs, PlanningMismatch }
pub enum RecoveryDecision { RetryFast, RetryWithPatchedArgs, DowngradeModel, ReplanLocal, Compensate, Abort }
```

Include:
- per-layer retry budgets
- token/time/retry budget guardrails
- compensation trigger on terminal failure

**Step 4: Run test to verify it passes**

Run: `cargo test -p zero-core runtime::recovery::tests::provider_timeout_triggers_retry_then_downgrade -v`
Expected: PASS

**Step 5: Commit**

```bash
git add zero-core/src/runtime/recovery.rs zero-core/src/runtime/control_plane.rs zero-core/src/agent/loop_config.rs
git commit -m "feat(recovery): add layered retry, downgrade, and compensation policies"
```

### Task 5: Add Verifier Loop For Real-Task Success Contract

**Files:**
- Create: `zero-core/src/runtime/verifier.rs`
- Modify: `zero-core/src/runtime/control_plane.rs`
- Modify: `zero-core/src/task/model.rs`
- Test: `zero-core/src/runtime/verifier.rs` (unit tests)

**Step 1: Write the failing test**

```rust
#[test]
fn failed_contract_requests_repair_loop() {
    let verifier = TaskVerifier::default();
    let out = verifier.verify(sample_contract(), sample_bad_result());
    assert_eq!(out, VerifyOutcome::NeedsRepair);
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test -p zero-core runtime::verifier::tests::failed_contract_requests_repair_loop -v`
Expected: FAIL with missing verifier module

**Step 3: Write minimal implementation**

```rust
pub enum VerifyOutcome { Passed, NeedsRepair(Vec<RepairAction>), HardFail }
pub struct TaskVerifier;
```

Include:
- contract fields for structure/fact/executability checks
- control-plane hook: verification failure routes back to repair steps

**Step 4: Run test to verify it passes**

Run: `cargo test -p zero-core runtime::verifier::tests::failed_contract_requests_repair_loop -v`
Expected: PASS

**Step 5: Commit**

```bash
git add zero-core/src/runtime/verifier.rs zero-core/src/runtime/control_plane.rs zero-core/src/task/model.rs
git commit -m "feat(quality): add task success contract verifier and repair loop trigger"
```

### Task 6: Observability, Benchmarks, And Rollout Guardrails

**Files:**
- Create: `zero-core/src/runtime/metrics.rs`
- Modify: `zero-core/src/perf/monitor.rs`
- Modify: `benches/bench_basic.rs`
- Create: `zero-core/tests/runtime_integration.rs`
- Modify: `zero-api/src/main.rs`
- Modify: `zero-cli/src/main.rs`

**Step 1: Write the failing integration test**

```rust
#[tokio::test]
async fn runtime_recovers_after_injected_timeout() {
    let harness = RuntimeHarness::new();
    let result = harness.run_fault_injected_workload().await;
    assert!(result.success_rate >= 0.95);
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test -p zero-core --test runtime_integration runtime_recovers_after_injected_timeout -v`
Expected: FAIL because harness/metrics are absent

**Step 3: Write minimal implementation**

```rust
pub struct RuntimeMetrics {
    pub tasks_per_min: f64,
    pub task_success_rate: f64,
    pub token_per_task: f64,
}
```

Include:
- task/step/tool/token metric emitters
- p50/p95/p99 latency collection
- API/CLI compatibility shim for legacy request path

**Step 4: Run verification suite**

Run: `cargo test -p zero-core -v`
Expected: PASS

Run: `cargo bench -p benches bench_basic -- --quick`
Expected: Benchmark completes and prints baseline vs refactor stats

**Step 5: Commit**

```bash
git add zero-core/src/runtime/metrics.rs zero-core/src/perf/monitor.rs benches/bench_basic.rs zero-core/tests/runtime_integration.rs zero-api/src/main.rs zero-cli/src/main.rs
git commit -m "feat(observability): add runtime SLO metrics, integration harness, and compatibility guards"
```

## Cross-Task Rules

1. Use `@test-driven-development` for every task before implementation edits.
2. Keep each commit scoped to one task and passing tests only.
3. Prefer reuse of existing modules (`task`, `scheduler`, `tool`, `provider`) via adapters before creating new abstractions.
4. Do not remove legacy paths until runtime integration tests and benchmark gates pass.
5. If duplicate/contradictory trait definitions are found, consolidate in the current task before adding new callers.

## Verification Matrix (Must Pass Before Merge)

1. Unit/Component: `cargo test -p zero-core -v`
2. Integration: `cargo test -p zero-core --test runtime_integration -v`
3. Benchmarks: `cargo bench -p benches bench_basic -- --quick`
4. Lint/Format: `cargo clippy --all-targets --all-features -- -D warnings` and `cargo fmt --check`
5. CLI/API smoke:
- `cargo run -p zero-cli -- --help`
- `cargo run -p zero-api -- --help`

## Rollout Checklist

1. Add shadow mode flag in API/CLI to run new control/data plane without external side effects.
2. Enable by task class: low-risk -> medium -> high complexity.
3. Define automatic rollback trigger thresholds on success-rate and latency regressions.
4. Keep checkpoint and idempotency stores enabled before first production traffic.
