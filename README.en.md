# mvsep-rs

> MVSep backend rewrite · MVP stage

See [Chinese README](./README.md) for full documentation.

## test-api — CLI Tester

```bash
cd test-api
cargo run --release
```

- Algorithm cache from API
- Streaming upload / download with resume & progress
- Task lifecycle: create, poll, cancel, download
- File rename: `{original_stem}_{suffix}.{ext}`
- Dual-database design (remote cache + user config)

Tech: Rust + reqwest + tokio + SQLite
