# mvsep-rs

MVSep 音乐分离工具的 Rust 后端实现，提供算法缓存、任务管理、流式上传下载、断点续传等核心能力。

[![License](https://img.shields.io/crates/l/mvsep-api-tester.svg)](https://crates.io/crates/mvsep-api-tester)
[![Crates.io](https://img.shields.io/crates/v/mvsep-api-tester.svg)](https://crates.io/crates/mvsep-api-tester)
[![Crates.io](https://img.shields.io/crates/v/mvsep-gui.svg)](https://crates.io/crates/mvsep-gui)
[![Docs](https://docs.rs/mvsep-api-tester/badge.svg)](https://docs.rs/mvsep-api-tester)

语言: [中文](README.md) | [English](README.en.md) | [日本語](README.ja.md)

## 功能特性

- **三数据库架构**：算法缓存、任务追踪、用户配置独立管理
- **流式上传**：基于 tokio 的异步文件上传，支持进度回调和取消
- **断点续传**：基于 Range 请求和 `.part` 文件的下载恢复机制
- **任务持久化**：完整的任务生命周期管理和历史记录
- **代理支持**：手动代理、系统代理、无代理三种模式

## 安装

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

### 从 GitHub Release 下载

```bash
# Linux
wget https://github.com/AntheaLaffy/mvsep-rs/releases/download/v1.2.0/mvsep-gui
chmod +x mvsep-gui
sudo mv mvsep-gui /usr/bin/

# Windows
# 下载 MVSEP_1.2.0_x64-setup.exe 并运行安装程序

# Debian/Ubuntu
wget https://github.com/AntheaLaffy/mvsep-rs/releases/download/v1.2.0/MVSEP_1.2.0_amd64.deb
sudo dpkg -i MVSEP_1.2.0_amd64.deb

# Fedora/RHEL
wget https://github.com/AntheaLaffy/mvsep-rs/releases/download/v1.2.0/MVSEP-1.2.0-1.x86_64.rpm
sudo dnf install MVSEP-1.2.0-1.x86_64.rpm
```

### 源码构建

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

### 开发模式

```bash
npm install
npm run tauri dev
```

## 快速开始

### 数据库操作

```rust
use mvsep_api_tester::db;

// 打开主数据库（算法缓存）
let db = db::Database::new(None)?;

// 读取所有算法
let algorithms = db.with_conn(|conn| {
    db::repositories::get_all_algorithms(conn)
})?;

// 打开任务数据库
let tasks_db = db::tasks_db::TasksDatabase::new(None)?;

// 读取用户配置
let config_db = db::user_config::UserConfigDB::default()?;
let token = config_db.get_string("api_token")?;
```

### 文件上传

```rust
use mvsep_api_tester::file_transfer::{self, TransferProgress};

let client = reqwest::Client::new();
let hash = file_transfer::upload_file_async(
    &client,
    "https://api.mvsep.com/upload",
    std::path::Path::new("./song.mp3"),
    vec![("api_token", "your-token".to_string())],
    None,
    |progress: TransferProgress| {
        println!("上传: {:.1}%", progress.percent);
    },
).await?;
```

### 文件下载

```rust
use mvsep_api_tester::file_transfer::{self, TransferProgress};

let client = reqwest::Client::new();
file_transfer::download_file_async(
    &client,
    "https://api.mvsep.com/download/file.wav",
    std::path::Path::new("./output/vocals.wav"),
    "remote_file_name.wav",
    None,
    |progress: TransferProgress| {
        println!("下载: {:.1}%", progress.percent);
    },
).await?;
```

## 项目结构

```text
mvsep-rs/
├── src/                      # TypeScript + Vite 前端
├── src-tauri/                # Tauri 桌面后端
│   └── src/lib.rs            # AppBackend facade
├── test-api/                 # Rust 核心库 (crates.io: mvsep-api-tester)
│   ├── src/lib.rs            # Crate 入口和公共 API
│   ├── src/db/               # 数据库层
│   │   ├── mod.rs            # 主数据库（算法缓存）
│   │   ├── tasks_db.rs       # 任务数据库
│   │   ├── user_config.rs    # 用户配置存储
│   │   └── repositories.rs   # 数据访问层
│   ├── src/file_transfer.rs  # 文件传输（上传/下载）
│   └── src/utils/            # 工具函数
├── docs/                     # 架构文档和 ADR
└── manifest/                 # 迁移批次状态
```

## API 参考

详细文档请访问 [docs.rs](https://docs.rs/mvsep-api-tester)。

### 数据库模块

- [`db::Database`](https://docs.rs/mvsep-api-tester/latest/mvsep_api_tester/db/struct.Database.html) - 主数据库连接（算法缓存）
- [`db::tasks_db::TasksDatabase`](https://docs.rs/mvsep-api-tester/latest/mvsep_api_tester/db/tasks_db/struct.TasksDatabase.html) - 任务数据库
- [`db::user_config::UserConfigDB`](https://docs.rs/mvsep-api-tester/latest/mvsep_api_tester/db/user_config/struct.UserConfigDB.html) - 用户配置存储

### 文件传输模块

- [`file_transfer::upload_file_async`](https://docs.rs/mvsep-api-tester/latest/mvsep_api_tester/file_transfer/fn.upload_file_async.html) - 异步文件上传
- [`file_transfer::download_file_async`](https://docs.rs/mvsep-api-tester/latest/mvsep_api_tester/file_transfer/fn.download_file_async.html) - 异步文件下载（支持断点续传）
- [`file_transfer::TransferProgress`](https://docs.rs/mvsep-api-tester/latest/mvsep_api_tester/file_transfer/struct.TransferProgress.html) - 传输进度信息

## 许可证

Apache License 2.0
