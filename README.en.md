# mvsep-rs

> MVSep backend rewrite · MVP stage

See [Chinese README](./README.md) for full documentation.

## Three Databases

| Database | Content |
|----------|---------|
| **`mvsep.db`** | Algorithm cache (groups, fields, formats) |
| **`tasks.db`** | Task tracking (tasks, history, download progress) |
| **`user_config.db`** | User preferences (token, proxy, presets) |

## Preset System

Save/load algorithm + options combinations as named presets.

**Create:** `[b]` Browse → press `s` | `[3]` Create task → press `s` before upload | `[c]` → `[s]`
**Load:** `[3]` Create task → type `l` at `Sep Type ID` prompt
**Delete:** `[c]` → `[d]`

## CLI Tester

```bash
cd test-api
cargo run --release
```

### Features

| Feature | Description |
|---------|-------------|
| Algorithm cache | Fetch from API, local expiry check |
| Task creation | Streaming upload with progress |
| Status polling | Auto-poll on list, queued/processing/done/expired |
| Resume download | `.part` + `Range` header, per-file tracking |
| File rename | `{original_stem}_{suffix}.{ext}` |
| Browse algorithms | Grouped by category, free/premium badges |

### Tech Stack

Rust + reqwest (async+blocking) + tokio + SQLite
