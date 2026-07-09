# mvsep-rs

mvsep-rs 是面向 MVSep 音频分离流程的 Tauri 2 桌面客户端和 Rust 后端重写工程。仓库同时包含桌面 UI、Tauri command facade，以及从 `test-api` 抽取并稳定下来的 Rust API/后端能力：配置、算法缓存、上传下载、任务持久化和下载状态。

当前重写策略是以后端为主：已经迁移到新后端的领域，以新后端存储为准。旧前端存储只作为迁移和回滚辅助；如果同一个任务、历史记录或配置同时存在于旧存储和新后端中，除非某个迁移记录明确写了不同冲突规则，否则以新后端为准。

## 当前状态

- `manifest/rewrite-status.yaml` 中的所有迁移批次都已验证。
- `src/app/backend/gateway.ts` 是前端唯一允许导入 Tauri JavaScript API、调用 `invoke` 或调用 `listen` 的模块。
- Tauri command 名称和进度事件名称保持稳定，后端实现细节通过 `AppBackend` 替换。
- 配置、输出格式、算法缓存、上传/下载传输、活动任务和任务历史都已经放到后端 facade 后面。
- 后端路径由 Tauri 注入的 app config/data 路径解析，不再以进程 cwd、仓库根目录或旧程序本体相对路径为主线依据。

## 仓库结构

```text
.
├── src/                       # TypeScript + Vite 前端
├── src-tauri/                 # Tauri 桌面后端和 AppBackend facade
├── test-api/                  # 抽取的 Rust MVSep API/后端层和 CLI 测试入口
├── docs/                      # 架构、使命、ADR 和文档索引
├── manifest/                  # 机器可读的迁移批次状态
├── rewrite-records/           # 持久化迁移经验和边界决策
├── reviews/                   # 各批次审查报告
├── doc/                       # 本地 MVSep API 笔记
└── scripts/                   # 构建脚本
```

## 快速开始

安装 JavaScript 和 Rust 依赖后，可以启动前端或 Tauri 应用：

```bash
npm install
npm run dev
npm run tauri dev
```

构建前端：

```bash
npm run build
```

构建 AppImage：

```bash
npm run build:appimage
```

运行独立 Rust CLI 测试入口：

```bash
cd test-api
cargo run --release
```

## 验证命令

后端重写相关改动完成后，使用这些基线检查：

```bash
npm run build
cd src-tauri && cargo test
cd src-tauri && cargo clippy --all-targets -- -D warnings
cd test-api && cargo test
cd test-api && cargo clippy --all-targets -- -D warnings
```

前端必须保持 Tauri API 访问集中化：

```bash
rg -n "\binvoke\b|\blisten\b|@tauri-apps" src --glob '*.ts'
```

严格结果应该只有 `src/app/backend/gateway.ts` 命中。

## 路径规则

后端路径从 Tauri app config/data 目录注入。主要数据库位于注入的 app data 目录下：

- `mvsep.db`
- `user_config.db`
- `tasks.db`

上传源文件路径保持用户选择的本地文件路径。下载输出目录可以是绝对路径；`./output` 这类相对输出路径会解析到注入的 app data 目录下，不会解析到旧后端二进制位置、仓库根目录或当前 cwd。

下载产物的本地路径记录保存在新后端任务/历史数据中。前端应该读取和展示这些后端记录，不要从旧 localStorage 重新推导下载路径。

## 文档入口

- `docs/INDEX.md`: 智能体和维护者的主入口。
- `docs/mission.md`: 目标、非目标和重写策略。
- `docs/architecture/backend-rewrite.md`: 已接受的后端重写架构。
- `manifest/rewrite-status.yaml`: 批次状态和审查门。
- `CONTEXT.md`: 项目术语表。
- `Note.md`: 人类工作笔记和长期偏好。
- `RESOURCES.md`: 高信度资料和借鉴边界。
- `rewrite-records/`: 非显然迁移决策和经验。
- `reviews/`: 行为、追踪、异步、风格、数据和 UX 审查报告。

## 生成文件

`dist/`、`node_modules/` 和 Vite 缓存都是本地生成物，不属于源码提交范围。需要时从 `package-lock.json` 和源码重新生成。

## 许可证

Apache License 2.0
