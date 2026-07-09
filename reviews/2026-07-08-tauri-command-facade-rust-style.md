# Findings ordered by severity

No high or medium Rust style findings were identified for `tauri_command_facade` after the error-tracing writer patch.

## Low - Backend selection is still concrete rather than router-shaped

`AppState` stores `backend: LegacyMainBackend` directly, and `new_app_state` always installs that concrete adapter: `src-tauri/src/main.rs:119`, `src-tauri/src/main.rs:120`, `src-tauri/src/main.rs:2989`, `src-tauri/src/main.rs:2991`. The command wrappers still call through the `AppBackend` trait surface, and this batch intentionally routes every method to legacy code, so this does not block `tauri_command_facade`.

The follow-up risk is that the first mixed-owner batch will need to change this state shape before a new backend can sit beside or behind `LegacyMainBackend`. The current private trait also uses `async fn` directly: `src-tauri/src/main.rs:130`, `src-tauri/src/main.rs:135`, `src-tauri/src/main.rs:205`, `src-tauri/src/main.rs:226`. That is fine for current static dispatch, but it is not an object-safe backend slot. Before the first partial migration, introduce a small router enum or another explicit dispatch shape so command wrappers can remain stable while individual methods move.

## Low - Facade, error wrapper, command wrappers and legacy implementation remain in one large module

The new error/result surface is in `src-tauri/src/main.rs:60` through `src-tauri/src/main.rs:114`, the `AppBackend` trait is in `src-tauri/src/main.rs:130` through `src-tauri/src/main.rs:249`, the `LegacyMainBackend` impl is in `src-tauri/src/main.rs:260` through `src-tauri/src/main.rs:557`, the command facade wrappers are in `src-tauri/src/main.rs:1237` through `src-tauri/src/main.rs:1530`, and the renamed legacy implementation continues from `src-tauri/src/main.rs:1533` onward. `src-tauri/src/lib.rs:4` is still only a stub re-export file.

This is acceptable for the first facade batch because the previous backend was already concentrated in `main.rs`, and the production diff stayed inside the intended minimum boundary. The maintainability risk is that adding a second real backend in the same file would blur error, command, adapter and legacy responsibilities. Move the facade/error types and adapters into modules before the next backend-owner migration expands this file further.

# Scope reviewed

Batch: `tauri_command_facade`

Role: `rust_style_reviewer`

Re-review date: 2026-07-09

Focus: Rust module shape, ownership and borrowing, warning/clippy status, maintainability of `BackendError`/`BackendResult`, boxed context sizing, command wrapper style, tests, and whether the implementation still stays inside the accepted Tauri command facade architecture.

Boundary check: the production diff is still limited to `src-tauri/src/main.rs` by `git diff --name-only`. The batch keeps the accepted architecture of Tauri commands delegating to an `AppBackend` surface backed by `LegacyMainBackend`; it does not import py2rs runtime architecture, broaden Tauri capabilities, or change frontend command/event contracts.

# Files or interfaces inspected

- `src-tauri/src/main.rs`
- `src-tauri/src/lib.rs`
- `src-tauri/Cargo.toml`
- `src-tauri/capabilities/default.json`
- Current working-tree diff and touched-file list
- `reviews/2026-07-08-tauri-command-facade-rust-style.md`
- Required rewrite context: `docs/INDEX.md`, `docs/architecture/backend-rewrite.md`, `RESOURCES.md`, `manifest/rewrite-status.yaml`, `rewrite-records/README.md`, `reviews/README.md`
- Source-boundary context: `docs/adr/0001-backend-rewrite-facade.md`, `rewrite-records/0001-source-boundaries.md`
- Local high-confidence source index: `docs/references/high-confidence-sources.md`

# Rust style notes

`BackendError` now carries boxed context (`src-tauri/src/main.rs:63`, `src-tauri/src/main.rs:64`, `src-tauri/src/main.rs:68`) and `BackendResult<T>` is the facade return alias (`src-tauri/src/main.rs:60`). The boxed context keeps the error value small enough that clippy does not report large `Result` errors, while the command edge still stringifies in one place through `to_tauri_result` (`src-tauri/src/main.rs:1219`, `src-tauri/src/main.rs:1220`).

The command wrapper style is consistent: each command gets `state.inner()`, delegates through `app.backend`, and converts once at the Tauri edge (`src-tauri/src/main.rs:1237` through `src-tauri/src/main.rs:1530`). The `too_many_arguments` allowances remain localized to facade surfaces that mirror existing command payloads and legacy functions (`src-tauri/src/main.rs:129`, `src-tauri/src/main.rs:1413`, `src-tauri/src/main.rs:1459`, `src-tauri/src/main.rs:2360`, `src-tauri/src/main.rs:2651`).

The patch added relevant unit coverage for facade state behavior, progress payload field names, structured error retention until the Tauri edge, and token-like log redaction: `src-tauri/src/main.rs:3078`, `src-tauri/src/main.rs:3109`, `src-tauri/src/main.rs:3143`, `src-tauri/src/main.rs:3164`.

# Tests or checks run

- `cargo fmt --check` in `src-tauri`: passed.
- `cargo test` in `src-tauri`: passed; 4 `src-tauri/src/main.rs` tests passed and 0 `src-tauri/src/lib.rs` tests ran.
- `cargo clippy --all-targets -- -D warnings` in `src-tauri`: passed.
- `git diff --check`: passed.
- `git diff --name-only`: production changes are limited to `src-tauri/src/main.rs`.
- `rg` inspections for `AppBackend`, `LegacyMainBackend`, `BackendError`, `BackendResult`, boxed context, command wrappers, lint allowances, unwraps, panic/todo markers and legacy facade routing.

# Residual risk

Checks ran on the current host target only. Platform-specific `#[cfg(target_os = "windows")]` and `#[cfg(target_os = "macos")]` file-manager branches were not compiled in this pass.

I did not run the full frontend build, live Tauri app, or `test-api` checks because this role is limited to Rust style for the touched Tauri crate and the production diff is limited to `src-tauri/src/main.rs`. I also did not re-browse official Tauri docs because this re-review did not assess a command/state/capability API change; local Tauri 2 dependencies and capabilities were inspected.

# Promotion decision: pass-with-followups

The Rust style gate is promotable. The current facade compiles cleanly, passes clippy with warnings denied, keeps `BackendError` boxed and centralized, and stays inside the accepted Tauri command facade architecture. The follow-ups are structural: make backend selection router-shaped before mixed legacy/new ownership, and split the facade/error/adapters out of monolithic `main.rs` before the next backend migration expands it.
