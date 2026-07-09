# Task Persistence Data And Algorithm Review

## Findings ordered by severity

No blocking data or algorithm findings.

## Scope reviewed

- Batch: `task_persistence`
- Role: `data_algorithm_reviewer`
- Scope: `tasks.db` schema fit, frontend DTO to DB row mapping, `output_files` JSON handling, transaction semantics, idempotence, history trim behavior, and localStorage migration conflict rules.
- Acceptance rule: `tasks.db` is canonical for migrated task/history data; legacy `localStorage` may seed an empty backend once but must not repeatedly overwrite backend state.

## Files or interfaces inspected

- `src-tauri/src/main.rs`
- `test-api/src/db/tasks_db.rs`
- `test-api/src/db/repositories.rs`
- `src/main.ts`
- `src/app/types.ts`

## Data checks

- `TaskInfo` now covers the full frontend `Task` shape, including secondary model fields, queue/message fields, phase, and download progress fields.
- `TaskHistoryRecord` covers retry-critical fields and output metadata.
- `output_files` writes frontend-compatible `string[]` JSON and reads both `string[]` rows and richer object rows with `local_path`, `url`, `remote_name`, `name`, or `file_name`.
- `complete_task` performs active task removal, history insert/upsert, and history trim in one SQLite transaction.
- History upsert is idempotent through `task_history.id`.
- History trim now uses `COALESCE(completed_at, created_at)` so nullable completion times follow frontend ordering semantics.
- Relative path resolution is centralized through injected `BackendPaths::app_data_dir`.

## Tests or checks run

- `cd src-tauri && cargo test`: passed, including restart restore, history upsert, retry roundtrip, atomic completion, rich `output_files` adapter, and backend path resolution tests.
- `cd test-api && cargo test`: passed.
- `cd src-tauri && cargo clippy --all-targets -- -D warnings`: passed.
- `cd test-api && cargo clippy --all-targets -- -D warnings`: passed.
- `git diff --check`: passed.

## Residual risk

- `replace_active_tasks` remains full-list replacement. It is acceptable for current task counts but should be revisited when the frontend gateway batch introduces a narrower backend adapter.
- Existing terminal orphan rows in `tasks` are not proactively migrated into history. Normal `complete_task` removes terminal rows going forward.

## Promotion decision

pass-with-followups
