# Behavior Review: algorithm_cache

## Findings ordered by severity

No blocking behavior-parity findings.

- Low: `fetch_latest_algorithm_info` still exposes `proxy_mode`, `proxy_host`, and `proxy_port`, but the current implementation discards those command arguments before making the remote request. The Tauri command signature still accepts the arguments at [src-tauri/src/main.rs](/home/fuurin/code/mvsep-rs/src-tauri/src/main.rs:1621), and the frontend still sends camelCase args at [src/main.ts](/home/fuurin/code/mvsep-rs/src/main.ts:1148), but `legacy_fetch_latest_algorithm_info` drops them at [src-tauri/src/main.rs](/home/fuurin/code/mvsep-rs/src-tauri/src/main.rs:2087). The actual HTTP client reads proxy settings from `state.config` instead at [src-tauri/src/main.rs](/home/fuurin/code/mvsep-rs/src-tauri/src/main.rs:1118). This is not blocking for the current frontend path when config has already been saved into state, but direct command callers can observe a behavior difference from the old CLI-assisted path.

- Low: corrupted-cache behavior is not proven for the new DB-backed cache. The manifest asks for missing, corrupted, and refreshed scenarios at [manifest/rewrite-status.yaml](/home/fuurin/code/mvsep-rs/manifest/rewrite-status.yaml:73). Missing DB behavior is covered and returns an empty successful list at [src-tauri/src/main.rs](/home/fuurin/code/mvsep-rs/src-tauri/src/main.rs:3459), but local refresh now opens `mvsep.db` and propagates DB errors through the Tauri edge at [src-tauri/src/main.rs](/home/fuurin/code/mvsep-rs/src-tauri/src/main.rs:615) and [src-tauri/src/main.rs](/home/fuurin/code/mvsep-rs/src-tauri/src/main.rs:751). The old JSON cache path recovered parse-corrupt cache files to an empty cache; add a DB-corruption fixture or explicitly document the changed behavior before relying on this as a full corrupted-cache parity gate.

## Scope reviewed

Role: `behavior_reviewer`.

Batch: `algorithm_cache`.

Theme: public behavior parity and frontend payload compatibility for:

- `refresh_algorithm_list_from_local`
- `get_algorithm_details_from_local`
- `fetch_latest_algorithm_info`
- `get_algorithm_cache_path_cmd`

## Files or interfaces inspected

- Tauri command registration and signatures: [src-tauri/src/main.rs](/home/fuurin/code/mvsep-rs/src-tauri/src/main.rs:1598), [src-tauri/src/main.rs](/home/fuurin/code/mvsep-rs/src-tauri/src/main.rs:3157)
- Backend adapter methods: [src-tauri/src/main.rs](/home/fuurin/code/mvsep-rs/src-tauri/src/main.rs:707)
- Algorithm cache DB adapter: [src-tauri/src/main.rs](/home/fuurin/code/mvsep-rs/src-tauri/src/main.rs:475), [src-tauri/src/main.rs](/home/fuurin/code/mvsep-rs/src-tauri/src/main.rs:612)
- Public Rust payload structs: [src-tauri/src/main.rs](/home/fuurin/code/mvsep-rs/src-tauri/src/main.rs:1018), [src-tauri/src/main.rs](/home/fuurin/code/mvsep-rs/src-tauri/src/main.rs:2240)
- TypeScript payload contracts: [src/app/types.ts](/home/fuurin/code/mvsep-rs/src/app/types.ts:68)
- Frontend command callers and consumers: [src/main.ts](/home/fuurin/code/mvsep-rs/src/main.ts:1020), [src/main.ts](/home/fuurin/code/mvsep-rs/src/main.ts:1069), [src/main.ts](/home/fuurin/code/mvsep-rs/src/main.ts:1129), [src/main.ts](/home/fuurin/code/mvsep-rs/src/main.ts:1203)
- Algorithm page and settings rendering: [src/app/render/algorithms.ts](/home/fuurin/code/mvsep-rs/src/app/render/algorithms.ts:11), [src/app/render/settings.ts](/home/fuurin/code/mvsep-rs/src/app/render/settings.ts:154)
- Existing algorithm cache tests: [src-tauri/src/main.rs](/home/fuurin/code/mvsep-rs/src-tauri/src/main.rs:3459)
- Official Tauri 2 command docs for invoke argument mapping: https://v2.tauri.app/develop/calling-rust/

## Tests or checks run

- `cd src-tauri && cargo test algorithm_cache_` passed.
- `cd src-tauri && cargo test algorithm_cache_db_store_preserves_frontend_payload_shape` passed.
- `cd src-tauri && cargo test algorithm_cache_missing_db_returns_empty_list` passed.
- `cd src-tauri && cargo test algorithm_cache_db_refresh_replaces_stale_algorithms` passed.
- `git diff --check` passed.

The Rust test commands emitted the known existing `test-api` warning debt for `futures_util::StreamExt` and `SCHEMA_VERSION`; I did not treat that as a behavior failure.

Writer-reported full checks were not rerun in this role: `npm run build`, full `src-tauri` tests, full `src-tauri` clippy, and full `test-api` tests.

## Residual risk

The frontend-visible shapes are preserved: list responses still provide `updated_at`, `groups`, and `total_algorithms`; algorithms expose `id`, `name`, and `group_id`; details expose `id`, `name`, and filtered `fields`; field options are still `Record<string, string>`. The new tests cover grouped input flattening, unsupported field filtering, non-string option values, stale algorithm replacement, missing DB success, and the new DB path returned by `get_algorithm_cache_path_cmd`.

Remaining risk is mostly around edge cases not covered by current behavior fixtures: direct callers passing proxy arguments that differ from saved state, DB-corruption recovery, and live remote payload variants beyond the fixture shapes already exercised.

## Promotion decision: `pass-with-followups`

Promote from the behavior gate once the other required reviewers pass. The follow-ups above are public-edge polish and fixture coverage issues, not blockers for the current frontend payload contract.
