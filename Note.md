# Note

Project working notes and style constraints. Treat this as the short human-readable checklist before changing code or docs.

## Working Style

- Prefer small migration batches over broad rewrites.
- Treat each batch as a minimum boundary: one useful slice that can be understood, tested, reviewed and resumed.
- Keep the app runnable and rollbackable after every batch.
- Preserve existing public Tauri command names and progress events until a migration batch explicitly changes them.
- Write behavior tests before optimizing architecture.
- Do not let code-writing agents review their own changes.
- Do not mix review roles: behavior, tracing, async ergonomics, data/algorithm, style and UX reviews are separate.
- Borrow `teach` for mission/resources/notes/records and gradual progression.
- Borrow `py2rs` for rewrite discipline only; do not borrow its Python/Rust runtime architecture.

## Visual And UX Constraints

- This is a desktop utility, not a marketing site. Prefer dense, calm, task-focused UI.
- Optimize the high-frequency flow: select file, choose algorithm, submit, monitor progress, download, retry.
- Avoid blocking the UI during upload, polling, download, logging and settings autosave.
- Avoid unnecessary full-page rerenders; preserve focus and selection state.
- Replace emoji-dependent controls during visual polish when practical.
- Long filenames, long algorithm names and translated text must not overflow buttons, cards or mobile navigation.

## Frontend Constraints

- Centralize Tauri communication in a backend adapter; page/render code should not call `invoke` directly.
- Render helpers must escape untrusted strings from remote API, logs, filenames and config.
- Tailwind is currently v3.4.x. Do not move to v4 inside backend migration batches.
- If Tailwind v4 is considered later, first update build tooling and design tokens in a dedicated batch.

## Backend Constraints

- During the rewrite, the rewritten backend store is canonical for migrated domains. Legacy frontend storage such as localStorage is only a migration and rollback aid; when backend and legacy storage contain the same task/history identity, prefer the rewritten backend unless a batch explicitly documents a different conflict rule.
- Tauri async commands must not call blocking HTTP clients or create nested Tokio runtimes.
- Backend paths must be injected from Tauri app config/data dirs, not inferred from cwd.
- Errors must preserve operation name and useful context before being stringified for Tauri.
- Logs must not expose API tokens or credential-like values.
- DB migration/import steps must be idempotent.
