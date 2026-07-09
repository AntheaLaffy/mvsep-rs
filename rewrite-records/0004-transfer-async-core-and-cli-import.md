# Context

The transfer batch moved upload/download streaming into `test-api::file_transfer` while the Tauri backend kept the public command and event adapter surface.

# Decision or Lesson

Reusable transfer behavior should live in the library module and GUI/CLI entrypoints should import that library module instead of re-declaring the same source file with `mod file_transfer`. Re-declaring the library file in the CLI binary creates a second private compilation unit, which can turn legitimate library APIs into bin-only dead-code warnings.

# Applies To

- Future transfer changes in `test-api/src/file_transfer.rs`
- CLI code in `test-api/src/main.rs`
- Later task persistence or API modules that are shared by Tauri and CLI entrypoints

# Does Not Imply

- This does not require rewriting the whole CLI to consume every `test-api` library module immediately.
- This does not change the accepted Tauri command facade architecture.
- This does not make the frontend aware of transfer internals.

# Follow-up

When cleaning CLI warning debt, consider moving other duplicated modules behind library imports in small, verified steps.
