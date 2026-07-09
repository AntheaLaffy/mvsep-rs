# MVSEP - 音乐分离工具

MVSEP 桌面客户端，用于将音乐分离为人声、伴奏、鼓点、贝斯等音轨。支持拖拽上传、一键运行、任务管理、断点续传等功能。

[![License](https://img.shields.io/crates/l/mvsep-api-tester.svg)](https://crates.io/crates/mvsep-api-tester)
[![Crates.io](https://img.shields.io/crates/v/mvsep-api-tester.svg)](https://crates.io/crates/mvsep-api-tester)
[![Crates.io](https://img.shields.io/crates/v/mvsep-gui.svg)](https://crates.io/crates/mvsep-gui)
[![Docs](https://docs.rs/mvsep-api-tester/badge.svg)](https://docs.rs/mvsep-api-tester)

语言: [中文](README.md) | [English](README.en.md) | [日本語](README.ja.md)

## 功能特点

### 用户功能
- **拖拽上传** - 将音频文件拖入窗口即可开始处理
- **一键运行** - 上传 → 等待分离完成 → 自动下载，全程无需手动操作
- **任务管理** - 实时查看分离进度，支持中断、下载、删除任务
- **多种算法** - 支持多种分离算法和模型可选
- **断点续传** - 下载中断后再次点击即可继续，无需重新开始
- **代理支持** - 支持系统代理、手动代理或无代理

### 技术特性
- **三数据库架构**：算法缓存、任务追踪、用户配置独立管理
- **流式上传**：基于 tokio 的异步文件上传，支持进度回调和取消
- **任务持久化**：完整的任务生命周期管理和历史记录

## 下载安装

### Arch Linux / Manjaro (AUR)

```bash
# 预编译二进制版本（推荐，快速安装）
paru -S mvsep-gui-bin
# 或
yay -S mvsep-gui-bin

# 源码构建版本（需要 Rust 和 Node.js）
paru -S mvsep-gui
# 或
yay -S mvsep-gui
```

### Windows

下载 `MVSEP_1.2.0_x64-setup.exe`，运行安装程序即可。

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

### 从源码构建

```bash
# 安装依赖
sudo pacman -S webkit2gtk libappindicator-gtk3 librsvg libvips npm nodejs

# 克隆仓库
git clone https://github.com/AntheaLaffy/mvsep-rs.git
cd mvsep-rs

# 构建前端
npm install
npm run build

# 构建后端
cd src-tauri
cargo build --release

# 运行
./target/release/mvsep-gui
```

## 快速开始

### 1. 首次设置

首次使用需要配置以下内容：

| 设置项 | 说明 |
|--------|------|
| **API Token** | 必填。在 [MVSEP 网站](https://mvsep.com/user-api) 获取 |
| **输出目录** | 分离结果保存位置 |
| **输出格式** | 可选 MP3/WAV/FLAC/M4A 等 |

### 2. 开始分离

1. **首页** 拖入音频文件，或点击选择文件
2. 选择 **算法** 和 **模型选项**（可选）
3. 选择 **输出格式**
4. 点击 **一键运行**，等待完成后自动下载到本地

### 3. 查看任务

- **任务页** 查看所有进行中和历史任务
- 点击 **下载** 可单独下载某个文件
- 支持 **取消** 进行中的任务

## 页面说明

| 页面 | 功能 |
|------|------|
| 首页 | 上传音频、选择参数、一键运行 |
| 任务 | 查看进度、下载结果、管理任务 |
| 算法 | 浏览可选算法和模型、保存预设 |
| 设置 | API Token、代理、输出目录等配置 |
| 日志 | 查看运行日志，用于问题排查 |

## 常见问题

### 如何获取 API Token？

1. 登录 [MVSEP](https://mvsep.com)
2. 点击右上角用户名 → 选择 **API**
3. 复制 Token 并粘贴到客户端设置页

### 分离速度慢怎么办？

- 查看 **任务页** 的队列信息，了解当前排队人数
- 切换不同算法可能获得更快的处理速度
- 考虑使用演示模式（免费但结果公开）

### 下载中断怎么办？

无需担心，客户端支持**断点续传**。直接再次点击下载按钮即可从中断处继续。

### 如何更新算法列表？

进入 **算法页**，点击「获取最新算法信息」从服务器拉取最新算法。

## 开发者指南

### 开发模式

```bash
npm install
npm run tauri dev
```

### 构建 AppImage

```bash
npm run build:appimage
```

### 数据库操作（Rust）

```rust
use mvsep_api_tester::db;

let db = db::Database::new(None)?;
let algorithms = db.with_conn(|conn| {
    db::repositories::get_all_algorithms(conn)
})?;
```

### 文件上传（Rust）

```rust
use mvsep_api_tester::file_transfer::{self, TransferProgress};

let hash = file_transfer::upload_file_async(
    &client, "https://api.mvsep.com/upload",
    std::path::Path::new("./song.mp3"),
    vec![("api_token", "your-token".to_string())],
    None, |progress| {
        println!("上传: {:.1}%", progress.percent);
    },
).await?;
```

## 项目结构

```text
mvsep-rs/
├── src/                      # TypeScript + Vite 前端
├── src-tauri/                # Tauri 桌面后端
├── test-api/                 # Rust 核心库 (crates.io: mvsep-api-tester)
│   ├── src/db/               # 数据库层
│   ├── src/file_transfer.rs  # 文件传输（上传/下载）
│   └── src/utils/            # 工具函数
├── docs/                     # 架构文档和 ADR
└── manifest/                 # 迁移批次状态
```

## API 参考

详细文档请访问 [docs.rs](https://docs.rs/mvsep-api-tester)。

## 反馈问题

如遇问题：
1. 查看 **日志页** 了解详细错误信息
2. 访问 [GitHub Issues](https://github.com/AntheaLaffy/mvsep-rs/issues) 报告

## 许可证

Apache License 2.0
