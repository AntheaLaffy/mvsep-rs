# Findings ordered by severity

## Blocking: none

The previous blocker is fixed. `AppBackend` no longer uses `Result<_, String>` as its internal fallible contract: the facade now defines `BackendResult<T> = Result<T, BackendError>` and the trait methods return `BackendResult<_>` instead of string errors: `src-tauri/src/main.rs:60`, `src-tauri/src/main.rs:130`, `src-tauri/src/main.rs:131`, `src-tauri/src/main.rs:249`.

The structured error type now has explicit fields for operation, message, endpoint, hash, path, HTTP status and source: `src-tauri/src/main.rs:62`, `src-tauri/src/main.rs:68`, `src-tauri/src/main.rs:75`. The legacy adapter wraps legacy string errors with operation context and, where available at the facade boundary, endpoint/path/hash context: `src-tauri/src/main.rs:253`, `src-tauri/src/main.rs:257`, `src-tauri/src/main.rs:301`, `src-tauri/src/main.rs:321`, `src-tauri/src/main.rs:453`, `src-tauri/src/main.rs:473`, `src-tauri/src/main.rs:483`, `src-tauri/src/main.rs:491`, `src-tauri/src/main.rs:505`, `src-tauri/src/main.rs:523`.

Tauri command wrappers still return `Result<_, String>`, but only at the command edge. They convert through `to_tauri_result`, which is the single stringification point for `BackendResult<T>` failures: `src-tauri/src/main.rs:1219`, `src-tauri/src/main.rs:1220`, `src-tauri/src/main.rs:1237`, `src-tauri/src/main.rs:1240`, `src-tauri/src/main.rs:1521`, `src-tauri/src/main.rs:1524`. That is compatible with Tauri's command boundary model, where command return values and errors must be serializable for the frontend invoke path.

## Medium follow-up: HTTP status is modeled but not populated by the legacy adapter

`BackendErrorContext` includes `http_status`, and `BackendError::with_http_status` exists: `src-tauri/src/main.rs:74`, `src-tauri/src/main.rs:109`. Current code has no call site for `with_http_status` beyond the helper definition.

The result is that some HTTP failures remain status-in-source-string rather than status-in-structured-field. For example, fallback endpoint failures record `url -> status` in a plain attempt string, and download stream failures return a message like `http status ... while downloading file`: `src-tauri/src/main.rs:917`, `src-tauri/src/main.rs:929`, `src-tauri/src/main.rs:2817`, `src-tauri/src/main.rs:2826`.

This does not reintroduce the previous seam blocker because `AppBackend` can now carry structured status values for future adapters. It should be followed up when the legacy HTTP helpers are split or when `TestApiBackend` starts owning these calls.

## Medium follow-up: some direct stderr diagnostics still bypass the redaction helper

Stored backend log entries now pass through `redact_sensitive`, and frontend debug forwarding redacts before both backend-log storage and stderr: `src-tauri/src/main.rs:1177`, `src-tauri/src/main.rs:1223`, `src-tauri/src/main.rs:1228`, `src-tauri/src/main.rs:2974`, `src-tauri/src/main.rs:2984`. The new unit test covers representative `api_token`, JSON `token`, and `Bearer` forms: `src-tauri/src/main.rs:3163`, `src-tauri/src/main.rs:3177`.

Several backend `eprintln!` calls still print raw source strings outside that helper. The riskiest examples are CLI stderr and request/decode errors that may include upstream details: `src-tauri/src/main.rs:1809`, `src-tauri/src/main.rs:1813`, `src-tauri/src/main.rs:1841`, `src-tauri/src/main.rs:1844`, `src-tauri/src/main.rs:2482`, `src-tauri/src/main.rs:2507`, `src-tauri/src/main.rs:2688`.

I did not prove a runtime token leak. This is residual diagnostic debt, not a facade-contract blocker, because the user-facing backend log buffer now has a redaction boundary and the prior frontend-debug raw forwarding issue is fixed.

# Scope reviewed

Batch: `tauri_command_facade`

Role: `error_tracing_reviewer`

Focus: structured error boundary, command-edge stringification, redaction of backend/frontend logs, diagnosability context, and whether the previous `AppBackend` `Result<_, String>` blocker was fixed.

# Files or interfaces inspected

- `src-tauri/src/main.rs`
- `src/main.ts`
- `src-tauri/Cargo.toml`
- `package.json`
- Current working-tree diff for `src-tauri/src/main.rs`
- Existing behavior review: `reviews/2026-07-08-tauri-command-facade-behavior.md`
- Previous error-tracing fail report: `reviews/2026-07-08-tauri-command-facade-error-tracing.md`
- Project rewrite context: `docs/INDEX.md`, `docs/architecture/backend-rewrite.md`, `RESOURCES.md`, `manifest/rewrite-status.yaml`, `rewrite-records/README.md`, `reviews/README.md`
- Tauri 2 official command documentation referenced from `docs/references/high-confidence-sources.md`: https://v2.tauri.app/develop/calling-rust/

# Tests or checks run

- `cargo test` in `src-tauri`: passed. This ran 4 `src/main.rs` tests, including `backend_error_keeps_context_until_tauri_edge` and `backend_logs_redact_token_like_values`.
- `git diff --check`: passed.
- `git diff --stat`: production diff is limited to `src-tauri/src/main.rs`.
- `rg -n "Result<[^\\n>]+, String>|Result<[^\\n>]+String>" src-tauri/src/main.rs`: remaining string errors are at legacy helpers and Tauri command wrappers, not the `AppBackend` trait contract.
- `rg -n "BackendResult<T>|BackendError|into_tauri_error|to_tauri_result|redact_sensitive|backend_logs_redact_token_like_values|backend_error_keeps_context_until_tauri_edge" src-tauri/src/main.rs`: confirmed the structured error seam, edge conversion helper, redaction helper and regression tests.
- `rg -n "with_http_status" src-tauri/src/main.rs`: confirmed no HTTP-status population call sites yet.
- `rg -n "eprintln!|println!|log::|tracing::|warn!|error!|info!" src-tauri/src/main.rs`: inspected direct diagnostic outputs for redaction bypass risk.
- `rg -n "py2rs|python runtime|runtime router|script-as" src-tauri/src/main.rs src-tauri/Cargo.toml`: no py2rs runtime architecture imports or router concepts found.

# Residual risk

This was a static/local test review. It did not exercise live MVSep API responses, real upload/download streams, proxy failures, CLI stderr content, OS file-manager behavior, or Tauri invoke serialization at runtime. The redaction helper covers common token-like markers but is not a proof against arbitrary secret formats in third-party error text.

# Promotion decision: pass-with-followups

`tauri_command_facade` should pass the error-tracing gate with follow-ups. The previous blocker is fixed: `AppBackend` now carries structured `BackendError` values internally, and Tauri command wrappers stringify only at the edge. The remaining issues are follow-up diagnostic hardening around structured HTTP status extraction and raw stderr output.
