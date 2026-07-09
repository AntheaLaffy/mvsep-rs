# Task Persistence Behavior Review

## Findings ordered by severity

No blocking behavior findings.

## Scope reviewed

- Batch: `task_persistence`
- Role: `behavior_reviewer`
- Scope: public `Task` / `TaskHistoryRecord` payload compatibility, Tauri command names and argument shapes, restart restore, history idempotence, retry fields, and canonical backend conflict behavior.
- Acceptance rule: for migrated domains, `tasks.db` is canonical. `localStorage` is migration/fallback only. When backend and legacy frontend storage conflict for the same task/history identity, backend wins.

## Files or interfaces inspected

- `src/main.ts`
- `src/app/types.ts`
- `src/app/services/tasks.ts`
- `src/app/contracts/app-context.ts`
- `src-tauri/src/main.rs`
- `manifest/rewrite-status.yaml`
- `Note.md`

## Compatibility checks

- `Task` remains frontend-compatible snake_case.
- `TaskHistoryRecord` remains frontend-compatible camelCase through Rust `#[serde(rename_all = "camelCase")]`.
- New Tauri commands are registered and invoked with matching payloads:
  - `get_tasks`
  - `replace_active_tasks`
  - `get_task_history`
  - `save_task_history`
  - `complete_task`
- Restart restore reads from `tasks.db`; `localStorage` import is one-time only when backend data is empty and migration marker is absent.
- Retry from history preserves algorithm id, opt1/opt2/opt3, and format id.
- Download completion writes history by stable task hash id and uses backend `complete_task` for active-task removal plus history insert.
- Relative output paths now resolve under injected Tauri `app_data_dir`, not process cwd.

## Tests or checks run

- `cd src-tauri && cargo test`: passed, 23 tests.
- `cd src-tauri && cargo clippy --all-targets -- -D warnings`: passed.
- `npm run build`: passed.
- `git diff --check`: passed.

## Residual risk

- Persistence failures are currently logged rather than surfaced prominently in the UI. The canonical backend row is preserved on completion persistence failure, so this is an ergonomics follow-up rather than a behavior blocker.
- Startup still awaits task/history backend loads before first render.

## Promotion decision

pass-with-followups
