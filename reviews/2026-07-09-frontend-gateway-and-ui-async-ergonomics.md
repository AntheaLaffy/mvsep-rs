# Findings ordered by severity

No unresolved async ergonomics findings remain in this delta scope.

## Resolved: duplicate algorithm refresh no longer clears loading ownership

`fetchLatestAlgorithmInfo` now checks the `algo-fetch-latest` action lock before setting `isLoadingAlgorithmDetails` at `src/main.ts:1392`. The caller that finds the action already running returns before entering the `try/finally`, so it no longer clears the locale-change guard while the original refresh is still in flight. The original owner still sets the loading guard before calling `withUiAction` at `src/main.ts:1395` and clears it in its own `finally` at `src/main.ts:1425`.

This resolves the previous medium finding for ordinary event-loop duplicate clicks and overlapping callers.

## Resolved: progress listener registration failures now have explicit catch paths

`setupDownloadProgressListener` catches `backendGateway.onDownloadProgress(...)` registration failures at `src/main.ts:238` and logs both to console and the frontend debug bridge. `setupUploadProgressListener` does the same for upload progress registration at `src/main.ts:260`.

This resolves the previous unhandled listener-registration Promise finding. The app still does not retain unlisten callbacks, but this remains acceptable for the current single-mount Tauri app lifecycle.

## Resolved: external URL opener failures now have an error path

The `open-url` DOM action now calls `backendGateway.openExternalUrl(url).catch(...)` at `src/app/controllers/dom-events.ts:196`. Failures are sent to console, a transient warning notice, and the frontend debug bridge.

This resolves the previous fire-and-forget opener finding.

# Scope reviewed

Delta async ergonomics review for batch `frontend_gateway_and_ui` after follow-up fixes.

Reviewed exactly the requested follow-up areas:

- duplicate `fetchLatestAlgorithmInfo` guard ownership in `src/main.ts`
- progress listener `.catch` paths in `src/main.ts`
- `openExternalUrl` `.catch` handling in `src/app/controllers/dom-events.ts`
- gateway centralization boundary for Tauri `invoke`, `listen`, and plugin imports

This pass did not re-review visual polish, HTML escaping, Rust backend behavior, public command payload parity, or unrelated async flows.

# Files or interfaces inspected

- `src/main.ts`
  - `withUiAction`
  - `setupDownloadProgressListener`
  - `setupUploadProgressListener`
  - `fetchLatestAlgorithmInfo`
- `src/app/controllers/dom-events.ts`
  - `open-url` DOM action
  - locale-change guard call site
- `src/app/backend/gateway.ts`
  - confirmed it remains the only Tauri JavaScript API import and call site under `src`
- Project process documents:
  - `/home/fuurin/.claude/skills/mvsep-rs-review-gate/SKILL.md`
  - `docs/INDEX.md`
  - `docs/architecture/backend-rewrite.md`
  - `RESOURCES.md`
  - `manifest/rewrite-status.yaml`
  - `rewrite-records/README.md`
  - `reviews/README.md`

# Tests or checks run

- `npm run build`
  - Passed: `tsc && vite build`.
- `rg -n "\binvoke\b|\blisten\b|@tauri-apps" src --glob '*.ts'`
  - Passed for the gateway boundary: every match is in `src/app/backend/gateway.ts`.
- `git diff --check`
  - Passed.

# Residual risk

I did not launch a live Tauri window or run a real remote MVSep upload/download/open-url cycle. This delta review proves the source-level Promise handling and gateway boundary, but not platform-specific plugin behavior at runtime.

The previous async ergonomics follow-ups are resolved within the requested scope.

# Promotion decision

pass
