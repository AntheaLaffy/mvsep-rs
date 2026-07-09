# MVSEP - Music Separation Tool

MVSEP desktop client for separating music into vocal, accompaniment, drums, bass and other tracks. Supports drag-and-drop upload, one-click operation, task management, and resumable downloads.

[![License](https://img.shields.io/crates/l/mvsep-api-tester.svg)](https://crates.io/crates/mvsep-api-tester)
[![Crates.io](https://img.shields.io/crates/v/mvsep-api-tester.svg)](https://crates.io/crates/mvsep-api-tester)
[![Crates.io](https://img.shields.io/crates/v/mvsep-gui.svg)](https://crates.io/crates/mvsep-gui)
[![Docs](https://docs.rs/mvsep-api-tester/badge.svg)](https://docs.rs/mvsep-api-tester)

Languages: [中文](README.md) | [English](README.en.md) | [日本語](README.ja.md)

## Features

### User Features
- **Drag and Drop** - Drag audio files into the window to start processing
- **One-click Operation** - Upload → Wait for separation → Auto download, no manual steps required
- **Task Management** - Real-time separation progress, support interrupt, download, delete tasks
- **Multiple Algorithms** - Support multiple separation algorithms and models
- **Resumable Downloads** - Click download again to resume from interruption
- **Proxy Support** - System proxy, manual proxy, or no proxy modes

### Technical Features
- **Three-database Architecture**: Algorithm cache, task tracking, user config independently managed
- **Streaming Upload**: Async file upload based on tokio, with progress callback and cancellation
- **Task Persistence**: Complete task lifecycle management and history records

## Installation

### Arch Linux / Manjaro (AUR)

```bash
# Prebuilt binary version (recommended, fast installation)
paru -S mvsep-gui-bin
# or
yay -S mvsep-gui-bin

# Source build version (requires Rust and Node.js)
paru -S mvsep-gui
# or
yay -S mvsep-gui
```

### Windows

Download `MVSEP_1.2.0_x64-setup.exe` and run the installer.

### Debian/Ubuntu

```bash
wget https://github.com/AntheaLaffy/mvsep-rs/releases/download/v1.2.0/MVSEP_1.2.0_amd64.deb
sudo dpkg -i MVSEP_1.2.0_amd64.deb
```

### Fedora/RHEL

```bash
wget https://github.com/AntheaLaffy/mvsep-rs/releases/download/v1.2.0/MVSEP-1.2.0-1.x86_64.rpm
sudo dnf install MVSEP-1.2.0-1.x86_64.rpm
```

### Build from Source

```bash
# Install dependencies
sudo pacman -S webkit2gtk libappindicator-gtk3 librsvg libvips npm nodejs

# Clone repository
git clone https://github.com/AntheaLaffy/mvsep-rs.git
cd mvsep-rs

# Build frontend
npm install
npm run build

# Build backend
cd src-tauri
cargo build --release

# Run
./target/release/mvsep-gui
```

## Quick Start

### 1. First-time Setup

You need to configure the following:

| Setting | Description |
|---------|-------------|
| **API Token** | Required. Get from [MVSEP website](https://mvsep.com/user-api) |
| **Output Directory** | Where separation results are saved |
| **Output Format** | MP3/WAV/FLAC/M4A and more |

### 2. Start Separation

1. **Home Page** - Drag audio file or click to select file
2. Select **Algorithm** and **Model Options** (optional)
3. Select **Output Format**
4. Click **One-click Run**, wait for completion and auto-download

### 3. View Tasks

- **Tasks Page** - View all running and historical tasks
- Click **Download** to download individual files
- Support **Cancel** for running tasks

## Page Overview

| Page | Function |
|------|----------|
| Home | Upload audio, select parameters, one-click run |
| Tasks | View progress, download results, manage tasks |
| Algorithms | Browse available algorithms and models, save presets |
| Settings | API Token, proxy, output directory configuration |
| Logs | View runtime logs for troubleshooting |

## FAQ

### How to get API Token?

1. Login to [MVSEP](https://mvsep.com)
2. Click username in top right → Select **API**
3. Copy Token and paste into client settings

### Separation is slow?

- Check **Tasks Page** for queue information
- Try different algorithms for faster processing
- Consider demo mode (free but results are public)

### Download interrupted?

No worries, the client supports **resumable downloads**. Just click download again to resume.

### How to update algorithm list?

Go to **Algorithms Page**, click "Get Latest Algorithms" to fetch from server.

## Developer Guide

### Development Mode

```bash
npm install
npm run tauri dev
```

### Build AppImage

```bash
npm run build:appimage
```

### Database Operations (Rust)

```rust
use mvsep_api_tester::db;

let db = db::Database::new(None)?;
let algorithms = db.with_conn(|conn| {
    db::repositories::get_all_algorithms(conn)
})?;
```

### File Upload (Rust)

```rust
use mvsep_api_tester::file_transfer::{self, TransferProgress};

let hash = file_transfer::upload_file_async(
    &client, "https://api.mvsep.com/upload",
    std::path::Path::new("./song.mp3"),
    vec![("api_token", "your-token".to_string())],
    None, |progress| {
        println!("Upload: {:.1}%", progress.percent);
    },
).await?;
```

## Project Structure

```text
mvsep-rs/
├── src/                      # TypeScript + Vite frontend
├── src-tauri/                # Tauri desktop backend
├── test-api/                 # Rust core library (crates.io: mvsep-api-tester)
│   ├── src/db/               # Database layer
│   ├── src/file_transfer.rs  # File transfer (upload/download)
│   └── src/utils/            # Utility functions
├── docs/                     # Architecture docs and ADR
└── manifest/                 # Migration batch status
```

## API Reference

Detailed documentation at [docs.rs](https://docs.rs/mvsep-api-tester).

## Feedback

If you encounter issues:
1. Check **Logs Page** for detailed error information
2. Report at [GitHub Issues](https://github.com/AntheaLaffy/mvsep-rs/issues)

## License

Apache License 2.0
