# MVSEP - Music Separation Tool

Desktop client for separating music into vocals, instrumentals, drums, bass, and more.

## Features

- **Drag & Drop** - Simply drop audio files into the window to start
- **One-click Run** - Upload → wait for separation → auto download, no manual steps needed
- **Task Management** - View progress in real-time, with support for cancel, download, and delete
- **Multiple Algorithms** - Choose from various separation algorithms and models
- **Resume Downloads** - Interrupted downloads can be resumed from where they left off
- **Proxy Support** - Support for system proxy, manual proxy, or no proxy

## Download & Install

### Windows

Download `MVSEP_x.x.x_x64.exe` or `MVSEP_x.x.x_x64.msi` and run the installer.

### Linux

Download AppImage or DEB package:
- AppImage: Make executable and run directly
- DEB: Double-click to install or run `sudo dpkg -i MVSEP_xxx.deb`

### macOS

Download `.dmg` package, double-click and drag to Applications folder.

## Quick Start

### 1. First-time Setup

Configure the following on first use:

| Setting | Description |
|---------|-------------|
| **API Token** | Required. Get it from [MVSEP website](https://mvsep.com/user-api) |
| **Output Directory** | Where separated files are saved, default: `./output` |
| **Output Format** | Optional MP3/WAV/FLAC/M4A etc. |

### 2. Start Separation

1. On **Home**, drag in audio file or click to select
2. Choose **Algorithm** and **Model Options** (optional)
3. Choose **Output Format**
4. Click **One-click Run**, wait for completion and download will start automatically

### 3. View Tasks

- **Tasks** page shows all in-progress and historical tasks
- Click **Download** to download individual files
- Support to **Cancel** in-progress tasks

## FAQ

### How to get API Token?

1. Log in to [MVSEP](https://mvsep.com)
2. Click username in top right → select **API**
3. Copy the Token and paste it into the client settings page

### Separation is slow, what can I do?

- Check the queue info in **Tasks** page to see how many people are ahead
- Try different algorithms - some may be faster
- Consider using demo mode (free but results are public)

### Download interrupted, what now?

No worries, the client supports **resume downloads**. Just click download again and it will continue from where it stopped.

### How to update algorithm list?

Go to **Algorithms** page, click "Fetch Latest Algorithm Info" to pull the latest algorithms from the server.

## Page Overview

| Page | Function |
|------|----------|
| Home | Upload audio, select parameters, one-click run |
| Tasks | View progress, download results, manage tasks |
| Algorithms | Browse algorithms and models, save presets |
| Settings | API Token, proxy, output directory, etc. |
| Logs | View run logs for troubleshooting |

## Report Issues

If you encounter problems:
1. Check **Logs** page for detailed error information
2. Visit [GitHub Issues](https://github.com/AntheaLaffy/mvsep-rs/issues) to report

## License

MIT
