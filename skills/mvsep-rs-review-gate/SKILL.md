---
name: mvsep-rs-review-gate
description: Review one mvsep-rs rewrite batch quality gate without writing production code. Use when the user asks for behavior parity review, error tracing review, async or ergonomics review, data structure or algorithm review, Rust style audit, frontend UX audit, or promotion readiness for a mvsep-rs migration batch.
---

# mvsep-rs Review Gate

## Overview

Run one independent review theme for one migration batch and write a report under `reviews/`. This is the reviewer role; it does not patch production code.

## Required Context

Read these first:

- `docs/INDEX.md`
- `docs/architecture/backend-rewrite.md`
- `RESOURCES.md`
- `manifest/rewrite-status.yaml`
- `rewrite-records/README.md`
- `reviews/README.md`
- The diff and files touched by the batch
- Tests relevant to the batch

If the review concerns Tauri or Tailwind behavior, verify against official sources from `docs/references/high-confidence-sources.md`.

## Choose Exactly One Role

- `behavior_reviewer`: public payloads, command/event compatibility, old vs new behavior.
- `error_tracing_reviewer`: structured errors, log context, redaction, diagnosability.
- `async_ergonomics_reviewer`: non-blocking behavior, cancellation, polling, user/developer ergonomics.
- `data_algorithm_reviewer`: schema, data migration, data structures, complexity and benchmarks.
- `rust_style_reviewer`: Rust module shape, ownership, clippy, warnings and maintainability.
- `frontend_ux_reviewer`: visual polish, accessibility, focus, text overflow and workflow usability.

If the user asks for all reviews, run behavior first and state that the remaining roles must be separate passes or separate agents.

## Review Workflow

1. Confirm batch id and role.
2. Confirm the batch stayed inside its minimum boundary.
3. Confirm the work respects source boundaries: teach-inspired state/resource/record structure, py2rs-inspired discipline only, mvsep-rs architecture unchanged.
4. Inspect only the scope needed for that role.
5. Run non-mutating checks where useful.
6. Report findings first, ordered by severity, with file/line references.
7. Write a review report using the format in `reviews/README.md`.
8. Use promotion decision `pass`, `pass-with-followups`, or `fail`.

## Boundaries

- Do not edit production code.
- Do not combine multiple review roles in one report.
- Do not mark the batch `verified`; the orchestrating rewrite step updates manifest after all required reviews pass.
- Do not rely on the writer's explanation when the code, tests or docs can answer the question.
- Do not approve py2rs architectural imports unless the user explicitly changes the accepted mvsep-rs architecture in an ADR.
- If no issue is found, say so clearly and document residual risk.

## Report Naming

Use:

```text
reviews/YYYY-MM-DD-<batch-id>-<role>.md
```

The final answer should summarize the decision and link the report path.
