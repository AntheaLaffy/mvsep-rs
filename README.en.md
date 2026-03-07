# MVSEP GUI (mvsep-rs)

Languages: [简体中文](./README.md) | English | [日本語](./README.ja.md)

A desktop client for MVSEP built with **Tauri 2 + TypeScript + Rust**.  
It helps you create separation tasks, track progress, and download results.

- Project: https://github.com/AntheaLaffy/mvsep-rs
- MVSEP: https://mvsep.com

## For End Users

You do not need Rust/Node.js. Just download the prebuilt app packages.

### Download

- Windows: `MVSEP-Setup.exe` (or portable `MVSEP.exe`)
- Linux: `MVSEP.AppImage` (or `.deb` / `.rpm` if provided)
- Recommended release channel:  
  `https://github.com/AntheaLaffy/mvsep-rs/releases`

### First-time Setup

1. Open app and go to **Settings**
2. Set API Token (`https://mvsep.com/user-api`)
3. Keep API URL as `https://mvsep.com`
4. Click **Test Connection**
5. Set output directory

### Daily Workflow

1. Drop an audio file on Home
2. Select algorithm/options/output format
3. Click **One-click Run**  
   or **Create Task** (create only, no auto-download)
4. Check progress and download files in **Tasks**

### User FAQ

#### 1) Connection failed

- Verify API token
- Check network/proxy settings
- Run **Test Connection** again

#### 2) Download interrupted

Click download again. Resume is supported.

#### 3) AppImage does not launch on Linux

```bash
chmod +x MVSEP.AppImage
./MVSEP.AppImage
```

## Features

- Drag/drop file and create separation tasks
- One-click run (create -> poll -> auto-download)
- Task center (in-progress / completed / history)
- Real-time upload/download progress
- Download cancel + resume
- Local algorithm cache + remote refresh
- Multiple output formats (MP3/WAV/FLAC/M4A...)
- Proxy support (system/manual/none)
- Frontend/backend logs
- Multilingual UI (中文 / English / 日本語)

## Tech Stack

- Frontend: Vite + TypeScript + TailwindCSS
- Desktop: Tauri 2
- Backend: Rust + Tokio + Reqwest
- Plugins: dialog / fs / log / opener / shell

## Environment (Developers)

- Node.js 18+
- Rust stable
- Tauri 2 system dependencies

Linux example dependencies:

```bash
sudo apt install -y \
  libwebkit2gtk-4.1-dev \
  libgtk-3-dev \
  libayatana-appindicator3-dev \
  librsvg2-dev \
  patchelf
```

## For Developers

## Quick Start

```bash
npm install
npm run tauri dev
```

## Build & Bundle

```bash
npm run build
npm run tauri build
npm run build:appimage
```

Bundle output is usually under `src-tauri/target/release/bundle/`.

## Config and Cache Paths

Stored in `mvsep-gui` under system config directory:

- `config.json`
- `algorithms_cache.json`

Common locations:

- Linux: `~/.config/mvsep-gui/`
- macOS: `~/Library/Application Support/mvsep-gui/`
- Windows: `%APPDATA%\mvsep-gui\`

## License

This project includes Apache License 2.0. See [LICENSE](./LICENSE).
