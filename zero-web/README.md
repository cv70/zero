# zero-web

Web console for `zero-api` orchestration endpoints.

## Run

```bash
cd zero-web
npm install
npm run dev
```

Default backend URL is `http://127.0.0.1:3000`.

Override with:

```bash
VITE_ZERO_API_BASE=http://127.0.0.1:3100 npm run dev
```

## Supported Operations

- `GET /healthz`
- `GET /metrics/runtime`
- Task create/list/filter/detail
- Plan submit/get
- Dispatch next step
- Step callback: completed/failed/timeout
- Runtime state query
- Recovery decision and verification outcome

## Notes

- Backend error contract is surfaced as `code: message` and optional JSON `details`.
- This app is single-page and targets local/operator usage.
