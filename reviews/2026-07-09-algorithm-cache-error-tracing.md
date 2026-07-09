# algorithm_cache error tracing re-review

## Findings ordered by severity

### Low follow-up: backend log redaction still misses whitespace-formatted JSON token fields

`redact_sensitive` handles compact JSON markers such as `"token":"` and `"api_token":"` at `src-tauri/src/main.rs:1538` through `src-tauri/src/main.rs:1547`, then handles unquoted markers such as `token=` and `bearer ` at `src-tauri/src/main.rs:1548` through `src-tauri/src/main.rs:1564`. It still does not match common pretty-printed JSON forms like `"token": "secret"` because the quoted-key marker requires no whitespace and the unquoted `token:` marker is not present inside `"token":`.

The current algorithm-cache error path does not appear to log full remote JSON payloads or config bodies, and the existing redaction test covers compact JSON/query/bearer values at `src-tauri/src/main.rs:3319` through `src-tauri/src/main.rs:3332`. This is not a promotion blocker for this batch, but it remains a diagnostics/log-safety follow-up.

### Resolved: local `mvsep.db` and `user_config.db` failures are no longer endpoint-only errors

The prior blocker is fixed. Local algorithm-cache helpers now include explicit path-bearing error strings: metadata reads/writes use `state.paths.user_config_db_path` at `src-tauri/src/main.rs:572` through `src-tauri/src/main.rs:584` and `src-tauri/src/main.rs:587` through `src-tauri/src/main.rs:600`; cache DB writes and reads use `state.paths.mvsep_db_path` at `src-tauri/src/main.rs:604` through `src-tauri/src/main.rs:617` and `src-tauri/src/main.rs:620` through `src-tauri/src/main.rs:660`.

The `fetch_latest_algorithm_info` wrapper now attaches `path=<...>` for local cache DB or metadata failures, and only attaches `endpoint=<.../app/algorithms>` for remote-fetch errors at `src-tauri/src/main.rs:753` through `src-tauri/src/main.rs:764`. The post-fetch inner logs also include the helper error messages, so save/reload failures carry the local path text before the final Tauri-edge structured log at `src-tauri/src/main.rs:2128` through `src-tauri/src/main.rs:2155`.

### Resolved: `refresh_algorithm_list_from_local` metadata failures now log `user_config.db`

The prior conflicting-path finding is fixed. `algorithm_cache_error_path` selects `user_config.db` when the message contains the metadata path or the `algorithm cache metadata` marker, and otherwise falls back to `mvsep.db` at `src-tauri/src/main.rs:683` through `src-tauri/src/main.rs:688`. `refresh_algorithm_list_from_local` uses that selector before building `BackendError` at `src-tauri/src/main.rs:768` through `src-tauri/src/main.rs:775`.

The regression test `algorithm_cache_metadata_error_logs_user_config_path` creates a directory at the `user_config.db` path, triggers the metadata failure, and asserts the Tauri-edge backend log contains `operation=refresh_algorithm_list_from_local` plus `path=<user_config.db>` and not `path=<mvsep.db>` at `src-tauri/src/main.rs:3681` through `src-tauri/src/main.rs:3700`.

## Scope reviewed

Batch: `algorithm_cache`

Role: `error_tracing_reviewer`

Reviewed structured errors, operation names, path/endpoint context, backend logs, redaction, and Tauri-edge stringification for:

- `fetch_latest_algorithm_info`
- `refresh_algorithm_list_from_local`
- `get_algorithm_details_from_local`
- algorithm-cache DB open/write/read failures
- cache metadata read/write failures
- remote algorithm fetch failures

I did not review behavior parity, schema/data correctness, async ergonomics, Rust style, or frontend UX beyond what was needed to evaluate error propagation and logging.

Boundary check: the implementation remains behind the accepted Tauri command facade and `AppBackend` seam. I did not see py2rs runtime architecture imported into this batch.

## Files or interfaces inspected

- `docs/INDEX.md`
- `docs/architecture/backend-rewrite.md`
- `RESOURCES.md`
- `manifest/rewrite-status.yaml`
- `rewrite-records/README.md`
- `reviews/README.md`
- `docs/references/high-confidence-sources.md`
- `reviews/2026-07-09-algorithm-cache-error-tracing.md`
- `reviews/2026-07-09-algorithm-cache-behavior.md`
- `reviews/2026-07-09-algorithm-cache-data-algorithm.md`
- `src-tauri/src/main.rs`
- `test-api/src/db/repositories.rs`
- `test-api/src/db/user_config.rs`
- Official Tauri 2 calling-Rust documentation: https://v2.tauri.app/develop/calling-rust/

## Tests or checks run

- `git status --short`
- `git diff --name-only`
- `git diff -- src-tauri/src/main.rs`
- `git diff -- test-api/src/db/repositories.rs test-api/src/db/mod.rs test-api/src/db/user_config.rs test-api/tests/db_integration.rs`
- `rg -n "algorithm_cache|fetch_latest_algorithm_info|refresh_algorithm_list_from_local|get_algorithm_details_from_local|algorithm_cache_error_path|metadata|user_config|mvsep\\.db|to_tauri_result|BackendError" src-tauri/src/main.rs`
- `rg -n "replace_algorithm_cache|algorithm_cache|set_json|get_json|metadata|algorithm_last_fetched|cache" test-api/src/db test-api/tests/db_integration.rs`
- `rg -n "fn fetch_remote_algorithms_raw|async fn fetch_remote_algorithms_raw|build_api_url|Authorization|Bearer|token|api_token|format!\\(.*token|push_backend_log\\(.*token" src-tauri/src/main.rs`
- `cd src-tauri && cargo test algorithm_cache_metadata_error_logs_user_config_path -- --nocapture`: passed 1 test.
- `cd src-tauri && cargo test algorithm_cache_ -- --nocapture`: passed 5 tests.
- `cd src-tauri && cargo test backend_logs_redact_token_like_values -- --nocapture`: passed 1 test.
- `git diff --check`: passed.

Writer-provided checks noted but not rerun in full: `cd src-tauri && cargo test` passed 13 tests; `cd test-api && cargo test` passed 14 DB integration tests plus doctest with proxy tests ignored; `npm run build` passed; `cd src-tauri && cargo clippy --all-targets -- -D warnings` passed; `git diff --check` passed. `cd test-api && cargo clippy --all-targets -- -D warnings` still fails on existing bin/library warning debt.

## Residual risk

- I did not force a remote MVSep endpoint failure in a live network path; the endpoint-vs-path distinction was reviewed from the wrapper and local fetch construction.
- I did not force every possible filesystem/SQLite corruption mode; the new metadata-path regression covers the previously failing `user_config.db` path selection branch.
- The low redaction gap should be fixed before later batches log larger JSON/config payloads.

## Promotion decision: pass-with-followups

The prior error-tracing blockers are resolved. `algorithm_cache` can pass this gate with the remaining redaction follow-up tracked outside promotion blocking scope.
