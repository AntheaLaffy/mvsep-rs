# Review Reports

Each migration batch must have review notes before it is marked `verified` in `manifest/rewrite-status.yaml`.

## Naming

Use this pattern:

```text
reviews/YYYY-MM-DD-<batch-id>-<role>.md
```

Examples:

- `reviews/2026-07-08-tauri-command-facade-behavior.md`
- `reviews/2026-07-08-transfer-async-ergonomics.md`
- `reviews/2026-07-08-frontend-gateway-ux.md`

## Required Sections

- Scope reviewed
- Files or interfaces inspected
- Findings ordered by severity
- Tests or checks run
- Residual risk
- Promotion decision: `pass`, `pass-with-followups` or `fail`

## Agent Separation

- Writer agents do not author review reports for their own patches.
- A reviewer handles one review theme only.
- Behavior review is the first gate for every batch.
