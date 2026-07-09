//! MVSep API tester - 音乐分离 API 的 Rust 实现
//!
//! 提供算法缓存、任务管理、流式上传下载、断点续传等核心能力。
//!
//! # 三数据库架构
//!
//! | 数据库 | 位置 | 内容 |
//! |--------|------|------|
//! | `mvsep.db` | [`db::Database`] | 算法缓存（算法、字段、格式、关联） |
//! | `tasks.db` | [`db::tasks_db::TasksDatabase`] | 任务追踪（任务、历史、下载进度） |
//! | `user_config.db` | [`db::user_config::UserConfigDB`] | 用户配置（Token、代理、预设） |
//!
//! # 快速开始
//!
//! ```rust,no_run
//! use mvsep_api_tester::db;
//! use mvsep_api_tester::file_transfer;
//! use std::path::Path;
//!
//! // 打开主数据库（算法缓存）
//! let db = db::Database::new(None).unwrap();
//! let tasks_db = db::tasks_db::TasksDatabase::new(None).unwrap();
//!
//! // 读取算法列表
//! let algos = db.with_conn(|c| {
//!     db::repositories::get_all_algorithms(c)
//! }).unwrap();
//!
//! // 流式下载文件（阻塞版本）
//! let client = reqwest::blocking::Client::new();
//! file_transfer::download_file(
//!     &client,
//!     "https://example.com/file.wav",
//!     Path::new("./output.wav"),
//!     0, // resume_from = 0 表示从头下载
//!     |p| println!("{:.1}%", p.percent),
//! ).unwrap();
//! ```
//!
//! # 异步示例
//!
//! ```rust,no_run
//! use mvsep_api_tester::file_transfer;
//! use std::path::Path;
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     let client = reqwest::Client::new();
//!
//!     // 异步上传
//!     let hash = file_transfer::upload_file_async(
//!         &client,
//!         "https://api.mvsep.com/upload",
//!         Path::new("./song.mp3"),
//!         vec![("api_token", "your-token".to_string())],
//!         None,
//!         |p| println!("上传: {:.1}%", p.percent),
//!     ).await?;
//!     println!("任务 Hash: {}", hash);
//!
//!     // 异步下载（支持断点续传）
//!     file_transfer::download_file_async(
//!         &client,
//!         "https://api.mvsep.com/download/file.wav",
//!         Path::new("./output.wav"),
//!         "remote_file.wav",
//!         None,
//!         |p| println!("下载: {:.1}%", p.percent),
//!     ).await?;
//!
//!     Ok(())
//! }
//! ```
//!
//! # 模块概览
//!
//! | 模块 | 说明 |
//! |------|------|
//! | [`db`] | 数据库层，含三个数据库的访问和迁移 |
//! | [`db::tasks_db`] | 任务数据库（独立 SQLite） |
//! | [`db::user_config`] | 用户配置 KV 存储 |
//! | [`db::repositories`] | 数据访问层（行类型 + CRUD） |
//! | [`file_transfer`] | 文件传输（流式上传/下载、续传、进度回调） |
//! | [`utils`] | 路径、控制台等工具函数 |

pub mod db;
pub mod file_transfer;
pub mod utils;
