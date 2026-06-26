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
│   │   │   ├── tasks_db.rs  # 任务数据库
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

## 三数据库架构

| 数据库 | 位置 | 内容 |
|--------|------|------|
| **`mvsep.db`** | 远端缓存 | 算法列表、字段、格式定义、算法-格式关联 |
| **`tasks.db`** | 任务追踪 | 任务记录、任务历史、下载进度、输出文件清单 |
| **`user_config.db`** | 用户配置 | Token、代理、输出目录、预设、缓存元数据 |

三者独立，通过 hash / ID 关联。

---

## 预设系统

预设是一组"算法 + 参数 + 格式"的快捷保存，存储在 `user_config.db` 中（JSON 格式）。

### 创建预设

| 入口 | 触发方式 | 说明 |
|------|---------|------|
| `[b]` 浏览算法 | 按 `s` | 选算法 ID → 命名保存 |
| `[3]` 创建任务时 | 按 `s`（上传前） | 自动保存当前全部参数 |
| `[c]` → `[s]` | 直接按 | 手动输入算法/格式/参数 |

### 加载预设

| 入口 | 触发方式 |
|------|---------|
| `[3]` 创建任务时 | 在 `Sep Type ID` 提示处输入 `l` |

### 删除预设

`[c]` → `[d]` → 选择预设名称删除。

---

## 任务数据库（tasks.db）

任务生命周期通过 `tasks.db` 追踪，每条任务记录包含：

```
hash          — API 返回的唯一标识
file_name     — 上传的原始文件名
algorithm_id  — 使用的算法 ID
format        — 输出格式 ID
status        — 当前状态（uploaded / queued / processing / done / expired / failed / cancelled）
output_files  — JSON 数组，记录每个产物文件的下载状态
```

### 任务状态流转

```
uploaded → queued → processing → done
                    → failed
                              → expired（文件过期不可下载）
```

### 输出文件追踪

`output_files` 字段存 JSON，记录每个产物的远程 URL、本地路径、下载状态：

```json
[
  {"remote_name": "vocals.flac", "url": "https://...", "size": 74290000, "downloaded": true, "local_path": "/output/xxx_vocals.flac"},
  {"remote_name": "other.flac",  "url": "https://...", "size": 74290000, "downloaded": false, "local_path": null}
]
```

- 下载时自动跳过已完成的文件
- 如果文件被删除但 DB 标记为已下载，会检测到磁盘不存在后重新下载

---

## 用户配置数据库（user_config.db）

基于 KV 存储的 `UserConfigDB`，所有配置项以 key-value 形式存储：

### 配置项清单

| Key | 类型 | 说明 | 默认值 |
|-----|------|------|--------|
| `token` | string | API 令牌 | 空 |
| `proxy_mode` | string | 代理模式（system/manual/none） | `system` |
| `proxy_host` | string | 代理主机 | `127.0.0.1` |
| `proxy_port` | string | 代理端口 | `7897` |
| `output_dir` | string | 下载输出目录 | `./output` |
| `output_format` | int | 默认输出格式 ID | `1`（WAV 16bit） |
| `premium_enabled` | int | Premium 模式开关 | `0` |
| `long_filenames_enabled` | int | 长文件名开关 | `0` |
| `algorithm_last_fetched_at` | int | 算法缓存最后拉取时间（Unix 时间戳） | 无 |
| `algorithm_auto_refresh_days` | int | 算法缓存自动刷新天数 | `15` |
| `preset:{名称}` | json | 用户预设（算法+参数+格式） | — |

### API 方式

```rust
use mvsep_api_tester::db::user_config::UserConfigDB;

let ucfg = UserConfigDB::new("path/to/user_config.db")?;

// 读写
ucfg.set_string("token", "xxx")?;
let token = ucfg.get_string("token")?;

ucfg.set_int("output_format", 2)?;
let fmt = ucfg.get_int("output_format")?;

// 预设（JSON）
let preset = serde_json::json!({"name":"vocal","algorithm_id":48});
ucfg.set_json("preset:my_vocal", &preset)?;
let loaded: serde_json::Value = ucfg.get_json("preset:my_vocal")?.unwrap();
```

---

## 功能

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
| **预设系统** | 保存/加载常用算法配置组合 |

---

## 快速开始

```bash
cd test-api
cargo run --release
```

首次使用：
1. 按 `2` 设置 API Token（从 https://mvsep.com/user-api 获取）
2. 按 `p` 配置代理（如需要）
3. 按 `r` 从 API 拉取算法缓存
4. 按 `3` 创建第一个分离任务

### 菜单

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
