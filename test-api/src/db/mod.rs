//! 数据库模块
//!
//! 提供三个独立的 SQLite 数据库管理：
//!
//! - [`Database`] - 主数据库，管理算法缓存
//! - [`tasks_db::TasksDatabase`] - 任务数据库，追踪任务生命周期
//! - [`user_config::UserConfigDB`] - 用户配置数据库，存储 Key-Value 配置
//!
//! # 连接管理
//!
//! 所有数据库使用 WAL 模式和 5 秒 busy timeout，确保多线程环境下的并发安全。

pub mod migrations;
pub mod repositories;
pub mod tasks_db;
pub mod user_config;

use rusqlite::Connection;
use std::sync::Mutex;

/// 主数据库连接（算法缓存）
///
/// 管理算法、算法字段、输出格式、预设、任务历史等数据。
/// 默认路径由 [`crate::utils::paths::db_path`] 确定。
///
/// # 示例
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
    /// SQLite 连接（带 Mutex 保护）
    pub conn: Mutex<Connection>,
}

impl Database {
    /// 创建新的数据库连接
    ///
    /// # 参数
    ///
    /// - `db_path`: 数据库文件路径，`None` 则使用默认路径
    ///
    /// # 返回
    ///
    /// `Result<Self>` - 数据库实例或错误
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
            .map_err(|e| anyhow::anyhow!("Database lock poisoned: {}", e))?;
        f(&conn)
    }

    /// 获取默认数据库路径
    ///
    /// # 返回
    ///
    /// `String` - 默认数据库文件路径
    pub fn default_path() -> String {
        crate::utils::paths::db_path().to_string_lossy().to_string()
    }
}
