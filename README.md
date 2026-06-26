# mvsep-rs

> MVSep 后端重构 · MVP 阶段

MVSep（https://mvsep.com）是一个在线音频分离平台。**mvsep-rs** 是对其后端 API 交互的 Rust 重构，包含一个独立的 CLI 测试工具（`test-api`）和原有的 Tauri 桌面客户端。

本仓库的目标是：用 Rust 实现 MVSep 完整 API 交互层，包括算法缓存、流式上传下载、断点续传、任务管理等核心能力，为后续前端/桌面端提供可靠的基础库。

---

## 项目结构

```
.
├── test-api/              # 📦 API 测试工具（主项目，MVP 阶段）
│   ├── src/
│   │   ├── main.rs         # 交互式 CLI（菜单驱动）
│   │   ├── file_transfer.rs # 文件传输模块（流式上传下载、断点续传）
│   │   ├── db/             # 数据库层（SQLite）
│   │   │   ├── migrations.rs
│   │   │   ├── repositories.rs
│   │   │   └── user_config.rs
│   │   └── utils/
│   └── tests/
├── src/                   # 🖥️ 原有前端（TypeScript + Vite）
├── src-tauri/             # 🖥️ 原有 Tauri 桌面客户端（Rust）
├── doc/                   # 📖 API 参考文档
└── scripts/               # 构建脚本
```

---

## test-api — CLI 测试工具

当前 MVP 的核心交付物，提供完整的交互式命令行界面：

### 功能

| 功能 | 说明 |
|------|------|
| **算法缓存** | 从 API 拉取算法列表和参数选项，本地缓存带过期检查 |
| **创建任务** | 选择算法、模型参数、输出格式，流式上传带进度显示 |
| **轮询状态** | 自动/手动轮询，区分排队/分离中/完成/过期/失败 |
| **断点续传下载** | 流式下载，支持中断恢复，按原文件名重命名产物 |
| **取消任务** | 取消正在处理的任务 |
| **任务管理** | 本地数据库追踪所有任务，按文件粒度跟踪下载状态 |
| **用户偏好** | Token、代理、输出目录、默认格式等配置管理 |
| **算法浏览** | 按分组浏览所有算法，标注免费/Premium |

### 快速开始

```bash
cd test-api
cargo run --release
```

首次使用：
1. 按 `2` 设置 API Token（从 https://mvsep.com/user-api 获取）
2. 按 `p` 配置代理（如需要）
3. 按 `r` 从 API 拉取算法缓存
4. 按 `3` 创建第一个分离任务

### 菜单预览

```
[l] Logout
[p] Configure Proxy
[c] User Preferences
[3] Create Task ⭐
[t] List Tasks (from DB)
[o] Operate Task (enter hash)
──────────────
[b] Browse Algorithms (from DB)
[h] API Reference
[9] Get User Info
[a] Run All Tests
[r] Refresh Algorithm Cache
[q] Quit
```

### 数据库

采用双数据库设计：

| 数据库 | 内容 |
|--------|------|
| **`mvsep.db`** | 远端/API 数据（算法、字段、格式定义、算法-格式关联） |
| **`user_config.db`** | 用户偏好（Token、代理、输出目录、缓存元数据） |

---

## 技术栈

- **语言**: Rust (edition 2021)
- **HTTP 客户端**: reqwest（async + blocking）
- **异步运行时**: tokio
- **数据库**: SQLite (rusqlite)
- **桌面壳**: Tauri 2（原有 GUI）

---

## 环境要求

- Rust stable（推荐 `rustup` 安装）
- SQLite（bundled，无需单独安装）

---

## 许可证

Apache License 2.0
