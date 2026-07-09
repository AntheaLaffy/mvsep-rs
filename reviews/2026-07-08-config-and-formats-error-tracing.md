# config_and_formats error tracing review

## Findings ordered by severity

### Low follow-up: backend log redaction misses whitespace-formatted JSON token fields

`redact_sensitive` covers compact JSON markers such as `"token":"` and query/header-style markers such as `token=`, `api_token=`, and `bearer ` at `src-tauri/src/main.rs:1332` through `src-tauri/src/main.rs:1358`. `push_backend_log` applies that redaction to backend log messages at `src-tauri/src/main.rs:1381` through `src-tauri/src/main.rs:1387`, and the existing test covers compact JSON, query-style, and bearer-style values at `src-tauri/src/main.rs:3284` through `src-tauri/src/main.rs:3299`.

The helper does not currently catch common pretty-printed JSON forms such as `"token": "secret"` or `"authorization": "Bearer secret"` because the quoted markers require the colon to be immediately followed by the quote. I did not find a current `config_and_formats` path that logs full config JSON, so this is not blocking this batch. It is still worth tightening before future batches start logging larger structured payloads.

### Resolved: legacy `config.json` import failures now point at the legacy JSON path

`load_config_from_backend_store` now wraps malformed legacy JSON as `legacy config import failed at <legacy_config_json_path>: <source>` at `src-tauri/src/main.rs:380` through `src-tauri/src/main.rs:389`. The backend wrapper maps that class of failure to `state.paths.legacy_config_json_path`, while other `load_config` failures map to `state.paths.user_config_db_path`, at `src-tauri/src/main.rs:435` through `src-tauri/src/main.rs:443`.

The regression test `legacy_config_import_error_logs_legacy_path` exercises malformed legacy JSON through `to_tauri_result`, asserts `operation=load_config`, asserts the legacy path appears, and asserts `mvsep.db` is not reported as the failure path at `src-tauri/src/main.rs:3405` through `src-tauri/src/main.rs:3423`.

### Resolved: autosave DB failures are logged with structured context before Tauri stringification

`save_config` maps persistence errors to `BackendError::legacy("save_config", e).with_path(state.paths.user_config_db_path...)` at `src-tauri/src/main.rs:446` through `src-tauri/src/main.rs:455`. The Tauri command wrapper calls the shared edge at `src-tauri/src/main.rs:1401` through `src-tauri/src/main.rs:1405`. That edge logs `error.to_log_message()` first, then returns the legacy-compatible `String` error with `into_tauri_error()` at `src-tauri/src/main.rs:1374` through `src-tauri/src/main.rs:1378`.

This closes the prior diagnostic gap: frontend autosave still receives the same rejected string through `invoke('save_config')` at `src/main.ts:450` through `src/main.ts:461` and `src/main.ts:1029` through `src/main.ts:1037`, while the backend log now keeps `operation=save_config`, `path=<user_config.db>`, message, and source context before the Tauri boundary flattens the error.

## Scope reviewed

Batch: `config_and_formats`

Role: `error_tracing_reviewer`

Reviewed structured errors, operation names, config/format path context, backend log context/redaction, and Tauri-edge stringification. I did not review behavior parity, DB schema correctness, async ergonomics, Rust style, or frontend UX beyond the save-config autosave error path required for this role.

Boundary check: the batch remains behind the accepted Tauri command facade and `AppBackend` seam. The relevant migrated surfaces are config load/save and output-format listing; no py2rs runtime architecture is introduced.

## Files or interfaces inspected

- `docs/INDEX.md`
- `docs/architecture/backend-rewrite.md`
- `RESOURCES.md`
- `manifest/rewrite-status.yaml`
- `rewrite-records/README.md`
- `reviews/README.md`
- `docs/references/high-confidence-sources.md`
- `reviews/2026-07-08-config-and-formats-error-tracing.md` prior failed report
- `src-tauri/src/main.rs`
- `src/main.ts`
- `test-api/src/db/repositories.rs`
- `test-api/tests/db_integration.rs`
- Official Tauri 2 calling-Rust documentation: https://v2.tauri.app/develop/calling-rust/

## Tests or checks run

- `git status --short`
- `git diff -- src-tauri/src/main.rs`
- `git diff -- test-api/src/db/repositories.rs test-api/tests/db_integration.rs`
- `rg -n "BackendError|to_tauri_result|load_config_from_backend_store|save_config_to_backend_store|legacy_config_import_error|backend_error_keeps|push_backend_log|redact" src-tauri/src/main.rs`
- `rg -n "function saveConfig|const saveConfig|saveConfig|save_config|load_config|list_formats|autosave|auto.?save|schedule.*save" src/main.ts src/app -g '!node_modules'`
- `cd src-tauri && cargo test backend_error_keeps_context_until_tauri_edge`: passed
- `cd src-tauri && cargo test legacy_config_import_error_logs_legacy_path`: passed
- `cd src-tauri && cargo test backend_logs_redact_token_like_values`: passed
- `cd src-tauri && cargo test config_store_imports_legacy_json_once`: passed
- `cd src-tauri && cargo check`: passed, with existing `test-api` warning noise
- `cd src-tauri && cargo test`: passed 8 Tauri-side tests, with existing `test-api` warning noise

## Residual risk

- I did not run live MVSep API/proxy tests; they are outside this config/format error-tracing gate.
- I did not force filesystem permission failures for `user_config.db`; the review relies on source inspection of the `save_config`/`load_config` error mapping and the Tauri-edge regression tests.
- The redaction helper should be extended to whitespace JSON token fields before future batches log larger structured payloads.

## Promotion decision: pass-with-followups

The prior blocking error-tracing findings are resolved. `config_and_formats` can proceed from the error-tracing gate, with the low redaction gap tracked as a follow-up rather than a blocker.
