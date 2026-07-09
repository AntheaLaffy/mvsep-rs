# Transfer Behavior Review

## Findings ordered by severity

No behavior parity findings.

The reviewed transfer public surface remains compatible for the Tauri command names, frontend invoke arguments, progress event names, progress payload fields, upload hash extraction, download resume/restart/cancel behavior, frontend-recognizable cancellation text, and the intentional redaction of token-like error payloads.

## Scope reviewed

- Batch: `transfer`
- Role: `behavior_reviewer`
- Scope: public behavior parity and command/event/payload compatibility after the latest transfer fixes.
- Out of scope: async implementation quality, structured error tracing quality beyond public payload compatibility, Rust style, data-schema review, and frontend UX polish.
- Boundary note: the current worktree diff is broader than transfer because earlier rewrite work remains uncommitted. This pass reviewed the transfer-visible command, event, payload, upload, download, cancellation, and redaction surfaces only.

## Files or interfaces inspected

- `docs/INDEX.md`
- `docs/architecture/backend-rewrite.md`
- `RESOURCES.md`
- `manifest/rewrite-status.yaml`
- `rewrite-records/README.md`
- `reviews/README.md`
- `docs/references/high-confidence-sources.md`
- `package.json`
- `src-tauri/Cargo.toml`
- `test-api/Cargo.toml`
- `src-tauri/capabilities/default.json`
- `doc/mvsep_api_endpoints.md`
- `src-tauri/src/main.rs`
- `test-api/src/file_transfer.rs`
- `src/main.ts`
- `src/app/types.ts`
- `src/app/services/tasks.ts`
- `src/app/contracts/app-context.ts`
- Relevant transfer and payload tests in `src-tauri/src/main.rs` and `test-api/src/file_transfer.rs`
- Historical `HEAD` versions of `src-tauri/src/main.rs` and `test-api/src/file_transfer.rs` for transfer parity comparison
- Official Tauri 2 command reference checked: https://v2.tauri.app/develop/calling-rust/

## Compatibility checks

- Command names remain registered as `create_task`, `download_result`, and `cancel_download` in the Tauri handler at `src-tauri/src/main.rs:3031`, `src-tauri/src/main.rs:3033`, and `src-tauri/src/main.rs:3034`.
- `cancel_download` still accepts only `hash` and returns `Result<(), String>` at `src-tauri/src/main.rs:1713`. The frontend still invokes it as `{ hash }` at `src/app/services/tasks.ts:175`.
- `create_task` still accepts `file_path`, `sep_type`, `opt1`, `opt2`, `opt3`, `output_format`, `demo`, `api_url`, and `token` at `src-tauri/src/main.rs:1747`. The frontend still invokes the same Tauri JS contract using camelCase argument keys at `src/main.ts:1360`.
- `download_result` still accepts `hash`, `output_dir`, `file_index`, `original_file_name`, `api_url`, and `token`, and returns `Vec<String>` at `src-tauri/src/main.rs:1797`. The frontend still invokes it with `hash`, `outputDir`, `fileIndex`, `originalFileName`, `apiUrl`, and `token` at `src/app/services/tasks.ts:119`.
- Progress event names remain stable:
  - Upload progress emits `upload-progress` during upload and on failed upload conversion at `src-tauri/src/main.rs:2526` and `src-tauri/src/main.rs:2552`.
  - Download progress emits `download-progress` at `src-tauri/src/main.rs:2849`.
  - The frontend listens to `download-progress` and `upload-progress` at `src/main.ts:182` and `src/main.ts:199`.
- Progress payload field names remain stable:
  - Rust `DownloadProgressPayload` fields are `hash`, `file_name`, `downloaded_bytes`, `total_bytes`, `speed_bps`, `percent`, and `done` at `src-tauri/src/main.rs:1080`.
  - Rust `UploadProgressPayload` fields are `file_name`, `uploaded_bytes`, `total_bytes`, `speed_bps`, `percent`, `done`, and `failed` at `src-tauri/src/main.rs:1090`.
  - TypeScript payload interfaces match those field names at `src/app/types.ts:116` and `src/app/types.ts:126`.
  - The payload key test covers the serialized event field names at `src-tauri/src/main.rs:3099`.
