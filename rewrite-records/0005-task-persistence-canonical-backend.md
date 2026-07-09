# Context

The task persistence batch moves active task and task history state from frontend-only `localStorage` into `tasks.db` behind the Tauri backend facade.

# Decision or Lesson

For migrated domains, the rewritten backend store is canonical. Legacy frontend storage is only a one-time migration source and fallback display source. It must not repeatedly write stale local snapshots back into the backend after the backend has state or after backend reads fail.

Terminal task completion should be one backend operation. The GUI now uses `complete_task` so active task removal, history upsert and history trimming happen in one SQLite transaction.

Relative GUI paths should resolve from injected Tauri backend paths. Relative download/output paths now resolve under `BackendPaths::app_data_dir`, not process cwd.

# Applies To

- Task restore and history restore in `src/main.ts`
- Task/history commands in `src-tauri/src/main.rs`
- Future frontend gateway work that centralizes Tauri calls
- Any future backend storage migration that replaces frontend-local state

# Does Not Imply

- This does not make `localStorage` authoritative after a domain is migrated.
- This does not require preserving old frontend conflict behavior when it disagrees with canonical backend state.
- This does not make upload temporary paths the base for downloaded outputs; download output paths use configured output directories resolved against the injected GUI app data dir when relative.

# Follow-up

Surface persistence failures in the UI during the frontend gateway/UI batch instead of only logging them.
