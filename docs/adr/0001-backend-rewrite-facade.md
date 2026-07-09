# ADR 0001: Use Tauri Command Facade As Backend Rewrite Seam

## Status

Accepted.

## Context

The current desktop backend is concentrated in `src-tauri/src/main.rs`, while the rewritten Rust API layer lives under `test-api`. The frontend already depends on Tauri command names and progress events. Replacing the whole backend at once would force simultaneous changes across Rust, TypeScript, storage and UI behavior.

## Decision

Introduce an `AppBackend` interface behind existing Tauri commands. Keep command names, request payloads and progress event names stable while migrating implementations from `LegacyMainBackend` to `TestApiBackend` batch by batch.

## Consequences

- Frontend pages do not need to know whether a capability is served by the legacy or rewritten backend.
- Rollback can happen per capability by routing back to the legacy adapter.
- Contract tests can compare adapters through the same interface.
- The facade adds one temporary layer that should be simplified after the replacement is complete.
