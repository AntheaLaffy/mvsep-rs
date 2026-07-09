# Manifest

`manifest/` records the state of the gradual backend replacement. It is the source of truth for migration progress, not a substitute for tests.

## Files

- `rewrite-status.yaml`: current migration batches, owners, status and verification requirements.

## Status Values

- `planned`: batch is described but not implemented.
- `active`: implementation has started.
- `reimplemented`: new backend path exists but is not fully verified.
- `verified`: behavior tests and required reviews passed.
- `promoted`: default runtime path uses the new backend implementation.
- `optimized`: post-migration quality improvements are complete.
- `blocked`: batch cannot move forward without an explicit decision or external dependency.

## Rules

- Advance one batch at a time unless the work scopes are independent.
- Do not mark a batch `verified` without a matching review report in `reviews/`.
- Record rollback notes before promotion.
- Keep statuses factual; do not use this file as a task wish list.
