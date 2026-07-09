# config_and_formats behavior review

## Findings ordered by severity

No blocking behavior-parity or public-contract findings.

- `Config` payload shape remains stable between Rust and TypeScript: Rust exposes `token`, `api_url`, `mirror`, `proxy_mode`, `proxy_host`, `proxy_port`, `output_dir`, `output_format`, `poll_interval`, and `algorithm_auto_refresh_days` at `src-tauri/src/main.rs:25`; TypeScript expects the same fields at `src/app/types.ts:1`. Defaults also match the legacy command defaults in `src-tauri/src/main.rs:39` and the pre-batch implementation.
- `OutputFormat` payload shape remains frontend-only `{ id, name }`: Rust exposes only those fields at `src-tauri/src/main.rs:737`, TypeScript expects only those fields at `src/app/types.ts:63`, and the DB-only fields `bits_per_sample`, `extension`, and `is_premium` stay in `test-api/src/db/repositories.rs:113` rather than the Tauri payload mapper at `src-tauri/src/main.rs:398`.
- Tauri command names and public argument names remain stable. `load_config`, `save_config`, and `list_formats` are still registered in `tauri::generate_handler!` at `src-tauri/src/main.rs:3095`; the wrappers keep `save_config(config)` at `src-tauri/src/main.rs:1364` and `list_formats(api_url, token)` at `src-tauri/src/main.rs:1476`. Frontend call sites still invoke `save_config` with `{ config }` at `src/main.ts:1029` and `list_formats` with `{ apiUrl, token }` at `src/main.ts:1186`. This matches Tauri 2's documented JS camelCase argument convention for snake_case Rust parameters.
- Settings autosave behavior is preserved at the public boundary. Frontend input/change handlers still schedule the debounced autosave at `src/app/controllers/dom-events.ts:260`, `src/app/controllers/dom-events.ts:328`, and `src/app/controllers/dom-events.ts:333`; the debounced path still calls `save_config` through `saveConfig` at `src/main.ts:439` and `src/main.ts:450`.
- Legacy JSON import is idempotent from the command path. `load_config_from_backend_store` reads the DB first at `src-tauri/src/main.rs:354`, imports legacy `config.json` only when no DB row exists at `src-tauri/src/main.rs:368`, and writes that row at `src-tauri/src/main.rs:370`. The added test covers a stale legacy JSON file no longer overriding the DB value at `src-tauri/src/main.rs:3269`.
- Injected path behavior is present for the migrated storage. Tauri setup injects app config/data directories at `src-tauri/src/main.rs:3059`, `BackendPaths` derives `mvsep.db` under injected `app_data_dir` at `src-tauri/src/main.rs:129`, and the DB open path uses that injected path at `src-tauri/src/main.rs:300`.
- Format list expectations match the current UI fallback list. The frontend fallback defines six formats at `src/main.ts:1172`; the DB defaults define the same ids and names at `test-api/src/db/repositories.rs:531`; the Tauri mapper returns only `id` and `name` at `src-tauri/src/main.rs:398`.

## Scope reviewed

Batch: `config_and_formats`

Role: `behavior_reviewer`

Reviewed public behavior and compatibility for config persistence, output format listing, command names, argument names, TypeScript/Rust payload shapes, settings autosave, legacy JSON import idempotency, default values, injected paths, and DB-only format-field leakage.

## Files or interfaces inspected

- `docs/INDEX.md`
- `docs/architecture/backend-rewrite.md`
- `RESOURCES.md`
- `manifest/rewrite-status.yaml`
- `rewrite-records/README.md`
- `reviews/README.md`
- `docs/references/high-confidence-sources.md`
- `src-tauri/src/main.rs`
- `src-tauri/Cargo.toml`
- `src-tauri/Cargo.lock`
- `src/app/types.ts`
- `src/main.ts`
- `src/app/controllers/dom-events.ts`
- `src/app/render/settings.ts`
- `test-api/src/db/repositories.rs`
- `test-api/src/db/migrations.rs`
- `test-api/tests/db_integration.rs`
- Pre-batch `HEAD:src-tauri/src/main.rs` for legacy command/config/format behavior.
- Official Tauri 2 command documentation: https://v2.tauri.app/develop/calling-rust/

Boundary check: the production diff is limited to `src-tauri/Cargo.toml`, `src-tauri/Cargo.lock`, and `src-tauri/src/main.rs`. The batch keeps the accepted Tauri command facade architecture and does not import py2rs runtime architecture.

## Tests or checks run

- `git status --short`
- `git diff --stat`
- `git diff --name-only`
- `rg -n "struct Config|interface Config|type Config|OutputFormat|serde\\(|rename_all|tauri::command|invoke\\(|save_config|load_config|get_output_formats|import|config|settings" src src-tauri test-api tests docs manifest reviews -g '!target' -g '!node_modules'` failed only because `tests/` does not exist; reran narrower `rg` commands over existing paths.
- `cargo test -p mvsep-gui config_store_imports_legacy_json_once output_formats_store_preserves_frontend_shape` failed only because Cargo accepts one test-name filter, not two.
- `cargo test -p mvsep-gui output_formats_store_preserves_frontend_shape` passed.
- `cargo test -p mvsep-gui config_store_imports_legacy_json_once` passed.
- `cargo test -p mvsep-gui` passed: 6 tests.
- `cd test-api && cargo test` passed: 11 DB integration tests, 9 ignored proxy integration tests, 1 doctest. Existing warning noise remains in `test-api`.
- `npm run build` passed.

## Residual risk

- I did not run live online MVSep API checks, so this review does not prove the remote output-format list is permanently identical to the local DB defaults. The current batch contract and UI fallback both expect the six local defaults.
- The lower-level SQLite schema has column defaults of its own, but the Tauri command path seeds the DB from `Config::default()` on first load. A manually inserted partial config row could surface null or schema-level values; that is outside normal frontend autosave/import flow and better suited to the data/schema review.

## Promotion decision: pass

The `behavior_reviewer` gate passes for `config_and_formats`.
