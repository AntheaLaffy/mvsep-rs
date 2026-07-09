# Context

The `frontend_gateway_and_ui` batch centralizes frontend calls to Tauri commands, events and plugins while keeping existing command names, event names and payload field names stable.

# Decision or Lesson

`src/app/backend/gateway.ts` is the only frontend module that should import `@tauri-apps/*`, call `invoke`, or call `listen`. Page code, services and DOM controllers should depend on gateway methods instead of Tauri JS APIs directly.

Gateway methods should preserve the current protocol-bearing argument object keys, including `apiUrl`, `token`, `filePath`, `sepType`, `outputFormat`, `outputDir`, `fileIndex`, `originalFileName`, `algorithmId`, `proxyMode`, `proxyHost` and `proxyPort`.

The verification search must use word boundaries, not `invoke\(`. Generic calls such as `invoke<T>(...)` are otherwise missed. Use:

```text
rg -n "\binvoke\b|\blisten\b|@tauri-apps" src --glob '*.ts'
```

Remote, cached, persisted or backend-provided strings rendered through `innerHTML` must be escaped for both text and quoted attribute contexts. Progress values used in inline styles should be clamped before interpolation.

# Applies To

- `src/app/backend/gateway.ts`
- `src/main.ts`
- `src/app/services/tasks.ts`
- `src/app/controllers/dom-events.ts`
- `src/app/render/*.ts`
- Future frontend pages, services and controllers that need backend access

# Does Not Imply

- This does not rename any Tauri command, event or payload field.
- This does not broaden Tauri capabilities.
- This does not require a broad visual redesign in the backend gateway batch.
- This does not make old frontend storage or direct invoke paths authoritative for migrated domains.

# Follow-up

Required review gates should independently check behavior compatibility, frontend UX/render safety and async ergonomics before the batch can move from `reimplemented` to `verified`.
