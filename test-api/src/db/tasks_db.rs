use rusqlite::Connection;
use std::sync::Mutex;

pub struct TasksDatabase {
    pub conn: Mutex<Connection>,
}

impl TasksDatabase {
    pub fn new(db_path: Option<&str>) -> anyhow::Result<Self> {
        let path = db_path
            .map(|p| p.to_string())
            .unwrap_or_else(|| crate::utils::paths::tasks_db_path().to_string_lossy().to_string());

        let conn = Connection::open(&path)?;
        conn.execute_batch("PRAGMA journal_mode=WAL")?;
        conn.execute_batch("PRAGMA foreign_keys=ON")?;
        conn.execute_batch("PRAGMA busy_timeout=5000")?;

        let db = Self {
            conn: Mutex::new(conn),
        };

        {
            let locked = db.conn.lock().map_err(|e| anyhow::anyhow!("Poison error: {}", e))?;
            run_migrations(&locked)?;
        }

        Ok(db)
    }

    pub fn with_conn<F, T>(&self, f: F) -> anyhow::Result<T>
    where
        F: FnOnce(&Connection) -> anyhow::Result<T>,
    {
        let conn = self
            .conn
            .lock()
            .map_err(|e| anyhow::anyhow!("TasksDatabase lock poisoned: {}", e))?;
        f(&conn)
    }
}

fn run_migrations(conn: &Connection) -> anyhow::Result<()> {
    conn.execute_batch(
        "
        CREATE TABLE IF NOT EXISTS tasks (
            hash TEXT PRIMARY KEY,
            file_name TEXT NOT NULL,
            algorithm_id INTEGER NOT NULL,
            algorithm_name TEXT NOT NULL,
            model_id INTEGER,
            model_name TEXT,
            model2_id INTEGER,
            model2_name TEXT,
            model3_id INTEGER,
            model3_name TEXT,
            format INTEGER NOT NULL DEFAULT 1,
            status TEXT NOT NULL DEFAULT 'uploaded',
            progress REAL DEFAULT 0,
            created_at INTEGER NOT NULL,
            output_files TEXT DEFAULT '[]',
            error TEXT,
            message TEXT,
            queue_count INTEGER,
            current_order INTEGER,
            phase TEXT DEFAULT 'uploaded',
            download_file_name TEXT,
            download_bytes INTEGER DEFAULT 0,
            download_total_bytes INTEGER,
            download_speed_bps REAL DEFAULT 0,
            download_percent REAL DEFAULT 0
        );

        CREATE TABLE IF NOT EXISTS task_history (
            id TEXT PRIMARY KEY,
            file_name TEXT NOT NULL,
            algorithm_id INTEGER NOT NULL,
            algorithm_name TEXT NOT NULL,
            model_id INTEGER,
            model_name TEXT,
            model2_id INTEGER,
            model2_name TEXT,
            model3_id INTEGER,
            model3_name TEXT,
            format_id INTEGER NOT NULL,
            format_name TEXT NOT NULL,
            status TEXT NOT NULL CHECK(status IN ('done', 'failed')),
            created_at INTEGER NOT NULL,
            completed_at INTEGER,
            output_files TEXT DEFAULT '[]',
            output_path TEXT,
            error TEXT
        );

        CREATE INDEX IF NOT EXISTS idx_tasks_status ON tasks(status);
        CREATE INDEX IF NOT EXISTS idx_tasks_created_at ON tasks(created_at);
        CREATE INDEX IF NOT EXISTS idx_task_history_completed_at ON task_history(completed_at);
        CREATE INDEX IF NOT EXISTS idx_task_history_algorithm_id ON task_history(algorithm_id);
        ",
    )?;
    Ok(())
}
