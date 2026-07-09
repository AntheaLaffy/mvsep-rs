//! 任务数据库（tasks.db）
//!
//! 独立的 SQLite 数据库，追踪任务生命周期和下载进度。
//!
//! # 任务状态流转
//!
//! ```text
//! uploaded → queued → processing → done
//!                     → failed
//!                               → expired（文件过期）
//! ```
//!
//! # 输出文件追踪
//!
//! `output_files` 字段存 JSON，记录每个产物的远程 URL、本地路径、下载状态：
//!
//! ```json
//! [
//!   {"remote_name":"vocals.flac","url":"https://...","downloaded":true,"local_path":"/out/..."},
//!   {"remote_name":"other.flac","url":"https://...","downloaded":false,"local_path":null}
//! ]
//! ```

use rusqlite::Connection;
use std::sync::Mutex;

/// 任务数据库连接
///
/// 管理任务和任务历史数据，支持任务状态追踪和下载进度记录。
/// 默认路径由 [`crate::utils::paths::tasks_db_path`] 确定。
///
/// # 示例
///
/// ```rust,no_run
/// use mvsep_api_tester::db::tasks_db;
///
/// let db = tasks_db::TasksDatabase::new(None).unwrap();
/// let tasks = db.with_conn(|conn| {
///     mvsep_api_tester::db::repositories::get_all_tasks(conn)
/// }).unwrap();
/// ```
pub struct TasksDatabase {
    /// SQLite 连接（带 Mutex 保护）
    pub conn: Mutex<Connection>,
}

impl TasksDatabase {
    /// 创建新的任务数据库连接
    ///
    /// # 参数
    ///
    /// - `db_path`: 数据库文件路径，`None` 则使用默认路径
    ///
    /// # 返回
    ///
    /// `Result<Self>` - 数据库实例或错误
    pub fn new(db_path: Option<&str>) -> anyhow::Result<Self> {
        let path = db_path.map(|p| p.to_string()).unwrap_or_else(|| {
            crate::utils::paths::tasks_db_path()
                .to_string_lossy()
                .to_string()
        });

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
            run_migrations(&locked)?;
        }

        Ok(db)
    }

    /// 在数据库连接上执行闭包
    ///
    /// 获取锁并执行闭包，自动处理锁中毒错误。
    ///
    /// # 参数
    ///
    /// - `f`: 接受 `&Connection` 并返回 `anyhow::Result<T>` 的闭包
    ///
    /// # 返回
    ///
    /// `anyhow::Result<T>` - 闭包返回值或错误
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
