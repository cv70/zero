# Architecture Optimization Design (Performance, Concurrency, Real-World Task Success)

Date: 2026-03-02
Status: Approved
Scope: Zero platform (`zero-core`, `zero-api`, `zero-cli`)

## 1. Context And Goals

This design targets a reconstruction-level architecture optimization with two primary goals:

1. Performance and concurrency efficiency
2. Real-world problem-solving capability (end-to-end complex task success)

Priority scenarios:

1. Long-running multi-step task execution
2. Multi-task concurrent processing
3. Multi-agent collaboration

Optimization stance:

1. Balanced optimization across throughput, success rate, and cost
2. Ambition: significant improvements across all three dimensions
3. Refactor strategy: architecture-first reconstruction (not incremental patching)

## 2. Approach Options And Decision

### Option A: Local optimization on current monolith

- Improve hotspots, lock contention, cache, batching, and runtime tuning
- Pros: quick wins, lower short-term risk
- Cons: low architectural ceiling; weak long-term gains for complex orchestration

### Option B: Execution kernel reconstruction with control/data plane split (Selected)

- Split orchestration from execution into two planes
- Build unified task state machine, event backflow, backpressure, idempotent replay
- Pros: structural gains on throughput, success rate, and cost at once
- Cons: high reconstruction effort and short-term volatility

### Option C: Full microservices decomposition

- Split provider/tool/scheduler/memory into independent services
- Pros: maximal long-term scalability
- Cons: too heavy now; high consistency and ops complexity

Decision: Option B

## 3. Target Architecture

### 3.1 Layered model

1. Ingress Layer
- `zero-cli` and `zero-api` only handle access, auth, request normalization, streaming output
- No complex orchestration logic

2. Control Plane
- Components: `Task Orchestrator`, `DAG Planner`, `Scheduler`, `Retry/Compensation Manager`, `State Store`
- Responsibility: decide what to execute, in what order, and how to recover
- Output: normalized `ExecutionPlan` and `DispatchEvent`

3. Data Plane
- Components: `Agent Runtime`, `Tool Executor Pool`, `Provider Gateway`, `Context/Memory Runtime`
- Responsibility: execute atomic steps and emit execution events
- Rule: no business orchestration decisions in data plane

4. Observability Plane
- Unified metrics and tracing at task/step/tool/token granularity
- Serves throughput, success rate, and cost optimization together

### 3.2 Boundary rules

1. Orchestration logic only in control plane; execution logic only in data plane
2. State transitions must go through event flow and task state machine
3. Every step must be retryable, interruptible, recoverable, and idempotent

## 4. Data Flow And Concurrency Model

### 4.1 End-to-end flow

1. Request -> Task Graph
- Ingress creates a task
- Control plane decomposes objective into DAG nodes with dependencies
- Each node carries priority, SLO, and budget (token/time/retry)

2. Scheduling -> Dispatch
- Scheduler uses priority + aging + quota fairness
- Dispatch unit is an atomic step, not full workflow

3. Execution -> Event Backflow
- Data plane executes step (agent inference, tool call, memory access)
- Emits `StepCompleted`/`StepFailed`/`StepTimedOut`
- Control plane advances state machine and unlocks next nodes

4. Completion/Recovery
- Success path returns aggregated output
- Failure path applies retry, model downgrade, tool fallback, or compensation

### 4.2 Concurrency model

1. Three-level concurrency
- Task-level concurrency
- DAG node-level concurrency
- I/O-level concurrency (provider/tool async + rate limits)

2. Backpressure
- Saturated pools push pressure signal back to scheduler
- Prevent cascading overload and queue explosion

3. Bulkhead isolation
- Isolate by tenant/task-type/tool-class
- Contain blast radius of local failures

4. Work stealing
- Idle workers steal tasks from equivalent-priority queues
- Improve utilization and reduce tail latency

## 5. Reliability And Real-World Task Success

### 5.1 Unified task state machine

- States: `Pending -> Runnable -> Running -> Waiting -> Succeeded/Failed/Compensated`
- Every transition carries event, reason code, and audit metadata

### 5.2 Layered retry policies

1. Provider failures
- fast retry + model downgrade path

2. Tool failures
- argument correction retry + tool substitution

3. Planning failures
- local re-plan instead of whole-task restart

4. Budget separation
- retry budgets per layer to avoid cost amplification

### 5.3 Idempotency and checkpoint recovery

1. Each step has `idempotency_key`
2. Duplicate delivery cannot cause duplicate side effects
3. Persistent checkpoints support crash recovery from last stable point

### 5.4 Real-task quality loop

1. Define `Task Success Contract` per task type
2. Verify post-execution via `Verifier`
- structure validation
- fact consistency checks
- executability checks
3. If contract fails, trigger auto-repair loop
- targeted tool call
- context enrichment
- local step rerun

### 5.5 Joint cost-performance governance

1. Per-task budgets for token/latency/retry
2. Budget overrun triggers downgrade path
3. Plan-template caching for repeated task patterns

## 6. Test Strategy And Acceptance Gates

### 6.1 Four-layer test pyramid

1. Unit tests
- state transitions, scheduler policies, retry decisions, idempotency

2. Component tests
- Scheduler + Queue
- Executor + Provider Gateway
- Verifier closed-loop

3. Integration tests
- long tasks (20+ steps)
- multi-task concurrency
- multi-agent collaboration + failure recovery

4. Stress and chaos tests
- rate limits, timeouts, dependency jitter, partial node outage, duplicate delivery

### 6.2 SLO/SLI metrics

1. Throughput: `tasks/min`, `steps/sec`
2. Success: `Task Success Rate`, `Retry-Recovered Success Rate`
3. Cost: `token/task`, `compute/task`, `p95 cost drift`
4. Latency: task completion `p50/p95/p99`
5. Stability: `MTTR`, failure blast radius

### 6.3 Acceptance gates

1. Under equal resources, throughput/success/cost all improve significantly vs baseline
2. Under fault injection, system degrades gracefully without global stall
3. After crash/restart, tasks resume from checkpoints without duplicated side effects
4. External CLI/API behavior remains compatible or has migration shim

### 6.4 Rollout strategy

1. Shadow traffic first (observe only)
2. Gradual cutover by task class
3. Auto rollback to legacy path when gates fail

## 7. Refactor Implications

1. Existing modules must be re-cut around control/data planes
2. Legacy direct calls between orchestration and execution must be replaced by event contracts
3. Scheduler and state store become first-class runtime primitives
4. Observability moves from optional diagnostics to mandatory runtime contract

## 8. Non-Goals (Current Stage)

1. Full microservices split across network boundaries
2. Cross-region orchestration and geo-distributed consensus
3. End-user feature expansion unrelated to architecture objectives

## 9. Next Step

After this design, the next mandatory step is writing an implementation plan with the `writing-plans` workflow, including milestones, module migration sequence, risk controls, and verification matrix.
