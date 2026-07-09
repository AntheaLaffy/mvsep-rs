# algorithm_cache data/algorithm review

## Findings ordered by severity

1. Medium follow-up: cache row replacement and cache metadata remain a two-phase write across `mvsep.db` and `user_config.db`. `replace_algorithm_cache_in_backend_store` commits the algorithm rows through `mvsep.db` first (`src-tauri/src/main.rs:604`, `src-tauri/src/main.rs:616`), then `fetch_latest_algorithm_info` writes `algorithm_cache_updated_at` / `algorithm_last_fetched_at` through `user_config.db` (`src-tauri/src/main.rs:572`, `src-tauri/src/main.rs:583`, `src-tauri/src/main.rs:2128`, `src-tauri/src/main.rs:2137`). If metadata write fails, the current rows are still visible with stale or empty metadata. This is not a blocker for the prior foreign-key/data-integrity issue, but it should be documented or moved into the cache DB if exact refresh atomicity becomes a product requirement.

No blocking data/schema/algorithm finding remains.

The prior blocking finding is fixed. The schema now has `algorithms.is_cached` on fresh databases and v3 migrations (`test-api/src/db/migrations.rs:40`, `test-api/src/db/migrations.rs:46`, `test-api/src/db/migrations.rs:154`). `tasks` and `presets` still keep foreign keys to `algorithms` (`test-api/src/db/migrations.rs:65`, `test-api/src/db/migrations.rs:91`, `test-api/src/db/migrations.rs:115`, `test-api/src/db/migrations.rs:124`), but `replace_algorithm_cache` now marks all algorithms stale, upserts the current cache as `is_cached = 1`, and deletes only stale algorithms that are not referenced by `tasks` or `presets` (`test-api/src/db/repositories.rs:193`, `test-api/src/db/repositories.rs:200`, `test-api/src/db/repositories.rs:207`, `test-api/src/db/repositories.rs:218`). Current cache reads filter to cached rows (`test-api/src/db/repositories.rs:234`, `test-api/src/db/repositories.rs:253`, `test-api/src/db/repositories.rs:319`), and default output-format associations are rebuilt only for cached algorithms (`test-api/src/db/repositories.rs:756`, `test-api/src/db/repositories.rs:761`). The regression test inserts both a task and preset referencing a stale algorithm, reruns replacement, proves the old row remains with `is_cached = 0`, proves current reads hide it, and proves old format associations are empty while the new algorithm receives defaults (`test-api/tests/db_integration.rs:500`, `test-api/tests/db_integration.rs:522`, `test-api/tests/db_integration.rs:528`, `test-api/tests/db_integration.rs:535`, `test-api/tests/db_integration.rs:552`, `test-api/tests/db_integration.rs:564`, `test-api/tests/db_integration.rs:571`, `test-api/tests/db_integration.rs:575`).

The previous field-ID collision follow-up is also addressed for the Tauri cache adapter. Algorithm fields now use a synthetic per-algorithm negative ID derived from the algorithm id and supported field order, instead of trusting remote `algorithm_fields[].id` (`src-tauri/src/main.rs:512`, `src-tauri/src/main.rs:522`). The regression test feeds two algorithms whose remote field ids both equal `1` and verifies both detail payloads retain their separate field text and options (`src-tauri/src/main.rs:3630`, `src-tauri/src/main.rs:3640`, `src-tauri/src/main.rs:3653`, `src-tauri/src/main.rs:3661`, `src-tauri/src/main.rs:3672`).

No complexity concern was found in the local cache replacement or read path. Replacement and list/detail reads remain linear in groups, current algorithms, fields, and default format associations, which is appropriate for a local algorithm catalog.

## Scope reviewed

Batch: `algorithm_cache`

Role: `data_algorithm_reviewer`

This re-review covered schema compatibility, stale cache replacement, foreign-key safety with existing `tasks` and `presets`, cached-vs-stale read filtering, output-format association rebuilding, field identity, and local algorithm-cache complexity. It did not review behavior parity, error tracing, async ergonomics, frontend UX, Rust style, production code outside the data/schema/algorithm path, or manifest promotion.

Boundary check: the implementation stays behind the accepted Tauri command / `AppBackend` seam and reuses the `test-api` DB repository. It does not introduce py2rs runtime architecture or frontend protocol changes.

## Files or interfaces inspected

- `docs/INDEX.md`
- `docs/architecture/backend-rewrite.md`
- `RESOURCES.md`
- `manifest/rewrite-status.yaml`
- `rewrite-records/README.md`
- `reviews/README.md`
- `reviews/2026-07-09-algorithm-cache-data-algorithm.md`
- `src-tauri/src/main.rs`
- `test-api/src/db/migrations.rs`
- `test-api/src/db/repositories.rs`
- `test-api/tests/db_integration.rs`

## Tests or checks run

- `git status --short`
- `rg -n "is_cached|replace_algorithm_cache|mark.*stale|stale|cached|set_algorithm_output_formats|get_all_algorithms|get_algorithm_by_id|get_algorithm_fields|get_algorithm_details|algorithm_cache_field_ids|preserves_referenced_stale|AlgorithmField" test-api/src/db test-api/tests/db_integration.rs src-tauri/src/main.rs`
- `cargo test --test db_integration replace_algorithm_cache -- --nocapture` from `test-api`: passed 2 tests. Existing `test-api` warning debt was emitted.
- `cargo test algorithm_cache_field_ids_are_scoped_per_algorithm -- --nocapture` from `src-tauri`: passed 1 test.
- `git diff --check`: passed.

Writer-reported checks noted but not rerun in full for this focused re-review: `cd src-tauri && cargo test` passed 13 tests; `cd test-api && cargo test` passed 14 DB integration tests plus doctest with proxy tests ignored; `npm run build` passed; `cd src-tauri && cargo clippy --all-targets -- -D warnings` passed; `cd test-api && cargo clippy --all-targets -- -D warnings` still fails on existing bin/library warning debt.

## Residual risk

I did not run online MVSep API checks or proxy-dependent ignored tests. This review uses local schema, repository code, Tauri cache adapter fixtures, and DB integration tests as the source of truth.

Stale referenced algorithms intentionally remain in `algorithms` with `is_cached = 0` to preserve foreign-key integrity for tasks and presets. That is now consistent with current read filters, but future task/preset screens that join to `algorithms` should be explicit about whether they need historical stale rows or only current cache rows.

## Promotion decision: pass-with-followups
