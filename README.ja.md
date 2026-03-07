# MVSEP GUI (mvsep-rs)

言語: [简体中文](./README.md) | [English](./README.en.md) | 日本語

**Tauri 2 + TypeScript + Rust** で構築した MVSEP デスクトップクライアントです。  
音声分離タスクの作成、進捗確認、結果ダウンロードを行えます。

- プロジェクト: https://github.com/AntheaLaffy/mvsep-rs
- MVSEP: https://mvsep.com

## 一般ユーザー向け

Rust/Node.js は不要です。ビルド済みアプリをそのまま利用できます。

### ダウンロード

- Windows: `MVSEP-Setup.exe`（またはポータブル版 `MVSEP.exe`）
- Linux: `MVSEP.AppImage`（または `.deb` / `.rpm`）
- 推奨配布先:  
  `https://github.com/AntheaLaffy/mvsep-rs/releases`

### 初期設定

1. アプリを開いて **Settings** へ移動
2. API Token を設定（`https://mvsep.com/user-api`）
3. API URL は `https://mvsep.com` のまま
4. **Test Connection** を実行
5. 出力先フォルダを設定

### 日常の使い方

1. Home で音声ファイルをドラッグ＆ドロップ
2. アルゴリズム/オプション/出力形式を選択
3. **One-click Run** を実行  
   または **Create Task**（作成のみ・自動DLなし）
4. **Tasks** で進捗確認とダウンロード

### ユーザー FAQ

#### 1) 接続失敗になる

- API Token を確認
- ネットワーク/プロキシ設定を確認
- **Test Connection** を再実行

#### 2) ダウンロードが中断した

再度ダウンロードを押せば再開できます（レジューム対応）。

#### 3) Linux で AppImage が起動しない

```bash
chmod +x MVSEP.AppImage
./MVSEP.AppImage
```

## 主な機能

- 音声ファイルのドラッグ＆ドロップ投入
- One-click run（作成 -> 監視 -> 自動ダウンロード）
- タスク管理（進行中 / 完了 / 履歴）
- アップロード/ダウンロード進捗表示
- ダウンロード中断・再開対応
- ローカルアルゴリズムキャッシュ + リモート更新
- 複数出力形式（MP3/WAV/FLAC/M4A など）
- プロキシ設定（system/manual/none）
- フロント/バックエンドログ表示
- 多言語 UI（中文 / English / 日本語）

## 技術スタック

- Frontend: Vite + TypeScript + TailwindCSS
- Desktop: Tauri 2
- Backend: Rust + Tokio + Reqwest
- Plugins: dialog / fs / log / opener / shell

## 開発環境（開発者）

- Node.js 18+
- Rust stable
- Tauri 2 のシステム依存ライブラリ

Linux 依存関係の例:

```bash
sudo apt install -y \
  libwebkit2gtk-4.1-dev \
  libgtk-3-dev \
  libayatana-appindicator3-dev \
  librsvg2-dev \
  patchelf
```

## 開発者向け

## クイックスタート

```bash
npm install
npm run tauri dev
```

## ビルドとパッケージ

```bash
npm run build
npm run tauri build
npm run build:appimage
```

成果物は通常 `src-tauri/target/release/bundle/` に生成されます。

## 設定・キャッシュ保存先

システム設定ディレクトリ内の `mvsep-gui` に保存されます:

- `config.json`
- `algorithms_cache.json`

主な保存先:

- Linux: `~/.config/mvsep-gui/`
- macOS: `~/Library/Application Support/mvsep-gui/`
- Windows: `%APPDATA%\mvsep-gui\`

## ライセンス

Apache License 2.0 を同梱しています。詳細は [LICENSE](./LICENSE) を参照してください。
