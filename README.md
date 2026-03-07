# MVSEP GUI (mvsep-rs)

语言 / Languages: 简体中文 | [English](./README.en.md) | [日本語](./README.ja.md)

基于 **Tauri 2 + TypeScript + Rust** 的 MVSEP 桌面客户端，用于音频人声/伴奏分离任务的创建、轮询与结果下载。

- 项目主页: https://github.com/AntheaLaffy/mvsep-rs
- MVSEP: https://mvsep.com

## 给普通用户

你不需要安装 Rust/Node，只需下载已经打包好的程序即可。

### 下载安装

- Windows: 下载 `MVSEP-Setup.exe`（或便携版 `MVSEP.exe`）
- Linux: 下载 `MVSEP.AppImage`（或 `.deb`/`.rpm`，按你提供的包格式）
- 推荐发布位置: GitHub Releases  
  `https://github.com/AntheaLaffy/mvsep-rs/releases`

### 首次使用（3 分钟）

1. 打开应用，进入「设置」页面
2. 填写 API Token（获取地址：`https://mvsep.com/user-api`）
3. API 地址保持默认 `https://mvsep.com`
4. 点击「测试连接」
5. 设置输出目录（分离后的文件保存位置）

### 日常使用流程

1. 在首页拖入音频文件
2. 选择算法、模型参数和输出格式
3. 点击「一键运行」  
   或点击「创建任务」（只创建不自动下载）
4. 在「任务」页查看进度并下载结果

### 常见问题（用户）

#### 1) 提示连接失败

- 检查 Token 是否正确
- 检查网络/代理设置
- 在设置中重新点击「测试连接」

#### 2) 下载中断

再次点击下载即可继续，应用支持断点续传。

#### 3) Linux 无法双击启动 AppImage

先给执行权限后再运行：

```bash
chmod +x MVSEP.AppImage
./MVSEP.AppImage
```

## 功能特性

- 拖拽/选择音频文件并创建分离任务
- 一键运行（创建任务 -> 轮询状态 -> 自动下载）
- 任务中心（进行中 / 已完成 / 历史）
- 实时上传与下载进度（速度、百分比、文件名）
- 下载中断续传与手动取消下载
- 算法列表本地缓存与远程刷新
- 输出格式选择（MP3/WAV/FLAC/M4A 等）
- 连接测试与代理设置（系统/手动/无代理）
- 前后端日志查看与导出
- 多语言界面（中文 / English / 日本語）

## 技术栈

- 前端: Vite + TypeScript + TailwindCSS
- 桌面壳: Tauri 2
- 后端: Rust + Tokio + Reqwest
- 插件: dialog / fs / log / opener / shell

## 环境要求

- Node.js 18+
- Rust stable（建议通过 `rustup` 安装）
- Tauri 2 构建依赖（按你的系统安装）

Linux 常见依赖（示例，按发行版调整）：

```bash
# Debian/Ubuntu 示例
sudo apt install -y \
  libwebkit2gtk-4.1-dev \
  libgtk-3-dev \
  libayatana-appindicator3-dev \
  librsvg2-dev \
  patchelf
```

## 给开发者

## 快速开始

```bash
# 1) 安装前端依赖
npm install

# 2) 启动桌面开发模式
npm run tauri dev
```

首次使用建议在「设置」页完成：

1. 填写 API Token（可从 https://mvsep.com/user-api 获取）
2. 确认 API 地址（默认 `https://mvsep.com`）
3. 测试连接
4. 设置输出目录与代理（如需要）

## 构建与打包

```bash
# 前端构建
npm run build

# Tauri 打包（平台默认 bundle）
npm run tauri build

# Linux AppImage（仓库提供的辅助脚本）
npm run build:appimage
```

生成产物一般位于：`src-tauri/target/release/bundle/`。

## 配置与缓存位置

应用会将配置和算法缓存写入系统配置目录下的 `mvsep-gui` 文件夹：

- `config.json`：用户设置（Token、API、代理、输出目录等）
- `algorithms_cache.json`：算法列表与详情缓存

常见路径：

- Linux: `~/.config/mvsep-gui/`
- macOS: `~/Library/Application Support/mvsep-gui/`
- Windows: `%APPDATA%\\mvsep-gui\\`

## 项目结构

```text
.
├── src/                    # 前端（页面渲染、事件、状态）
├── src-tauri/              # Tauri/Rust 后端与打包配置
├── scripts/                # 构建辅助脚本
└── README.md
```

## 常见问题

### 1) 无法获取算法列表 / 连接失败

- 先在「设置」页点击“测试连接”
- 检查 Token、API 地址是否正确
- 如在代理网络环境，尝试切换代理模式或填写手动代理

### 2) 下载中断

应用支持断点续传。再次点击下载会尝试从中断位置继续。

### 3) Linux 打包报 GTK/GDK 相关错误

优先使用 `npm run build:appimage`，该脚本已包含部分兼容处理（`gdk-pixbuf` 缓存与 `PKG_CONFIG_PATH` 设置）。

## 许可证

本仓库包含 Apache License 2.0 许可证文件，详见 [LICENSE](./LICENSE)。
