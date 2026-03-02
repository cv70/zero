# Zero Web Design

## Goal
Build `zero-web` as the browser control surface for `zero-api`, covering the full single-node in-memory orchestration flow already implemented by backend endpoints.

## Scope
- In scope: task create/list/filter/detail, plan submit/get, step dispatch, step callbacks (completed/failed/timeout), runtime state query, recovery decision, verification outcome, runtime metrics, consistent API error display.
- Out of scope: auth, persistence, distributed execution, SSR, advanced routing, design system package.

## Architecture
- `src/api/client.ts`: typed API client wrapping all `zero-api` endpoints and normalizing error contract (`error/code/details`).
- `src/types.ts`: frontend DTO and payload types matching `zero-api` contracts.
- `src/App.tsx`: single-page orchestration console with sections for task lifecycle and runtime operations.
- `src/App.css`: intentional visual language with CSS variables, non-default type stack, responsive layout, and focused interaction states.

## UX Layout
- Header: API base URL, health status, quick refresh actions.
- Left column: create task form, task list (with status filter), task detail card.
- Right column: plan editor/submit, dispatch and callback actions, runtime state panel, recovery/verification controls, runtime metrics card.
- Feedback: inline loading states, success notices, structured error rendering (`code + message + details`).

## Data Flow
1. User triggers action from UI form/button.
2. `App.tsx` calls typed methods in `api/client.ts`.
3. Client executes `fetch` against `VITE_ZERO_API_BASE` (default `http://127.0.0.1:3000`).
4. Response maps to typed model or throws normalized API error.
5. UI updates local state and selectively refreshes task list/details/metrics.

## Error Handling
- Network failures: show `network_error` with actionable text.
- API failures: render backend `code`, `error`, and JSON `details` if present.
- Local validation: prevent empty task title, empty task id for id-bound operations, and malformed plan JSON.

## Testing Strategy
- Backend remains validated via `cargo test -p zero-api`.
- Frontend tests should cover at least: create task success, plan submit success, dispatch flow, and API error rendering.
- In current environment, `npm` is unavailable; implementation will include test-ready structure and explicit verification notes.
