# Config Storage And Error Context

## Context

The `config_and_formats` review gate initially failed on storage ownership, SQLite upsert semantics, partial config saves and diagnostic context. The fixes established the pattern for moving app-owned settings and local catalog data behind the Tauri command facade.

## Decision or Lesson

- Store user-owned settings in `user_config.db`; keep `mvsep.db` for local API/cache/catalog data.
- Merge partial config saves with defaults and the current stored value before writing.
- Do not use SQLite `INSERT OR REPLACE` for rows referenced by foreign keys; use `ON CONFLICT DO UPDATE` when associations must survive.
- Log structured backend errors before `Result<_, String>` is flattened at the Tauri command edge.

## Applies To

- `config_and_formats`
- Future config or settings persistence work
- Future algorithm-cache format association writes
- Future command wrappers that convert `BackendError` into Tauri-compatible strings

## Does Not Imply

- This does not require frontend pages to know DB table shapes.
- This does not make `mvsep.db` the owner of tokens, proxy settings or output directory settings.
- This does not remove the need for separate review gates after production code changes.

## Follow-up

- Add an atomic config update path before introducing additional partial config writers or multi-window settings saves.
- Extend backend log redaction for whitespace-formatted JSON token fields before future batches log larger structured payloads.
