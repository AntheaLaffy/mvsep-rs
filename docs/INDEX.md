# Project Resource Index

本目录是 mvsep-rs 后续重写、替换、审查工作的入口。任何智能体开始工作前，先读本文件，再读与任务相关的具体文档。

## Core Documents

- [Mission](mission.md): 项目目标、非目标、当前迁移策略。
- [Resources](../RESOURCES.md): 高信度资料、借鉴来源和禁止套用边界。
- [High Confidence Sources](references/high-confidence-sources.md): 资料查询优先级和官方文档入口。
- [Backend Rewrite Architecture](architecture/backend-rewrite.md): 渐进替换架构、接口缝合点、数据流和质量门。
- [ADR 0001](adr/0001-backend-rewrite-facade.md): 选择 Tauri command facade 后的 `AppBackend` 作为迁移 seam 的决策记录。
- [API Contract Skill](../skills/mvsep-api-contract/SKILL.md): Endpoint integration, response parsing, transfers, and English API documentation.
- [Language Validation Skill](../skills/mvsep-i18n-validation/SKILL.md): Startup locale selection, saved preferences, and visible translations.
- [AppImage Release Skill](../skills/mvsep-appimage-release/SKILL.md): Packaging, artifact validation, and catalog acceptance.
- [Domain Context](../CONTEXT.md): 项目术语表。
- [Working Notes](../Note.md): 视觉、人体工学、工程风格和协作约束。
- [Rewrite Status](../manifest/rewrite-status.yaml): 迁移批次状态的机器可读记录。
- [Rewrite Records](../rewrite-records/README.md): 非显然迁移经验和资料借鉴边界记录。
- [Review Reports](../reviews/README.md): 多智能体审查报告入口。

## Reusable Skills

These existing skills are copied into the repository with their supporting files and original attribution. Use them for the relevant task; project-specific skills above add MVSEP context.

| Task | Skill |
|---|---|
| Current library and CLI documentation | [find-docs](../skills/find-docs/SKILL.md) |
| Crashes, memory errors, intermittent failures | [debugging](../skills/debugging/SKILL.md) |
| Regression tests and test strategy | [testing](../skills/testing/SKILL.md) |
| Formatters and lint tooling | [code-quality-tooling](../skills/code-quality-tooling/SKILL.md) |
| Change review | [code-review](../skills/code-review/SKILL.md) |
| Commits, branches, pushes, recovery | [git-cli](../skills/git-cli/SKILL.md) |
| README, comments, commit and PR explanations | [writing-for-readers](../skills/writing-for-readers/SKILL.md) |
| GitHub Actions and build automation | [ci-cd](../skills/ci-cd/SKILL.md) |
| Upstream issues and pull requests | [contributing-upstream](../skills/contributing-upstream/SKILL.md) |
| Version numbers, dependency ranges, and lockfiles | [versioning-reproducibility](../skills/versioning-reproducibility/SKILL.md) |

Release preparation: [Version Management](version-management.md).

## Local Source Anchors

- `src-tauri/src/main.rs`: Tauri command facade、`AppBackend` seam、路径注入、任务/传输命令边界。
- `src/app/backend/gateway.ts`: 前端唯一 Tauri JavaScript API adapter。
- `test-api/src/lib.rs`: 新 Rust API 交互层的库入口。
- `test-api/src/db/`: 新后端数据库和 repository 层。
- `test-api/src/file_transfer.rs`: async upload/download、resume、cancel 和 progress 传输核心。
- `src/main.ts`: 当前前端状态、页面注册、渲染调度集中处。
- `src/app/services/tasks.ts`: 前端任务轮询、下载、取消逻辑。
- `src/app/render/`: HTML 字符串渲染层。
- [MVSep API Endpoint Reference](../doc/mvsep_api_endpoints.md): English endpoint reference captured on 2026-06-27.

## Required Baseline Checks

- Frontend: `npm run build`
- Tauri backend: `cd src-tauri && cargo test`
- Tauri backend lint: `cd src-tauri && cargo clippy --all-targets -- -D warnings`
- Rust API layer: `cd test-api && cargo test`
- Release gate after migration work: `cargo clippy -- -D warnings` for touched Rust crates, plus ignored online tests when token/proxy are available.
