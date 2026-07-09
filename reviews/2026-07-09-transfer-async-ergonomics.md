# Transfer Async Ergonomics Review

## Findings ordered by severity

### Medium - Cancellation does not wake a stalled network await

`cancel_download` flips the active transfer token at `src-tauri/src/main.rs:2377`, but `download_file_async` only checks that token before the request and after a response chunk has already been read and written at `test-api/src/file_transfer.rs:556` and `test-api/src/file_transfer.rs:652`. It does not race cancellation against `request.send().await` at `test-api/src/file_transfer.rs:570` or `stream.next().await` at `test-api/src/file_transfer.rs:621`. The Tauri client builder also has no explicit transfer timeout at `src-tauri/src/main.rs:1158`.

In normal flowing downloads this is acceptable cooperative cancellation, and the partial-file preservation path is covered. If the remote server or socket stalls, however, the frontend can report that cancel was requested while the original `download_result` command remains pending until the network future wakes. Follow-up: add a wakeable cancellation primitive, race request/body awaits with cancellation, or apply a bounded transfer timeout.

### Medium - Some synchronous filesystem work remains in the Tauri async download path

The GUI download command no longer uses blocking HTTP or a nested runtime, but it still runs synchronous filesystem calls inside the async Tauri path: `fs::create_dir_all` before downloading at `src-tauri/src/main.rs:2779`, the duplicate synchronous resume probe at `src-tauri/src/main.rs:2827`, and the sync helper internals at `test-api/src/file_transfer.rs:854`, `test-api/src/file_transfer.rs:863`, and `test-api/src/file_transfer.rs:874`. The async helper also uses a synchronous `dest_path.exists()` check at `test-api/src/file_transfer.rs:673`.

This is not a correctness blocker for local output directories, but slow or remote filesystems can still block an async command worker. Follow-up: remove the duplicate preflight resume probe or make it async, use `tokio::fs::create_dir_all`, and replace `Path::exists` with async existence/removal handling.

### Low - Multi-file progress can briefly make an active download look idle

`download_file_async` emits `done: true` for each individual file at `test-api/src/file_transfer.rs:687`, and the Tauri adapter forwards that payload for every file at `src-tauri/src/main.rs:2841`. The frontend listener treats any `done` download progress payload as task-level completion at `src/main.ts:181`. Since the card disables Download and shows Cancel only while `phase === 'downloading'` at `src/app/render/cards.ts:83`, a multi-file download can briefly render as done between files even though the backend command is still running.

The latest per-hash running guard prevents an actual duplicate command in that window at `src/app/services/tasks.ts:104`, so this is an ergonomics issue rather than a data race. Follow-up: keep task phase as `downloading` until the `download_result` promise resolves, or include an aggregate completion signal separate from per-file completion.

### Low - Upload cancellation is library-level only

The extracted upload helper accepts a cancel token at `test-api/src/file_transfer.rs:333`, but the Tauri upload path passes `None` at `src-tauri/src/main.rs:2520`, and the command list exposes only `cancel_download` at `src-tauri/src/main.rs:3042`. If upload cancellation is intentionally deferred, record that explicitly so later batches do not assume GUI uploads are cancellable end to end.

## Scope reviewed

- Batch: `transfer`.
- Role: `async_ergonomics_reviewer`.
- Scope: non-blocking behavior, async API shape, cancellation semantics, duplicate-download dedupe, progress callback ergonomics, runtime nesting risk, and blocking I/O in Tauri async paths.
- Out of scope: behavior parity beyond async ergonomics, structured error tracing, data/schema review, Rust style beyond async impact, and frontend visual design.
- Boundary note: the working diff contains DB/config/CLI changes outside the transfer minimum boundary. This pass inspected them only where they affect transfer async behavior, tests, or runtime usage.

## Files or interfaces inspected

- `docs/INDEX.md`
- `docs/architecture/backend-rewrite.md`
- `RESOURCES.md`
- `manifest/rewrite-status.yaml`
- `rewrite-records/README.md`
- `reviews/README.md`
- `docs/references/high-confidence-sources.md`
- `src-tauri/Cargo.toml`
- `test-api/Cargo.toml`
- `package.json`
- `src-tauri/src/main.rs`
- `test-api/src/file_transfer.rs`
- `test-api/src/main.rs`
- `src/app/services/tasks.ts`
- `src/app/contracts/app-context.ts`
- `src/app/render/cards.ts`
- `src/app/controllers/dom-events.ts`
- `src/main.ts`
- Relevant tests in `src-tauri/src/main.rs`, `test-api/src/file_transfer.rs`, `test-api/tests/db_integration.rs`, and `test-api/tests/proxy_integration.rs`
- Official Tauri 2 command documentation listed by the project source index: https://v2.tauri.app/develop/calling-rust/

## Confirmations

- The duplicate-download race from the previous failed report is resolved. `register_download_token` rejects a second active token for the same hash at `src-tauri/src/main.rs:2350`, and `unregister_download_token` only removes the map entry when the stored token pointer matches the finishing transfer at `src-tauri/src/main.rs:2366`.
- The frontend has a per-hash running guard in `downloadTask` via `download:${hash}` at `src/app/services/tasks.ts:104`, and releases it in `finally` at `src/app/services/tasks.ts:169`.
- The download button disables while the task is in the `downloading` phase at `src/app/render/cards.ts:83`.
- `download_cancellation_registry_rejects_duplicate_hashes` covers the backend registry invariant: duplicate registration fails, cancel flips only the first token, stale-token unregister does not remove the active entry, and matching-token unregister clears it at `src-tauri/src/main.rs:3213`.
- Tauri transfer commands use async reqwest clients. The GUI client is `reqwest::Client` at `src-tauri/src/main.rs:1128`; `reqwest::blocking` remains in CLI/test-api compatibility paths at `test-api/src/main.rs:129` and `test-api/src/file_transfer.rs:709`.
- No nested Tokio runtime is used by the Tauri transfer path. `Runtime::new().block_on(...)` remains only in the synchronous compatibility upload helper and tests at `test-api/src/file_transfer.rs:487`, while Tauri upload/download call `upload_file_async` and `download_file_async` directly at `src-tauri/src/main.rs:2515` and `src-tauri/src/main.rs:2841`.
- Progress callbacks remain transport-agnostic in `test-api/src/file_transfer.rs`; Tauri event emission stays in the adapter, preserving the existing `upload-progress` and `download-progress` event names.
- Polling still deduplicates in-flight status checks per task hash with `pollInFlightHashes` at `src/app/services/tasks.ts:33`.

## Tests or checks run

- `cargo test download_cancellation_registry_rejects_duplicate_hashes` from `src-tauri`: passed, 1 test.
- `cargo test file_transfer` from `test-api`: passed, 4 async transfer tests.
- `cargo test` from `test-api`: passed, including 4 library tests, 14 DB integration tests, and 1 doc test; 9 online proxy/API tests remained ignored.
- `cargo test` from `src-tauri`: passed, 16 tests.
- `rg` checks for `reqwest::blocking`, `Runtime::new`, `block_on`, async helper usage, cancellation registry use, progress events, and synchronous filesystem calls across the reviewed transfer paths.

## Residual risk

- There is no end-to-end Tauri command test that starts `download_result`, sends `cancel_download`, verifies progress event delivery, and asserts command completion.
- The duplicate-download test covers registry invariants, not a fully concurrent pair of `download_result` commands writing to the same output target.
- There is no stalled-response cancellation test covering cancellation while parked on request headers or body reads.
- Online proxy/API tests are intentionally ignored unless credentials and a proxy are configured.

## Promotion decision

pass-with-followups
