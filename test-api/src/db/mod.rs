//! Database module
//!
//! Manages three separate SQLite databases:
//!
//! - [`Database`] - Main database for algorithm caching
//! - [`tasks_db::TasksDatabase`] - Task database tracking the task lifecycle
//! - [`user_config::UserConfigDB`] - User configuration database for key-value storage
//!
//! # Connection management
//!
//! All databases use WAL mode and a five-second busy timeout for concurrent access from multiple threads.

pub mod migrations;
pub mod repositories;
pub mod tasks_db;
pub mod user_config;

use rusqlite::Connection;
use std::sync::Mutex;

/// Main database connection (algorithm cache)
///
/// Manages algorithms, algorithm fields, output formats, presets, and task history.
/// The default path comes from [`crate::utils::paths::db_path`].
///
/// # Examples
///
/// ```rust,no_run
/// use mvsep_api_tester::db;
///
/// let db = db::Database::new(None).unwrap();
/// let algorithms = db.with_conn(|conn| {
///     db::repositories::get_all_algorithms(conn)
/// }).unwrap();
/// ```
pub struct Database {
    /// SQLite connection protected by a mutex
    pub conn: Mutex<Connection>,
}

impl Database {
    /// Create a database connection
    ///
    /// # Parameters
    ///
    /// - `db_path`: Database file path; `None` selects the default path
    ///
    /// # Returns
    ///
    /// `Result<Self>` - Database instance or error
    pub fn new(db_path: Option<&str>) -> anyhow::Result<Self> {
        let path = db_path
            .map(|p| p.to_string())
            .unwrap_or_else(|| crate::utils::paths::db_path().to_string_lossy().to_string());

        let conn = Connection::open(&path)?;
        conn.execute_batch("PRAGMA journal_mode=WAL")?;
        conn.execute_batch("PRAGMA foreign_keys=ON")?;
        conn.execute_batch("PRAGMA busy_timeout=5000")?;

        let db = Self {
            conn: Mutex::new(conn),
        };

        {
            let locked = db
                .conn
                .lock()
                .map_err(|e| anyhow::anyhow!("Poison error: {}", e))?;
            migrations::run_migrations(&locked)?;
        }

        Ok(db)
    }

    /// Execute a closure with the database connection
    ///
    /// Acquires the lock and runs the closure, converting lock poisoning into an error.
    ///
    /// # Parameters
    ///
    /// - `f`: Closure accepting `&Connection` and returning `anyhow::Result<T>`
    ///
    /// # Returns
    ///
    /// `anyhow::Result<T>` - Closure return value or error
    pub fn with_conn<F, T>(&self, f: F) -> anyhow::Result<T>
    where
        F: FnOnce(&Connection) -> anyhow::Result<T>,
    {
        let conn = self
            .conn
            .lock()
            .map_err(|e| anyhow::anyhow!("Database lock poisoned: {}", e))?;
        f(&conn)
    }

    /// Get the default database path
    ///
    /// # Returns
    ///
    /// `String` - Default database file path
    pub fn default_path() -> String {
        crate::utils::paths::db_path().to_string_lossy().to_string()
    }
}
