# mvsep-rs

<p align="center">
  <img src="docs/assets/rust-mascot.svg" alt="Rust crab mascot icon for mvsep-rs" width="96">
</p>

Languages: [中文](README.md) | [English](README.en.md)

mvsep-rs is a Tauri 2 desktop client and Rust backend rewrite for MVSep audio separation workflows. The repository contains the desktop UI, the Tauri command facade, and the extracted Rust API/backend layer from `test-api`: config, algorithm cache, upload and download transfer, task persistence, and download state.

The current rewrite policy is backend-first: for domains already migrated to the rewritten backend, the rewritten backend store is canonical. Legacy frontend storage is only a migration and rollback aid. If the same task, history record, or configuration exists in both old storage and the rewritten backend, prefer the rewritten backend unless a migration record explicitly documents a different conflict rule.

## Current State

- All migration batches in `manifest/rewrite-status.yaml` are verified.
- `src/app/backend/gateway.ts` is the only frontend module allowed to import Tauri JavaScript APIs, call `invoke`, or call `listen`.
- Tauri command names and progress event names remain stable while backend implementation details are replaced through `AppBackend`.
- Config, output formats, algorithm cache, upload/download transfer, active tasks, and task history are behind the backend facade.
- Backend paths are resolved from Tauri-injected app config/data paths, not from process cwd, the repository root, or the old program-body-relative path model.

## Repository Layout

```text
.
├── src/                       # TypeScript + Vite frontend
├── src-tauri/                 # Tauri desktop backend and AppBackend facade
├── test-api/                  # Extracted Rust MVSep API/backend layer and CLI harness
├── docs/                      # Architecture, mission, ADRs and source index
├── manifest/                  # Machine-readable migration batch status
├── rewrite-records/           # Durable migration lessons and boundary decisions
├── reviews/                   # Batch review reports
├── doc/                       # Local MVSep API notes
└── scripts/                   # Build scripts
```

## Quick Start

Install JavaScript and Rust dependencies, then run the frontend or Tauri app:

```bash
npm install
npm run dev
npm run tauri dev
```

Build the frontend:

```bash
npm run build
```

Build the AppImage:

```bash
npm run build:appimage
```

Run the standalone Rust CLI harness:

```bash
cd test-api
cargo run --release
```

## Verification

Use these baseline checks after backend rewrite work:

```bash
npm run build
cd src-tauri && cargo test
cd src-tauri && cargo clippy --all-targets -- -D warnings
cd test-api && cargo test
cd test-api && cargo clippy --all-targets -- -D warnings
```

Frontend Tauri API access must stay centralized:

```bash
rg -n "\binvoke\b|\blisten\b|@tauri-apps" src --glob '*.ts'
```

The strict expected result is that only `src/app/backend/gateway.ts` matches.

## Path Rules

Backend paths are injected from Tauri app config/data directories. The main databases live under the injected app data directory:

- `mvsep.db`
- `user_config.db`
- `tasks.db`

Uploaded source files keep the local file path selected by the user. Download output directories may be absolute paths. Relative output paths such as `./output` resolve under the injected app data directory; they do not resolve under the old backend binary location, the repository root, or the current cwd.

Downloaded output file paths are stored in the rewritten backend task/history data. The frontend should read and display those backend records instead of reconstructing download paths from legacy localStorage.

## Documentation

- `docs/INDEX.md`: main entry point for agents and maintainers.
- `docs/mission.md`: goals, non-goals, and rewrite strategy.
- `docs/architecture/backend-rewrite.md`: accepted backend rewrite architecture.
- `manifest/rewrite-status.yaml`: batch status and review gates.
- `CONTEXT.md`: project glossary.
- `Note.md`: human working notes and durable preferences.
- `RESOURCES.md`: high-confidence sources and borrowing boundaries.
- `rewrite-records/`: non-obvious migration decisions and lessons.
- `reviews/`: behavior, tracing, async, style, data, and UX review reports.

## Generated Files

`dist/`, `node_modules/`, and Vite cache files are local generated output and are not part of source control. Regenerate them from `package-lock.json` and the source tree when needed.

## License

Apache License 2.0
