# MVSEP - 音楽分離ツール

MVSEP デスクトップクライアント。音楽をボーカル、伴奏、ドラム、ベースなどのトラックに分離します。ドラッグアンドドロップアップロード、ワンクリック操作、タスク管理、再開可能なダウンロードに対応しています。

[![License](https://img.shields.io/crates/l/mvsep-api-tester.svg)](https://crates.io/crates/mvsep-api-tester)
[![Crates.io](https://img.shields.io/crates/v/mvsep-api-tester.svg)](https://crates.io/crates/mvsep-api-tester)
[![Crates.io](https://img.shields.io/crates/v/mvsep-gui.svg)](https://crates.io/crates/mvsep-gui)
[![Docs](https://docs.rs/mvsep-api-tester/badge.svg)](https://docs.rs/mvsep-api-tester)

言語: [中文](README.md) | [English](README.en.md) | [日本語](README.ja.md)

## 機能

### ユーザー機能
- **ドラッグアンドドロップ** - ウィンドウにオーディオファイルをドラッグして処理を開始
- **ワンクリック操作** - アップロード → 分離待ち → 自動ダウンロード、手動操作不要
- **タスク管理** - リアルタイムで分離進捗を確認、中断・ダウンロード・削除に対応
- **複数アルゴリズム** - 複数の分離アルゴリズムとモデルに対応
- **再開可能なダウンロード** - 中断後も再びダウンロードボタンをクリックして続行可能
- **プロキシサポート** - システムプロキシ、手動プロキシ、プロキシなしの三つのモード

### 技術的特徴
- **三層データベースアーキテクチャ**: アルゴリズムキャッシュ、タスク追跡、ユーザー設定を独立して管理
- **ストリーミングアップロード**: tokio ベースの非同期ファイルアップロード、進捗コールバックとキャンセルに対応
- **タスク永続化**: 完全なタスクライフサイクル管理と履歴記録

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

### Windows

`MVSEP_1.2.0_x64-setup.exe` をダウンロードしてインストーラーを実行します。

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

## クイックスタート

### 1. 初回設定

以下の設定が必要です：

| 設定項目 | 説明 |
|----------|------|
| **API Token** | 必須。[MVSEP ウェブサイト](https://mvsep.com/user-api) から取得 |
| **出力ディレクトリ** | 分離結果の保存先 |
| **出力フォーマット** | MP3/WAV/FLAC/M4A など |

### 2. 分離を開始

1. **ホームページ** - オーディオファイルをドラッグ、またはファイル選択をクリック
2. **アルゴリズム**と**モデルオプション**を選択（オプション）
3. **出力フォーマット**を選択
4. **ワンクリック実行**をクリック、完了後自動的にローカルにダウンロード

### 3. タスクを確認

- **タスクページ** - 実行中および履歴タスクを確認
- **ダウンロード**をクリックして個別のファイルをダウンロード
- 実行中のタスクを**キャンセル**可能

## ページ概要

| ページ | 機能 |
|--------|------|
| ホーム | オーディオアップロード、パラメータ選択、ワンクリック実行 |
| タスク | 進捗確認、結果ダウンロード、タスク管理 |
| アルゴリズム | 使用可能なアルゴリズムとモデルの閲覧、プリセットの保存 |
| 設定 | API Token、プロキシ、出力ディレクトリなどの設定 |
| ログ | 実行ログの確認、問題解決に使用 |

## FAQ

### API Token はどこで取得できますか？

1. [MVSEP](https://mvsep.com) にログイン
2. 右上のユーザー名をクリック → **API** を選択
3. Token をコピーしてクライアントの設定ページに貼り付け

### 分離速度が遅いですか？

- **タスクページ**でキュー情報を確認し、現在の待機人数を確認
- 異なるアルゴリズムに切り替えると処理速度が向上する場合があります
- デモモード（無料だが結果は公開）を検討してください

### ダウンロードが中断されましたか？

心配しないでください。クライアントは**再開可能なダウンロード**をサポートしています。ダウンロードボタンをもう一度クリックするだけで中断箇所から続行できます。

### アルゴリズムリストを更新するには？

**アルゴリズムページ**に移動し、「最新アルゴリズム情報を取得」をクリックしてサーバーから取得します。

## 開発者ガイド

### 開発モード

```bash
npm install
npm run tauri dev
```

### AppImage をビルド

```bash
npm run build:appimage
```

### データベース操作（Rust）

```rust
use mvsep_api_tester::db;

let db = db::Database::new(None)?;
let algorithms = db.with_conn(|conn| {
    db::repositories::get_all_algorithms(conn)
})?;
```

### ファイルアップロード（Rust）

```rust
use mvsep_api_tester::file_transfer::{self, TransferProgress};

let hash = file_transfer::upload_file_async(
    &client, "https://api.mvsep.com/upload",
    std::path::Path::new("./song.mp3"),
    vec![("api_token", "your-token".to_string())],
    None, |progress| {
        println!("Upload: {:.1}%", progress.percent);
    },
).await?;
```

## プロジェクト構造

```text
mvsep-rs/
├── src/                      # TypeScript + Vite フロントエンド
├── src-tauri/                # Tauri デスクトップバックエンド
├── test-api/                 # Rust コアライブラリ (crates.io: mvsep-api-tester)
│   ├── src/db/               # データベース層
│   ├── src/file_transfer.rs  # ファイル転送（アップロード/ダウンロード）
│   └── src/utils/            # ユーティリティ関数
├── docs/                     # アーキテクチャドキュメントと ADR
└── manifest/                 # マイグレーションバッチステータス
```

## API リファレンス

詳細なドキュメントは [docs.rs](https://docs.rs/mvsep-api-tester) をご覧ください。

## フィードバック

問題が発生した場合：
1. **ログページ**で詳細なエラー情報を確認
2. [GitHub Issues](https://github.com/AntheaLaffy/mvsep-rs/issues) で報告

## ライセンス

Apache License 2.0
