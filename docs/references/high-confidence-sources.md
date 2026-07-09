# High Confidence Sources

工作时按可信度从高到低使用资料。若线上资料和本地代码冲突，先确认版本，再以当前仓库依赖版本和实际代码为准。

## Priority 1: Local Truth

- `RESOURCES.md`: 借鉴来源、可信资料和禁止套用边界。
- `package.json`: Tauri、Tailwind、Vite、TypeScript 版本。
- `src-tauri/Cargo.toml`: Tauri 后端依赖。
- `test-api/Cargo.toml`: 新 Rust API 层依赖。
- `src-tauri/capabilities/default.json`: Tauri 2 capability 权限。
- `doc/mvsep_api_endpoints.md`: 已整理的 MVSep API 端点资料。
- Existing tests: `test-api/tests/`.

## Priority 2: Official Documentation

- Tauri 2 calling Rust commands and state: https://v2.tauri.app/develop/calling-rust/
- Tauri 2 state management: https://v2.tauri.app/develop/state-management/
- Tauri 2 capabilities and permissions: https://v2.tauri.app/security/capabilities/
- Tauri JavaScript API: https://v2.tauri.app/reference/javascript/api/
- Tailwind CSS v3 configuration: https://v3.tailwindcss.com/docs/configuration
- Tailwind CSS v4 Vite install, for later optional upgrade only: https://tailwindcss.com/docs/installation/using-vite
- Vite configuration: https://vite.dev/config/
- Rust standard library: https://doc.rust-lang.org/std/
- Tokio docs: https://docs.rs/tokio/
- reqwest docs: https://docs.rs/reqwest/
- rusqlite docs: https://docs.rs/rusqlite/

## Priority 3: Verified Runtime Checks

- `npm run build`
- `cd test-api && cargo test`
- `cargo clippy -- -D warnings`
- Mock HTTP transfer tests for upload, download, resume, cancellation and error paths.
- Ignored proxy/API tests only when `MVSEP_API_TOKEN`, proxy host and proxy port are intentionally configured.

## Rules

- For Tauri/Tailwind API details, check official docs before changing architecture.
- Do not use blog posts or generated snippets as final authority for framework behavior.
- Do not infer MVSep response shapes from memory; use `doc/mvsep_api_endpoints.md`, current parser code, or captured fixtures.
- Do not copy py2rs architecture into this project; only borrow explicitly recorded engineering principles from `RESOURCES.md` and `rewrite-records/`.
- If a dependency version changes, update this file in the same PR.
