# Zero CLI TUI v1 Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Build a production-usable TUI interface for `zero-cli` with multi-session, streaming output, keyboard navigation, and compatibility fallback.

**Architecture:** Introduce a `tui` module that owns UI state, rendering, event loop, and runtime event integration while reusing existing provider/tool wiring from `main.rs`. Keep legacy stdio behavior behind `--no-tui`.

**Tech Stack:** Rust, tokio, clap, ratatui, crossterm, anyhow, zero-core.

---

### Task 1: Add TUI dependencies and mode flag

**Files:**
- Modify: `zero-cli/Cargo.toml`
- Modify: `zero-cli/src/main.rs`

**Step 1: Write the failing test**
- Add a unit test for CLI parsing expecting `--no-tui` to parse and set `no_tui=true`.

**Step 2: Run test to verify it fails**
- Run: `cargo test -p zero-cli cli_parses_no_tui_flag`
- Expected: FAIL (flag missing)

**Step 3: Write minimal implementation**
- Add `ratatui` + `crossterm` dependencies.
- Add `no_tui: bool` clap arg with `--no-tui`.

**Step 4: Run test to verify it passes**
- Run: `cargo test -p zero-cli cli_parses_no_tui_flag`
- Expected: PASS

### Task 2: Create TUI state and reducer actions

**Files:**
- Create: `zero-cli/src/tui/mod.rs`
- Create: `zero-cli/src/tui/app.rs`

**Step 1: Write the failing tests**
- Test session creation, focus switching, input editing, submit transition to busy state.

**Step 2: Run tests to verify they fail**
- Run: `cargo test -p zero-cli tui::app`
- Expected: FAIL

**Step 3: Write minimal implementation**
- Implement app/session/message structs + action methods used by tests.

**Step 4: Run tests to verify they pass**
- Run: `cargo test -p zero-cli tui::app`
- Expected: PASS

### Task 3: Add runtime event model and integration logic

**Files:**
- Create: `zero-cli/src/tui/runtime.rs`
- Modify: `zero-cli/src/tui/app.rs`

**Step 1: Write the failing tests**
- Test `TokenDelta` appends to streaming buffer.
- Test `Done` finalizes assistant message.
- Test `Error` marks session not busy and records status.

**Step 2: Run tests to verify they fail**
- Run: `cargo test -p zero-cli runtime_event`
- Expected: FAIL

**Step 3: Write minimal implementation**
- Add runtime event enum + state-application methods.

**Step 4: Run tests to verify they pass**
- Run: `cargo test -p zero-cli runtime_event`
- Expected: PASS

### Task 4: Implement rendering with ratatui

**Files:**
- Create: `zero-cli/src/tui/ui.rs`

**Step 1: Write the failing test**
- Add a render smoke test (buffer draw) for non-empty output.

**Step 2: Run test to verify it fails**
- Run: `cargo test -p zero-cli render_smoke`
- Expected: FAIL

**Step 3: Write minimal implementation**
- Build 3-pane layout + help popup and status line rendering.

**Step 4: Run test to verify it passes**
- Run: `cargo test -p zero-cli render_smoke`
- Expected: PASS

### Task 5: Implement crossterm event loop and wire into main

**Files:**
- Create: `zero-cli/src/tui/event.rs`
- Modify: `zero-cli/src/main.rs`

**Step 1: Write the failing test**
- Add unit test for key translation (`q`, `Tab`, arrows, `Enter`, `Ctrl+C`).

**Step 2: Run test to verify it fails**
- Run: `cargo test -p zero-cli key_translation`
- Expected: FAIL

**Step 3: Write minimal implementation**
- Event pump and key mapping.
- Add `run_tui(...)` path in main and keep `--no-tui` fallback.

**Step 4: Run test to verify it passes**
- Run: `cargo test -p zero-cli key_translation`
- Expected: PASS

### Task 6: Verification

**Files:**
- Modify: `README.md` (if needed)
- Modify: `README.zh-CN.md` (if needed)

**Step 1: Run targeted tests**
- Run: `cargo test -p zero-cli`
- Expected: PASS

**Step 2: Build check**
- Run: `cargo build -p zero-cli`
- Expected: PASS

**Step 3: Smoke commands**
- Run: `cargo run -p zero-cli -- --help`
- Run: `cargo run -p zero-cli -- --no-tui --help`
- Expected: PASS
