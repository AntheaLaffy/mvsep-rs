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

## User Config (user_config.db)

Key-value storage via `UserConfigDB`. All settings in one table:

| Key | Type | Default | Description |
|-----|------|---------|-------------|
| `token` | string | — | API token |
| `proxy_mode` | string | `system` | Proxy mode |
| `proxy_host` | string | `127.0.0.1` | Proxy host |
| `proxy_port` | string | `7897` | Proxy port |
| `output_dir` | string | `./output` | Download directory |
| `output_format` | int | `1` | Default format ID |
| `premium_enabled` | int | `0` | Premium toggle |
| `long_filenames_enabled` | int | `0` | Long filenames toggle |
| `algorithm_last_fetched_at` | int | — | Cache timestamp (Unix) |
| `algorithm_auto_refresh_days` | int | `15` | Cache expiry days |
| `preset:{name}` | json | — | Saved presets |

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
