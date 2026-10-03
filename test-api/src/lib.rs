//! MVSep API tester - Rust implementation of the music separation API
//!
//! Provides algorithm caching, task management, streaming transfers, and resumable downloads.
//!
//! # Three-database architecture
//!
//! | Database | Location | Contents |
//! |--------|------|------|
//! | `mvsep.db` | [`db::Database`] | Algorithm cache (algorithms, fields, formats, associations) |
//! | `tasks.db` | [`db::tasks_db::TasksDatabase`] | Task tracking (tasks, history, download progress) |
//! | `user_config.db` | [`db::user_config::UserConfigDB`] | User configuration (tokens, proxies, presets) |
//!
//! # Quick start
//!
//! ```rust,no_run
//! use mvsep_api_tester::db;
//! use mvsep_api_tester::file_transfer;
//! use std::path::Path;
//!
//! // Open the main database (algorithm cache)
//! let db = db::Database::new(None).unwrap();
//! let tasks_db = db::tasks_db::TasksDatabase::new(None).unwrap();
//!
//! // Read the algorithm list
//! let algos = db.with_conn(|c| {
//!     db::repositories::get_all_algorithms(c)
//! }).unwrap();
//!
//! // Stream a file download (blocking version)
//! let client = reqwest::blocking::Client::new();
//! file_transfer::download_file(
//!     &client,
//!     "https://example.com/file.wav",
//!     Path::new("./output.wav"),
//!     0, // resume_from = 0 starts from the beginning
//!     |p| println!("{:.1}%", p.percent),
//! ).unwrap();
//! ```
//!
//! # Async example
//!
//! ```rust,no_run
//! use mvsep_api_tester::file_transfer;
//! use std::path::Path;
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     let client = reqwest::Client::new();
//!
//!     // Asynchronous upload
//!     let hash = file_transfer::upload_file_async(
//!         &client,
//!         "https://api.mvsep.com/upload",
//!         Path::new("./song.mp3"),
//!         vec![("api_token".to_string(), "your-token".to_string())],
//!         None,
//!         |p| println!("Upload: {:.1}%", p.percent),
//!     ).await?;
//!     println!("Task hash: {}", hash);
//!
//!     // Asynchronous download with resume support
//!     file_transfer::download_file_async(
//!         &client,
//!         "https://api.mvsep.com/download/file.wav",
//!         Path::new("./output.wav"),
//!         "remote_file.wav",
//!         None,
//!         |p| println!("Download: {:.1}%", p.percent),
//!     ).await?;
//!
//!     Ok(())
//! }
//! ```
//!
//! # Module overview
//!
//! | Module | Description |
//! |------|------|
//! | [`db`] | Database access and migrations for the three databases |
//! | [`db::tasks_db`] | Task database (separate SQLite database) |
//! | [`db::user_config`] | User configuration key-value storage |
//! | [`db::repositories`] | Data access layer (row types and CRUD operations) |
//! | [`file_transfer`] | File transfers (streaming uploads/downloads, resume, progress callbacks) |
//! | [`utils`] | Path and console utilities |

pub mod db;
pub mod file_transfer;
pub mod utils;
