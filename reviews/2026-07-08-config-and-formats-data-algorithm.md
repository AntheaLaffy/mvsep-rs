# config_and_formats data/algorithm review

## Findings ordered by severity

1. Low: config partial-save merging is correct for sequential calls, but still not atomic across concurrent writers. `save_config_to_backend_store` reads the current JSON value through a freshly opened `UserConfigDB`, merges defaults/current/patch, then writes through `save_config_to_user_config_db`, which opens another `UserConfigDB` handle (`src-tauri/src/main.rs:364`, `src-tauri/src/main.rs:399`). Two simultaneous partial saves can both merge against the same old value and the later write can lose the earlier patch. The current frontend debounces settings autosave and sends a full `Config` object (`src/main.ts:439`, `src/main.ts:450`, `src/main.ts:1029`; `src/app/types.ts:1`), so this is not a promotion blocker for this batch, but future partial-update or multi-window callers should use one connection plus a transaction or an app-level config write lock.

No blocking data/schema findings remain. The prior failed findings were rechecked and are resolved:

- User config now persists to `user_config.db`, not `mvsep.db`: paths are split in `BackendPaths` (`src-tauri/src/main.rs:143`), config opens `UserConfigDB` at `src-tauri/src/main.rs:330`, and config load/save uses `get_json` / `set_json` on that database (`src-tauri/src/main.rs:369`, `src-tauri/src/main.rs:399`). The import test asserts `user_config.db` exists and `mvsep.db` does not after config-only import (`src-tauri/src/main.rs:3320`).
- Partial config saves now preserve existing/default values: `merge_config` keeps base values when the patch field is `None` (`src-tauri/src/main.rs:337`), and `save_config_to_backend_store` merges default, current, then patch before persisting (`src-tauri/src/main.rs:399`). The targeted regression test covers this path (`src-tauri/src/main.rs:3360`).
- Output-format upsert no longer deletes algorithm associations: `upsert_output_format` now uses `INSERT ... ON CONFLICT(id) DO UPDATE` instead of `INSERT OR REPLACE` (`test-api/src/db/repositories.rs:533`). The regression test verifies an algorithm-format association survives a format update (`test-api/tests/db_integration.rs:203`).

## Scope reviewed

Batch: `config_and_formats`

Role: `data_algorithm_reviewer`

Reviewed schema ownership, DB paths, config persistence, legacy config migration/idempotency, partial-save data preservation, output-format upsert behavior, data structures and algorithmic risk for this batch. This review did not edit production code or manifest state.

Boundary check: the batch stays on the accepted Tauri-command/AppBackend seam and uses `test-api` DB modules directly through a path dependency. It does not import py2rs runtime architecture. Config and output format ownership match the documented three-database split (`test-api/src/lib.rs:5`, `test-api/src/utils/paths.rs:3`).

## Files or interfaces inspected

- `docs/INDEX.md`
- `docs/architecture/backend-rewrite.md`
- `RESOURCES.md`
- `manifest/rewrite-status.yaml`
- `rewrite-records/README.md`
- `reviews/README.md`
- `reviews/2026-07-08-config-and-formats-data-algorithm.md` prior failed report
- `src-tauri/Cargo.toml`
- `src-tauri/src/main.rs`
- `test-api/src/lib.rs`
- `test-api/src/utils/paths.rs`
- `test-api/src/db/mod.rs`
- `test-api/src/db/migrations.rs`
- `test-api/src/db/repositories.rs`
- `test-api/src/db/user_config.rs`
- `test-api/tests/db_integration.rs`
- `src/main.ts`
- `src/app/types.ts`
- `src/app/controllers/dom-events.ts`

## Tests or checks run

- `git status --short`
- `git diff -- src-tauri/src/main.rs`
- `git diff -- test-api/src/db/repositories.rs`
- `git diff -- test-api/tests/db_integration.rs`
- `rg -n "INSERT OR REPLACE INTO output_formats|REPLACE INTO output_formats|save_config_to_backend_store|user_config_db_path|mvsep_db_path|set_json\\(\"config\"|get_json::<Config>" src-tauri/src/main.rs test-api/src/db test-api/tests/db_integration.rs`
- `cargo test config_store` from `src-tauri`: passed 2 tests.
- `cargo test output_formats_store_preserves_frontend_shape` from `src-tauri`: passed 1 test.
- `cargo test --test db_integration output_format` from `test-api`: passed 6 tests.
- `cargo test --test db_integration test_config` from `test-api`: passed 2 tests.
- `cargo test` from `src-tauri`: passed 8 tests.
- `cargo test --test db_integration` from `test-api`: passed 12 tests.
- `git diff --check`: passed.

The test-api checks still emit existing warning noise in unrelated code paths; no data/algorithm failure was observed.

## Residual risk

I did not run live MVSep API checks because this review is limited to local config/format persistence and schema behavior. The remaining concurrency point is documented above. It should be handled before adding additional partial config writers or multi-window settings saves, but the current debounced full-config frontend flow keeps this out of the critical path for `config_and_formats`.

No benchmark issue was found. This batch's migrated operations are constant-size config JSON reads/writes and six default output formats; the complexity risk is negligible.

## Promotion decision: pass-with-followups
