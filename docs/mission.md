# Mission

mvsep-rs 的目标是把 MVSep API 交互能力沉淀为可靠的 Rust 后端库，并让 Tauri 桌面客户端逐步从旧的内嵌后端逻辑切换到这个新后端。

## Goals

- 任意迁移阶段应用仍可运行、可验证、可回滚。
- 保持现有前端 Tauri command 名称和进度事件稳定，先替换 command 后面的实现。
- 将配置、算法缓存、任务、日志、文件传输逐步迁移到 `test-api` 库化后的能力。
- 为每一批迁移建立行为一致性测试、错误追踪审查、人体工学审查和代码质量审查。

## Non Goals

- 不重写远端 MVSep 服务。
- 不在后端替换主线中升级 Tailwind v4。
- 不在第一阶段做大规模视觉重设计或前端框架迁移。
- 不让前端页面直接理解数据库表或 repository 形状。
- 不套用 py2rs 的 Python/Rust 目录、runtime router 或脚本迁移单元架构。

## Strategy

采用小型化的 strangler facade 策略：

1. 在 Tauri command 后建立 `AppBackend` 接口。
2. 先用 `LegacyMainBackend` 包住旧实现，行为不变。
3. 按批次引入 `TestApiBackend`。
4. 每批通过行为一致性、错误追踪、异步/人体工学、数据结构/算法和代码风格审查后再推进下一批。
