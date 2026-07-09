# Algorithm Cache Stale Rows And Error Context

## Context

The `algorithm_cache` batch moved local algorithm list/details from `algorithms_cache.json` into `mvsep.db`. Review found that a naive full-table delete breaks once task or preset rows reference algorithms, and that a single command can fail against either a remote endpoint or a local database.

## Decision or Lesson

- Treat algorithm-cache refresh as a current-cache view update, not as permission to delete every algorithm row.
- Preserve foreign-key integrity by marking stale algorithm rows with `is_cached = 0`; delete only stale rows that are not referenced by tasks or presets.
- Current algorithm-cache readers must filter to `is_cached = 1`.
- Use synthetic per-algorithm field ids for `add_opt1` / `add_opt2` / `add_opt3`; do not trust remote field ids to be globally unique.
- When one command touches both remote APIs and local storage, attach endpoint context only to remote failures and path context to local DB or metadata failures.

## Applies To

- `algorithm_cache`
- Future task/preset joins to algorithm rows
- Future cache-like tables that have foreign-key dependents
- Future Tauri commands that combine network fetches with local DB writes

## Does Not Imply

- This does not make stale algorithms visible in the current algorithm picker.
- This does not define exact per-algorithm output-format support; current behavior still associates all default formats with current algorithms.
- This does not make cross-database metadata writes atomic.

## Follow-up

- Decide whether cache metadata belongs in `mvsep.db` before exact refresh atomicity becomes product-visible.
- Add an explicit DB-corruption recovery policy for algorithm cache.
- Keep task/preset history screens explicit about whether they need stale historical algorithms or only current cache rows.
