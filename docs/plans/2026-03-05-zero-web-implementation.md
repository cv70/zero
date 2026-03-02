# Zero Web Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Deliver a production-structured MVP `zero-web` that fully drives `zero-api` orchestration endpoints from a single React page.

**Architecture:** Add typed API and model modules, then rebuild `App.tsx` into an orchestration console that binds user actions to backend endpoints and keeps local UI state synchronized via targeted refreshes.

**Tech Stack:** React 19, TypeScript, Vite, fetch API, CSS

---

### Task 1: Add frontend type and API contracts

**Files:**
- Create: `zero-web/src/types.ts`
- Create: `zero-web/src/api/client.ts`

**Step 1: Write the failing test**
- Add/define expected type-level and runtime API assumptions in code comments and usage points in `App.tsx` imports (build should fail before files exist).

**Step 2: Run test to verify it fails**
- Run: `cd zero-web && npm run build`
- Expected: FAIL with missing module/type errors.

**Step 3: Write minimal implementation**
- Implement DTO types for tasks, plans, metrics, task state, API errors, and action payloads.
- Implement `createApiClient(baseUrl)` with methods for all required endpoints and shared error parsing.

**Step 4: Run test to verify it passes**
- Run: `cd zero-web && npm run build`
- Expected: PASS for modules/types compilation.

**Step 5: Commit**
```bash
git add zero-web/src/types.ts zero-web/src/api/client.ts
git commit -m "feat(web): add typed zero-api client contracts"
```

### Task 2: Build orchestration console UI and behavior

**Files:**
- Modify: `zero-web/src/App.tsx`
- Modify: `zero-web/src/App.css`

**Step 1: Write the failing test**
- Wire `App.tsx` to call non-existing client methods and state bindings first to force compile/runtime failures.

**Step 2: Run test to verify it fails**
- Run: `cd zero-web && npm run build`
- Expected: FAIL with unresolved identifiers / behavior gaps.

**Step 3: Write minimal implementation**
- Implement page sections and forms for all in-scope actions.
- Add state refresh logic for tasks/detail/metrics/state.
- Add structured success/error notices.
- Replace template CSS with responsive, intentional UI styles using CSS variables.

**Step 4: Run test to verify it passes**
- Run: `cd zero-web && npm run build`
- Expected: PASS.

**Step 5: Commit**
```bash
git add zero-web/src/App.tsx zero-web/src/App.css
git commit -m "feat(web): implement zero orchestration console"
```

### Task 3: Wire API-base configuration and docs

**Files:**
- Modify: `zero-web/README.md`

**Step 1: Write the failing test**
- Add documented run path requiring env config and verify docs mismatch with current behavior.

**Step 2: Run test to verify it fails**
- Run: manual doc verification against code.
- Expected: mismatch before update.

**Step 3: Write minimal implementation**
- Document `VITE_ZERO_API_BASE`, startup commands, and key UI operations.

**Step 4: Run test to verify it passes**
- Run: manual verification against `api/client.ts` defaults and exposed features.
- Expected: docs align with implementation.

**Step 5: Commit**
```bash
git add zero-web/README.md
git commit -m "docs(web): document zero-web api integration"
```

### Task 4: End-to-end verification and stabilization

**Files:**
- Modify: touched files as needed

**Step 1: Verify backend regression safety**
- Run: `cargo test -p zero-api`
- Expected: PASS.

**Step 2: Verify frontend build/test**
- Run: `cd zero-web && npm run build && npm run lint`
- Expected: PASS.

**Step 3: Fix and rerun until green**
- Address any compile/lint/runtime issues.

**Step 4: Final validation notes**
- Capture environment limitations if commands unavailable.

**Step 5: Commit**
```bash
git add docs/plans/2026-03-05-zero-web-design.md docs/plans/2026-03-05-zero-web-implementation.md
git commit -m "docs(plan): add zero-web design and implementation plan"
```
