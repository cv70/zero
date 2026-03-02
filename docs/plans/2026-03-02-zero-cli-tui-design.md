# Zero CLI TUI v1 Design

**Date:** 2026-03-02
**Scope:** `zero-cli` interactive interface migration from stdio REPL to `ratatui + crossterm`

## Goals
- Replace plain REPL UX with a full-screen TUI.
- Preserve current provider/tool integration behavior.
- Support streaming output, multi-session view, and quick navigation.
- Keep a compatibility fallback path for non-TUI operation.

## Architecture
- `main.rs` keeps CLI parsing and startup routing.
- Add `zero-cli/src/tui/` module set:
  - `app.rs`: app/session/message state.
  - `ui.rs`: ratatui rendering.
  - `event.rs`: keyboard + tick event pump.
  - `runtime.rs`: bridge between UI and zero_core loops.
  - `actions.rs`: keybinding behavior.
- Reuse existing provider creation and tool registry logic.
- Keep legacy mode via `--no-tui` fallback.

## State Model
- `AppState`
  - `sessions: Vec<SessionState>`
  - `active_session: usize`
  - `focus: FocusArea`
  - `mode: UiMode`
  - `status: StatusLine`
- `SessionState`
  - `title`
  - `messages: Vec<UiMessage>`
  - `input`
  - `streaming`
  - `busy`

## Data Flow
- Input events -> action reducer -> state updates.
- Submit action sends prompt to runtime worker through channel.
- Runtime emits `TokenDelta`, `ToolEvent`, `Done`, `Error`.
- UI consumes events, merges streaming buffer, redraws each tick.

## Interaction
- `Enter` send, `Shift+Enter` newline.
- `Ctrl+C` / `q` quit.
- `Tab` switch focus area.
- `j/k` or arrows scroll.
- `h/l` or left/right switch session.
- `n` create session.
- `?` toggle help popup.

## Error Handling
- Provider/tool/runtime errors: status bar + system message.
- Streaming interruption keeps partial text with interrupted marker.
- Guaranteed terminal restore on exit.

## Testing
- Unit tests for action reducers and runtime-event->state transitions.
- Mocked runtime integration tests to validate streaming path.
- Smoke checks for `--no-tui` and TUI startup path.

## Rollout
- Default to TUI mode.
- `--no-tui` retains old behavior for compatibility and debugging.
