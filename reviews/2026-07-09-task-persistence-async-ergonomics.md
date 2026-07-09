# Task Persistence Async And Ergonomics Review

## Findings ordered by severity

No blocking async or ergonomics findings.

## Scope reviewed

- Batch: `task_persistence`
- Role: `async_ergonomics_reviewer`
- Scope: startup persistence flow, save queueing, terminal completion durability, delete/clear history durability, polling interaction, duplicate saves, and user-visible workflow.
- Acceptance rule: `tasks.db` is canonical; local fallback must not silently overwrite canonical backend state after backend load failures.

## Async and workflow checks

- Active task saves are queued, and normal in-progress polling keeps the debounce path.
- Critical paths await persistence:
  - new task creation calls forced active save
  - terminal polling awaits `addToHistory`
  - download completion awaits `addToHistory`
  - task delete awaits active save
  - history delete/clear awaits backend replacement and rolls back local memory on failure
- Terminal completion uses backend `complete_task`. The frontend only prunes active tasks after completion persistence succeeds.
- If completion persistence fails, the task is kept/reinserted as active so canonical state can be retried.
- Backend load failure marks local fallback as read-only for later backend writes, preventing stale local snapshots from becoming canonical.
- Duplicate downloads are guarded by frontend action state and backend cancellation registry.

## Tests or checks run

- `npm run build`: passed.
- `cd src-tauri && cargo test`: passed.
- `cd src-tauri && cargo clippy --all-targets -- -D warnings`: passed.
- `git diff --check`: passed.

## Residual risk

- Persistence failures are logged and protected, but not yet shown as first-class user-facing status. This should be handled in the frontend gateway/UI polish batch.
- Startup still waits for task/history backend commands before the first render; acceptable for this batch but a gateway-level ergonomics target.

## Promotion decision

pass-with-followups
