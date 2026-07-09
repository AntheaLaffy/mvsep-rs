---
name: mvsep-rs-rewrite
description: Coordinate the mvsep-rs gradual backend rewrite. Use when the user asks to continue, plan, implement, audit, or route work for replacing the old Tauri backend with the rewritten Rust backend; when updating migration state, project constraints, rewrite docs, or multi-agent rewrite workflow; or when deciding the next mvsep-rs migration batch.
---

# mvsep-rs Rewrite

## Overview

Drive the project-specific rewrite loop for mvsep-rs. This skill treats the repo like a stateful engineering workspace: mission, resources, notes, rewrite records, architecture, manifest and reviews are the durable handoff.

## Start Here

1. Confirm the current workspace is `mvsep-rs`; if not, locate it or ask for the repo path.
2. Read these files before making decisions:
   - `docs/INDEX.md`
   - `docs/mission.md`
   - `RESOURCES.md`
   - `Note.md`
   - `CONTEXT.md`
   - `rewrite-records/README.md`
   - `manifest/rewrite-status.yaml`
   - `docs/architecture/backend-rewrite.md`
3. Read `docs/references/high-confidence-sources.md` when touching Tauri, Tailwind, Vite, Tokio, reqwest, rusqlite, or MVSep API behavior.
4. If the task is only a review, use the review workflow in `mvsep-rs-review-gate` instead of writing production code.
5. If the task is implementation, use the writer workflow in `mvsep-rs-batch-writer` or follow its rules directly.

## Batch Selection

- If the user names a batch, work on that batch.
- Otherwise pick the first batch in `manifest/rewrite-status.yaml` that is not `verified`, `promoted`, or `optimized`.
- Do not skip a behavior gate to do later optimization.
- Keep one migration batch active unless the user explicitly asks for parallel work and write scopes are disjoint.
- Treat each batch as a minimum boundary: one useful slice that can be understood, tested, reviewed and resumed.

## Borrowed Ideas Boundary

- Borrow from `teach`: mission-grounded work, resource records, notes, rewrite records and minimum scoped progression.
- Borrow from `py2rs`: behavior-first migration, reversible state, manifest-driven progress, role-separated agents and review gates.
- Do not borrow py2rs architecture: no `py/` and `rs/` split, no Python runtime router, no script-as-unit migration rule and no py2rs stage numbering as this project's architecture.
- If this boundary changes, update `RESOURCES.md` and add a `rewrite-records/` entry before changing implementation.

## Rewrite Loop

1. Ground in state: read resources, rewrite records, manifest, architecture, relevant source files and existing tests.
2. Define the batch's public interface: command names, event names, TypeScript payloads, DB paths and rollback route.
3. Add or update tests that prove behavior before changing implementation when practical.
4. Implement through the chosen seam: Tauri commands -> `AppBackend` -> legacy or rewritten adapter.
5. Run the relevant checks from `docs/INDEX.md`.
6. Update `manifest/rewrite-status.yaml` only to a factual state reached by the work.
7. Produce or request required review reports under `reviews/` before promotion.

## Non-Negotiables

- Preserve existing Tauri command names and progress event names unless a batch explicitly changes protocol.
- Do not let frontend pages call database/repository shapes directly.
- Do not replace the accepted Tauri command facade architecture with py2rs runtime/router architecture.
- Do not place blocking HTTP or nested Tokio runtimes in Tauri async command paths.
- Keep errors structured until the Tauri command edge stringifies them.
- Do not let a code-writing agent review its own code.
- Do not mix review themes in one reviewer pass.

## Multi-Agent Rules

- Use a writer for implementation and separate reviewers for behavior, error tracing, async ergonomics, data/algorithm, Rust style and frontend UX.
- If subagents are available and explicitly requested, assign disjoint ownership and tell workers they are not alone in the codebase.
- Reviewers may write review reports, but must not edit production code.
- Behavior review is the first gate for every batch.

## Completion Criteria

- The next action is obvious from the manifest or final response.
- Any code changes have tests or a clear statement of untested risk.
- Any review outcome has a report in `reviews/`.
- The final response names changed files and checks run.
