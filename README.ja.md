# mvsep-rs

<p align="center">
  <a href="https://www.rustacean.net/">
    <img src="https://www.rustacean.net/assets/rustacean-orig-noshadow.svg" alt="rustacean.net の Rust マスコット Ferris" width="96">
  </a>
</p>

<p align="center">
  Ferris 画像: <a href="https://www.rustacean.net/">rustacean.net</a>
</p>

[![License](https://img.shields.io/crates/l/mvsep-api-tester.svg)](https://crates.io/crates/mvsep-api-tester)
[![Crates.io](https://img.shields.io/crates/v/mvsep-api-tester.svg)](https://crates.io/crates/mvsep-api-tester)
[![Crates.io](https://img.shields.io/crates/v/mvsep-gui.svg)](https://crates.io/crates/mvsep-gui)
[![Docs](https://docs.rs/mvsep-api-tester/badge.svg)](https://docs.rs/mvsep-api-tester)

言語: [中文](README.md) | [English](README.en.md) | [日本語](README.ja.md)

mvsep-rs は 1.2 版で MVSep バックエンドをリファクタリングし、Tauri 部分も更新しました。このリポジトリには、デスクトップ UI、Tauri command facade、そして `test-api` から抽出して安定化した Rust API/バックエンド機能が含まれます。対象は設定、アルゴリズムキャッシュ、アップロード/ダウンロード転送、タスク永続化、ダウンロード状態です。

現在のリライト方針は新バックエンド優先です。すでに新バックエンドへ移行した領域では、新バックエンドのストアを正とします。従来のフロントエンド保存は、移行とロールバックの補助に限ります。同じタスク、履歴、設定が旧ストレージと新バックエンドの両方に存在する場合、移行記録で別の競合規則が明示されていない限り、新バックエンドを優先します。

## 現在の状態

- `manifest/rewrite-status.yaml` にあるすべての移行バッチは検証済みです。
- プロジェクトのバージョンメタデータは `1.2.0` に同期済みで、Tauri 部分は Tauri 2 を使用しています。
- `src/app/backend/gateway.ts` は、Tauri JavaScript API の import、`invoke`、`listen` を許可されている唯一のフロントエンドモジュールです。
- Tauri command 名と進捗イベント名は安定させたまま、バックエンド実装の詳細を `AppBackend` 経由で置き換えます。
- 設定、出力形式、アルゴリズムキャッシュ、アップロード/ダウンロード転送、アクティブタスク、タスク履歴はバックエンド facade の背後にあります。
- バックエンドパスは Tauri から注入された app config/data パスで解決します。プロセス cwd、リポジトリルート、旧プログラム本体からの相対パスは主線の基準ではありません。

## リポジトリ構成

```text
.
├── src/                       # TypeScript + Vite フロントエンド
├── src-tauri/                 # Tauri デスクトップバックエンドと AppBackend facade
├── test-api/                  # 抽出済み Rust MVSep API/バックエンド層と CLI テスト入口
├── docs/                      # アーキテクチャ、ミッション、ADR、ドキュメント索引
├── manifest/                  # 機械可読な移行バッチ状態
├── rewrite-records/           # 永続化された移行知見と境界判断
├── reviews/                   # 各バッチのレビュー報告
├── doc/                       # ローカル MVSep API メモ
└── scripts/                   # ビルドスクリプト
```

## インストール

### Arch Linux / Manjaro (AUR)

```bash
# プリビルドバイナリ版（推奨、高速インストール）
paru -S mvsep-gui-bin
# または
yay -S mvsep-gui-bin

# ソースビルド版（Rust と Node.js が必要）
paru -S mvsep-gui
# または
yay -S mvsep-gui
```

### GitHub Release からダウンロード

```bash
# Linux
wget https://github.com/AntheaLaffy/mvsep-rs/releases/download/v1.2.0/mvsep-gui
chmod +x mvsep-gui
sudo mv mvsep-gui /usr/bin/

# Windows
# MVSEP_1.2.0_x64-setup.exe をダウンロードしてインストーラーを実行します

# Debian/Ubuntu
wget https://github.com/AntheaLaffy/mvsep-rs/releases/download/v1.2.0/MVSEP_1.2.0_amd64.deb
sudo dpkg -i MVSEP_1.2.0_amd64.deb

# Fedora/RHEL
wget https://github.com/AntheaLaffy/mvsep-rs/releases/download/v1.2.0/MVSEP-1.2.0-1.x86_64.rpm
sudo dnf install MVSEP-1.2.0-1.x86_64.rpm
```

### ソースからビルド

```bash
# 依存関係をインストール
sudo pacman -S webkit2gtk libappindicator-gtk3 librsvg libvips npm nodejs

# リポジトリをクローン
git clone https://github.com/AntheaLaffy/mvsep-rs.git
cd mvsep-rs

# フロントエンドをビルド
npm install
npm run build

# バックエンドをビルド
cd src-tauri
cargo build --release

# 実行
./target/release/mvsep-gui
```

### クイックスタート

JavaScript と Rust の依存関係をインストールしてから、フロントエンドまたは Tauri アプリを起動します。

```bash
npm install
npm run dev
npm run tauri dev
```

フロントエンドをビルドします。

```bash
npm run build
```

AppImage をビルドします。

```bash
npm run build:appimage
```

独立した Rust CLI テスト入口を実行します。

```bash
cd test-api
cargo run --release
```

## 検証コマンド

バックエンドリライト関連の変更後は、次の基線チェックを使います。

```bash
npm run build
cd src-tauri && cargo test
cd src-tauri && cargo clippy --all-targets -- -D warnings
cd test-api && cargo test
cd test-api && cargo clippy --all-targets -- -D warnings
```

フロントエンドからの Tauri API アクセスは集中化されている必要があります。

```bash
rg -n "\binvoke\b|\blisten\b|@tauri-apps" src --glob '*.ts'
```

厳密な期待結果は、`src/app/backend/gateway.ts` だけが一致することです。

## パス規則

バックエンドパスは Tauri app config/data ディレクトリから注入されます。主要データベースは注入された app data ディレクトリ配下に置かれます。

- `mvsep.db`
- `user_config.db`
- `tasks.db`

アップロード元ファイルのパスは、ユーザーが選択したローカルファイルパスを保持します。ダウンロード出力ディレクトリには絶対パスを指定できます。`./output` のような相対出力パスは、注入された app data ディレクトリ配下に解決されます。旧バックエンドの実行ファイル位置、リポジトリルート、現在の cwd には解決されません。

ダウンロード済み成果物のローカルパス記録は、新バックエンドのタスク/履歴データに保存されます。フロントエンドは旧 localStorage からダウンロードパスを再推測せず、これらのバックエンド記録を読み取って表示するべきです。

## ドキュメント入口

- `docs/INDEX.md`: エージェントとメンテナー向けの主入口。
- `docs/mission.md`: 目標、非目標、リライト戦略。
- `docs/architecture/backend-rewrite.md`: 採用済みのバックエンドリライトアーキテクチャ。
- `manifest/rewrite-status.yaml`: バッチ状態とレビューゲート。
- `CONTEXT.md`: プロジェクト用語集。
- `Note.md`: 人間向け作業メモと長期的な方針。
- `RESOURCES.md`: 高信頼資料と借用境界。
- `rewrite-records/`: 非自明な移行判断と知見。
- `reviews/`: 振る舞い、トレース、非同期、スタイル、データ、UX のレビュー報告。

## 生成ファイル

`dist/`、`node_modules/`、Vite キャッシュはローカル生成物であり、ソース管理の対象ではありません。必要に応じて `package-lock.json` とソースツリーから再生成します。

## ライセンス

Apache License 2.0
