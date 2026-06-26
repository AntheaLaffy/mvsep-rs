pub mod migrations;
pub mod repositories;
pub mod tasks_db;
pub mod user_config;

use rusqlite::Connection;
use std::sync::Mutex;

pub struct Database {
    pub conn: Mutex<Connection>,
}

impl Database {
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
            let locked = db.conn.lock().map_err(|e| anyhow::anyhow!("Poison error: {}", e))?;
            migrations::run_migrations(&locked)?;
        }

        Ok(db)
    }

    pub fn with_conn<F, T>(&self, f: F) -> anyhow::Result<T>
    where
        F: FnOnce(&Connection) -> anyhow::Result<T>,
    {
        let conn = self.conn.lock().map_err(|e| anyhow::anyhow!("Database lock poisoned: {}", e))?;
        f(&conn)
    }

    pub fn default_path() -> String {
        crate::utils::paths::db_path().to_string_lossy().to_string()
    }
}
