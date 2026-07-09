# Backend Rewrite Architecture

## Source Boundaries

This architecture borrows the `teach` skill's stateful-workspace pattern: mission, resources, notes, records and minimum scoped progression. In engineering terms, every migration batch should be the smallest useful slice that can be understood, tested, reviewed and resumed later.

This architecture borrows only software engineering principles from `py2rs`: behavior first, reversible states, manifest-driven progress, role-separated agents and review gates. It does not borrow the py2rs Python/Rust runtime architecture. There is no `py/` and `rs/` split, no Python runtime router and no script-as-migration-unit rule.

## Current State

- The accepted seam is implemented and verified through the current manifest batches.
- `src/app/backend/gateway.ts` is the only frontend module that imports Tauri JavaScript APIs, calls `invoke`, or calls `listen`.
- Tauri command and progress event names remain stable; gateway methods preserve existing JavaScript payload field names.
- Config, output formats, algorithm cache, transfer, active tasks and task history are migrated behind the backend facade and verified by review gates.
- For migrated domains, rewritten backend storage is canonical. Legacy frontend storage is a one-time migration or rollback aid, not the source of truth.

## Target Seam

The migration seam is behind Tauri commands:

```text
Frontend
  -> BackendGateway
  -> Tauri commands with stable names
  -> AppBackend interface
  -> LegacyMainBackend or TestApiBackend adapter
```

Commands keep the public payload shape stable. The backend implementation can change without forcing every page to understand the new storage or transfer model.

## Core Interfaces

- `AppBackend`: config, formats, algorithm cache, task create/status/download/cancel, logs.
- `ProgressSink`: upload/download progress reporting; Tauri adapter emits existing event names.
- `BackendError`: structured error with operation, endpoint, hash, path, HTTP status and source.
- `BackendPaths`: explicit app config/data paths injected from Tauri, never inferred from cwd.

## Migration Batches

Each batch is a minimum boundary: one user-visible capability or one infrastructure seam, small enough to be verified independently and recorded in the manifest.

1. `tauri_command_facade`: verified.
2. `config_and_formats`: verified.
3. `algorithm_cache`: verified.
4. `transfer`: verified.
5. `task_persistence`: verified.
6. `frontend_gateway_and_ui`: verified.

The manifest remains the source of truth for exact batch state and required review reports.

## Error Tracing

- Business modules return structured errors; only Tauri command edges stringify.
- Every error from network, JSON parsing, filesystem, DB and cancellation includes the operation name.
- Upload/download errors include hash when available, local path, remote URL or endpoint and HTTP status.
- Frontend logs and backend logs must redact token-like values.

## Async And Ergonomics

- No blocking HTTP client inside Tauri async commands.
- No nested Tokio runtime in async command paths.
- High-frequency operations use progress events/channels, debounced saves and render coalescing.
- Polling must deduplicate in-flight requests per task hash.
- Cancel and retry flows must preserve partial download state when safe.

## Capability And Security

- Keep Tauri capabilities minimal and review `src-tauri/capabilities/default.json` whenever adding plugin commands.
- Do not broaden filesystem or shell permissions as a convenience.
- Frontend render helpers must escape untrusted strings from remote API, config, filenames and logs.
- Frontend pages, services and controllers must call backend functionality through `src/app/backend/gateway.ts`, not Tauri APIs directly.

## Acceptance Gates

- Behavior contract tests pass between legacy and new backend adapters for migrated methods.
- `npm run build` passes.
- `cd test-api && cargo test` passes.
- Clippy warnings are either fixed or intentionally documented during the batch.
- Required review report exists in `reviews/` before changing manifest status to `verified`.
