# 0001 Source Boundaries

## Context

The mvsep-rs rewrite workflow borrows ideas from both `teach` and `py2rs`, but the project is neither a teaching workspace nor a Python-to-Rust migration. The accepted architecture is the mvsep-rs Tauri command facade with `AppBackend`, `LegacyMainBackend` and `TestApiBackend`.

## Decision Or Lesson

Borrow from `teach`:

- Mission-grounded work.
- Resource records before relying on memory.
- Durable notes for user preferences and style constraints.
- Records for non-obvious lessons.
- Minimum scoped progression: each batch should be small enough to understand, verify and review.

Borrow from `py2rs`:

- Behavior before optimization.
- Reversible migration states.
- Manifest as source of migration state.
- Writer and reviewer role separation.
- Dedicated review gates for behavior, errors, async ergonomics, data/algorithm, code style and UX.

Do not borrow py2rs architecture:

- No `py/` and `rs/` split.
- No Python runtime router.
- No script-as-migration-unit rule.
- No Python/Rust dependency alignment workflow.
- No py2rs stage numbering as the project architecture.

## Applies To

All mvsep-rs rewrite skills, docs, implementation batches and review reports.

## Does Not Imply

This record does not change the accepted backend seam. The seam remains:

```text
Frontend -> BackendGateway -> Tauri commands -> AppBackend -> LegacyMainBackend/TestApiBackend
```

## Follow-up

Keep skill instructions and architecture docs aligned with this boundary whenever the rewrite workflow is updated.
