# Transfer Error Tracing Review

## Findings ordered by severity

No blocking or follow-up findings in the reviewed transfer error-tracing scope.

The previous failure items are addressed in the current diff:

- `TransferError` now carries `http_status`, `url`, `path`, and `cancelled` state in `test-api/src/file_transfer.rs:51`.
- `transfer_backend_error` maps transfer URL/path/status into `BackendErrorContext` while preserving the operation and optional task hash in `src-tauri/src/main.rs:315`.
- Tauri error payloads and stored backend logs pass through `redact_sensitive` at `src-tauri/src/main.rs:130` and `src-tauri/src/main.rs:1543`.
- Transfer stderr paths now print redacted structured log messages at `src-tauri/src/main.rs:2556`, `src-tauri/src/main.rs:2722`, and `src-tauri/src/main.rs:2873`.
- The `/separation/get` metadata query checks HTTP status before JSON parsing and records `http_status` at `src-tauri/src/main.rs:2728`.
- Cancellation text still crosses the Tauri edge as `Download cancelled`, and the frontend recognizes that text at `src/app/services/tasks.ts:5`.

## Scope reviewed

- Batch: `transfer`
- Role: `error_tracing_reviewer`
- Scope: structured errors, useful diagnostic context, redaction, operation/hash/path/endpoint/status preservation, and cancellation/error text crossing the Tauri edge.
- Out of scope: behavior parity, async ergonomics, Rust style, data schema, frontend UX, and promotion of unrelated DB/CLI changes present in the worktree.

## Files or interfaces inspected

- `docs/INDEX.md`
- `docs/architecture/backend-rewrite.md`
- `RESOURCES.md`
- `manifest/rewrite-status.yaml`
- `rewrite-records/README.md`
- `reviews/README.md`
- Current `git status`, `git diff --stat`, and relevant diff hunks
- `src-tauri/src/main.rs`
- `test-api/src/file_transfer.rs`
- `src/app/services/tasks.ts`
- Relevant tests in `src-tauri/src/main.rs`
- Relevant tests in `test-api/src/file_transfer.rs`

## Passing checks

- Transfer upload failures preserve endpoint/path/status before the Tauri edge through `upload_file_async` and `transfer_backend_error`.
- Transfer download stream failures preserve the remote file URL, local output path, task hash, operation, and HTTP status when available.
- The metadata query path preserves operation, endpoint, hash, output path, and HTTP status for non-success responses before body parsing.
- Tauri-facing error strings and backend log entries redact common token forms before returning to the frontend or storing in backend logs.
- Frontend debug-log forwarding redacts again at the backend edge, so task download errors do not reintroduce token-bearing error strings.
- Cancellation remains distinguishable by the preserved `Download cancelled` text and by the `TransferError::is_cancelled` branch used for warning-level logging.

## Tests or checks run

- `cargo test file_transfer` from `test-api`: passed, 4 transfer tests.
- `cargo test tauri_error_payload_redacts_transfer_tokens` from `src-tauri`: passed.
- `cargo test transfer_backend_error_preserves_status_url_hash_and_path` from `src-tauri`: passed.
- `cargo test backend_logs_redact_token_like_values` from `src-tauri`: passed.
- `cargo test backend_error_keeps_context_until_tauri_edge` from `src-tauri`: passed.

## Residual risk

- There is no end-to-end Tauri test that drives a real failing `download_result` command through the frontend task service.
- Cancellation is still classified by stable text at the frontend edge rather than by a structured Tauri payload field. That matches the current public command contract, but future message edits could break classification.
- Redaction coverage is tested for common token syntaxes; signed CDN URLs or unusual secret parameter names would need additional patterns if the service starts returning them.

## Promotion decision

pass