- Upload hash extraction preserves and broadens the accepted response shapes:
  - The extractor accepts `hash`, `data.hash`, and `task_hash` at `test-api/src/file_transfer.rs:281`.
  - The local endpoint note documents the `data.hash` response shape at `doc/mvsep_api_endpoints.md:87`.
  - The mock HTTP test proves `data.hash` extraction at `test-api/src/file_transfer.rs:952`.
- Download resume/restart/cancel behavior matches the transfer policy:
  - Existing `.part` state is read before download at `test-api/src/file_transfer.rs:548`.
  - Resumable downloads send the `Range` header at `test-api/src/file_transfer.rs:565`.
  - `206 Partial Content` appends to the partial file, while successful non-206 responses restart from scratch at `test-api/src/file_transfer.rs:576`.
  - Cancellation before or during streaming returns `Download cancelled` and leaves resumable partial state in place at `test-api/src/file_transfer.rs:556` and `test-api/src/file_transfer.rs:652`.
  - Final success renames the `.part` file and removes metadata at `test-api/src/file_transfer.rs:673`.
  - Tests cover upload hash extraction, Range resume, Range rejection with full restart, and cancellation preserving partial files at `test-api/src/file_transfer.rs:952`, `test-api/src/file_transfer.rs:988`, `test-api/src/file_transfer.rs:1033`, and `test-api/src/file_transfer.rs:1071`.
- Cancellation remains frontend-recognizable:
  - The Tauri download command returns `Download cancelled` before a file starts when cancellation is already requested at `src-tauri/src/main.rs:2787`.
  - The async transfer layer returns the same phrase before request start and mid-stream at `test-api/src/file_transfer.rs:560` and `test-api/src/file_transfer.rs:661`.
  - The frontend recognizes `download cancelled`, `canceled`, and `cancelled` at `src/app/services/tasks.ts:5`.
- Token-like error payload redaction is intentional and covered:
  - `to_tauri_result` logs structured context and returns only the redacted UI-facing message at `src-tauri/src/main.rs:1543`.
  - The redaction markers cover token, api_token, authorization, bearer, password, and secret-like forms at `src-tauri/src/main.rs:1501`.
  - Tests assert token-like data is removed from both Tauri error payloads and backend logs at `src-tauri/src/main.rs:3156` and `src-tauri/src/main.rs:3205`.

## Tests or checks run

- `cargo test file_transfer` from `test-api`: passed, 4 transfer tests passed.
- `cargo test progress_event_payload_field_names_stay_stable` from `src-tauri`: passed.
- `cargo test tauri_error_payload_redacts_transfer_tokens` from `src-tauri`: passed.
- `cargo test transfer_backend_error_preserves_status_url_hash_and_path` from `src-tauri`: passed.
- `cargo test backend_error_keeps_context_until_tauri_edge` from `src-tauri`: passed.
- `cargo test backend_logs_redact_token_like_values` from `src-tauri`: passed.
- `./node_modules/.bin/tsc --noEmit`: passed.
- `git diff --check`: passed.

## Residual risk

- The Range/resume/restart/cancel tests exercise `test-api/src/file_transfer.rs` directly with mock HTTP. They do not run an end-to-end Tauri command fixture that queries `data.files`, downloads through `download_result`, and observes real Tauri event delivery.
- The endpoint note shows completed files with `link` fields at `doc/mvsep_api_endpoints.md:291`, while the current and historical Tauri download path expects `url` at `src-tauri/src/main.rs:2802`. This is not a transfer parity regression, but remains a live remote-payload compatibility risk if MVSep does not also return `url`.
- Broad release gates were not run in this behavior role. This pass ran targeted transfer behavior checks only.

## Promotion decision

pass
