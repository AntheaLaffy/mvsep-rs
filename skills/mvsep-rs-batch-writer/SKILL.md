---
name: mvsep-rs-batch-writer
description: Implement one mvsep-rs backend rewrite batch as the code-writing role. Use when the user asks to write code for a selected migration batch, create AppBackend seams, move config/formats/algorithm cache/transfer/tasks to the rewritten backend, or prepare a batch for independent review. Do not use for review-only requests.
---

# mvsep-rs Batch Writer

## Overview

Implement exactly one migration batch while preserving behavior and preparing the work for independent review. This skill is the writer role; it must not author the review gate for its own changes.

## Required Context

Read these before editing:

- `docs/INDEX.md`
- `docs/architecture/backend-rewrite.md`
- `RESOURCES.md`
- `manifest/rewrite-status.yaml`
- `rewrite-records/README.md`
- `Note.md`
- Source files named by the selected batch
- Existing tests around the touched behavior

When touching framework or library APIs, read the official sources listed in `docs/references/high-confidence-sources.md`.

## Writer Workflow

1. Select or confirm the batch from `manifest/rewrite-status.yaml`.
2. If the batch is still `planned`, mark it `active` only when implementation actually starts.
3. Shrink the work to a minimum boundary: one useful slice with one primary verification surface.
4. Identify the stable public surface: Tauri command names, event names, TypeScript types, DB paths, file paths and rollback route.
5. Add focused tests or fixtures for the behavior being moved.
6. Implement behind the existing seam; never force frontend pages to understand backend internals.
7. Run the narrowest useful checks first, then the baseline checks from `docs/INDEX.md`.
8. If implementation is present but not reviewed, mark the batch `reimplemented`, not `verified`.
9. Add a `rewrite-records/` entry when the work reveals a reusable lesson or changes a source-boundary rule.
10. Final response must request the required review roles from the manifest.

## Source Boundary

- Use `teach` only for stateful workspace structure: mission, resources, notes, records and gradual minimum-boundary progression.
- Use `py2rs` only for engineering discipline: behavior first, reversible states, manifest state, role separation and review gates.
- Do not introduce py2rs architecture into this repo. The accepted architecture remains Tauri commands -> `AppBackend` -> `LegacyMainBackend`/`TestApiBackend`.

## Batch Rules

- `tauri_command_facade`: commands become thin adapters; behavior must remain identical.
- `config_and_formats`: use injected Tauri app paths; legacy JSON import must be idempotent.
- `algorithm_cache`: preserve local list/details payloads; cache errors need explicit recovery behavior.
- `transfer`: use async, cancellable transfer; preserve `upload-progress` and `download-progress`.
- `task_persistence`: keep frontend task shapes stable until adapter migration is complete.
- `frontend_gateway_and_ui`: centralize `invoke`/`listen`; no broad visual rewrite mixed with backend protocol work.

## Forbidden In Writer Role

- Do not mark a batch `verified` without independent review reports.
- Do not write the behavior/error/style/UX review report for your own patch.
- Do not broaden Tauri capabilities as a shortcut.
- Do not add blocking network/file transfer paths to async commands.
- Do not refactor unrelated UI or storage code in the same batch.
- Do not create py2rs-style `py/`, `rs/` or runtime router directories for this project.

## Completion Criteria

- Code and tests for one batch are complete or the blocker is concrete.
- Manifest state reflects reality.
- No review report claims are made by the writer.
- Final response lists files changed, checks run, remaining risks and required review roles.
