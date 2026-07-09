//! 用户配置数据库（user_config.db）
//!
//! 基于 SQLite 的 Key-Value 配置存储，支持多种数据类型：
//!
//! - `string` - 字符串
//! - `bool` - 布尔值
//! - `int` - 整数
//! - `float` - 浮点数
//! - `json` - JSON 对象
//!
//! # 示例
//!
//! ```rust,no_run
//! use mvsep_api_tester::db::user_config;
//!
//! let db = user_config::UserConfigDB::default().unwrap();
//!
//! // 设置和获取字符串
//! db.set_string("api_token", "my-token").unwrap();
//! let token = db.get_string("api_token").unwrap();
//!
//! // 设置和获取布尔值
//! db.set_bool("auto_update", true).unwrap();
//! let auto_update = db.get_bool("auto_update").unwrap();
//!
//! // 设置和获取 JSON
//! use serde_json::json;
//! db.set_json("settings", &json!({"theme": "dark"})).unwrap();
//! let settings: Option<serde_json::Value> = db.get_json("settings").unwrap();
//! ```

use anyhow::Result;
use rusqlite::{params, Connection};
use serde::{Deserialize, Serialize};

/// 用户配置条目
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserConfigEntry {
    /// 配置键
    pub key: String,
    /// 配置值（字符串形式存储）
    pub value: String,
    /// 值类型（string, bool, int, float, json）
    pub value_type: String,
    /// 更新时间
    pub updated_at: String,
}

/// 用户配置数据库
///
/// 提供类型安全的 Key-Value 存储，支持字符串、布尔值、整数、浮点数和 JSON。
pub struct UserConfigDB {
    conn: Connection,
}

impl UserConfigDB {
    /// 创建新的配置数据库连接
    ///
    /// # 参数
    ///
    /// - `path`: 数据库文件路径
    ///
    /// # 返回
    ///
    /// `Result<Self>` - 数据库实例或错误
    pub fn new(path: &str) -> Result<Self> {
        let conn = Connection::open(path)?;
        let db = Self { conn };
        db.init()?;
        Ok(db)
    }

    /// 创建默认配置数据库连接
    ///
    /// 默认路径为 `$HOME/user_config.db`。
    ///
    /// # 返回
    ///
    /// `Result<Self>` - 数据库实例或错误
    #[allow(clippy::should_implement_trait)]
    pub fn default() -> Result<Self> {
        let path = format!(
            "{}/user_config.db",
            std::env::var("HOME").unwrap_or_else(|_| ".".to_string())
        );
        Self::new(&path)
    }

    fn init(&self) -> Result<()> {
        self.conn.execute_batch(
            "
            CREATE TABLE IF NOT EXISTS user_config (
                key TEXT PRIMARY KEY NOT NULL,
                value TEXT NOT NULL DEFAULT '',
                value_type TEXT NOT NULL DEFAULT 'string',
                updated_at TEXT DEFAULT (datetime('now')),
                created_at TEXT DEFAULT (datetime('now'))
            );

            CREATE INDEX IF NOT EXISTS idx_user_config_key ON user_config(key);
            CREATE INDEX IF NOT EXISTS idx_user_config_type ON user_config(value_type);

            CREATE TABLE IF NOT EXISTS config_groups (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                name TEXT NOT NULL UNIQUE,
                description TEXT,
                created_at TEXT DEFAULT (datetime('now'))
            );
            ",
        )?;
        Ok(())
    }

    /// 设置字符串值
    ///
    /// # 参数
    ///
    /// - `key`: 配置键
    /// - `value`: 字符串值
    ///
    /// # 返回
    ///
    /// `Result<()>` - 成功或错误
    pub fn set_string(&self, key: &str, value: &str) -> Result<()> {
        self.conn.execute(
            "INSERT OR REPLACE INTO user_config (key, value, value_type, updated_at) VALUES (?1, ?2, 'string', datetime('now'))",
            params![key, value],
        )?;
        Ok(())
    }

    /// 获取字符串值
    ///
    /// # 参数
    ///
    /// - `key`: 配置键
    ///
    /// # 返回
    ///
    /// `Result<Option<String>>` - 字符串值或 `None`
    pub fn get_string(&self, key: &str) -> Result<Option<String>> {
        let mut stmt = self
            .conn
            .prepare("SELECT value FROM user_config WHERE key = ?1")?;
        let mut rows = stmt.query_map(params![key], |row| row.get(0))?;
        match rows.next() {
            Some(result) => Ok(Some(result?)),
            None => Ok(None),
        }
    }

    /// 设置布尔值
    ///
    /// # 参数
    ///
    /// - `key`: 配置键
    /// - `value`: 布尔值
    ///
    /// # 返回
    ///
    /// `Result<()>` - 成功或错误
    pub fn set_bool(&self, key: &str, value: bool) -> Result<()> {
        self.conn.execute(
            "INSERT OR REPLACE INTO user_config (key, value, value_type, updated_at) VALUES (?1, ?2, 'bool', datetime('now'))",
            params![key, if value { "true" } else { "false" }],
        )?;
        Ok(())
    }

    /// 获取布尔值
    ///
    /// # 参数
    ///
    /// - `key`: 配置键
    ///
    /// # 返回
    ///
    /// `Result<Option<bool>>` - 布尔值或 `None`
    pub fn get_bool(&self, key: &str) -> Result<Option<bool>> {
        match self.get_string(key)? {
            Some(value) => Ok(Some(value == "true")),
            None => Ok(None),
        }
    }

    /// 设置整数值
    ///
    /// # 参数
    ///
    /// - `key`: 配置键
    /// - `value`: 整数值
    ///
    /// # 返回
    ///
    /// `Result<()>` - 成功或错误
    pub fn set_int(&self, key: &str, value: i64) -> Result<()> {
        self.conn.execute(
            "INSERT OR REPLACE INTO user_config (key, value, value_type, updated_at) VALUES (?1, ?2, 'int', datetime('now'))",
            params![key, value.to_string()],
        )?;
        Ok(())
    }

    /// 获取整数值
    ///
    /// # 参数
    ///
    /// - `key`: 配置键
    ///
    /// # 返回
    ///
    /// `Result<Option<i64>>` - 整数值或 `None`
    pub fn get_int(&self, key: &str) -> Result<Option<i64>> {
        match self.get_string(key)? {
            Some(value) => match value.parse::<i64>() {
                Ok(v) => Ok(Some(v)),
                Err(_) => Ok(None),
            },
            None => Ok(None),
        }
    }

    /// 设置浮点数值
    ///
    /// # 参数
    ///
    /// - `key`: 配置键
    /// - `value`: 浮点数值
    ///
    /// # 返回
    ///
    /// `Result<()>` - 成功或错误
    pub fn set_float(&self, key: &str, value: f64) -> Result<()> {
        self.conn.execute(
            "INSERT OR REPLACE INTO user_config (key, value, value_type, updated_at) VALUES (?1, ?2, 'float', datetime('now'))",
            params![key, value.to_string()],
        )?;
        Ok(())
    }

    /// 获取浮点数值
    ///
    /// # 参数
    ///
    /// - `key`: 配置键
    ///
    /// # 返回
    ///
    /// `Result<Option<f64>>` - 浮点数值或 `None`
    pub fn get_float(&self, key: &str) -> Result<Option<f64>> {
        match self.get_string(key)? {
            Some(value) => match value.parse::<f64>() {
                Ok(v) => Ok(Some(v)),
                Err(_) => Ok(None),
            },
            None => Ok(None),
        }
    }

    /// 设置 JSON 值
    ///
    /// # 参数
    ///
    /// - `key`: 配置键
    /// - `value`: 实现 `Serialize` trait 的值
    ///
    /// # 返回
    ///
    /// `Result<()>` - 成功或错误
    pub fn set_json<T: Serialize>(&self, key: &str, value: &T) -> Result<()> {
        let json_str = serde_json::to_string(value)?;
        self.conn.execute(
            "INSERT OR REPLACE INTO user_config (key, value, value_type, updated_at) VALUES (?1, ?2, 'json', datetime('now'))",
            params![key, json_str],
        )?;
        Ok(())
    }

    /// 获取 JSON 值
    ///
    /// # 参数
    ///
    /// - `key`: 配置键
    ///
    /// # 返回
    ///
    /// `Result<Option<T>>` - JSON 反序列化后的值或 `None`
    pub fn get_json<T: for<'de> Deserialize<'de>>(&self, key: &str) -> Result<Option<T>> {
        match self.get_string(key)? {
            Some(value) => match serde_json::from_str(&value) {
                Ok(v) => Ok(Some(v)),
                Err(_) => Ok(None),
            },
            None => Ok(None),
        }
    }

    /// 删除配置项
    ///
    /// # 参数
    ///
    /// - `key`: 配置键
    ///
    /// # 返回
    ///
    /// `Result<bool>` - 是否删除成功
    pub fn delete(&self, key: &str) -> Result<bool> {
        let rows_affected = self
            .conn
            .execute("DELETE FROM user_config WHERE key = ?1", params![key])?;
        Ok(rows_affected > 0)
    }

    /// 检查配置项是否存在
    ///
    /// # 参数
    ///
    /// - `key`: 配置键
    ///
    /// # 返回
    ///
    /// `Result<bool>` - 是否存在
    pub fn exists(&self, key: &str) -> Result<bool> {
        let mut stmt = self
            .conn
            .prepare("SELECT COUNT(*) FROM user_config WHERE key = ?1")?;
        let count: i64 = stmt.query_row(params![key], |row| row.get(0))?;
        Ok(count > 0)
    }

    /// 按前缀获取配置项
    ///
    /// # 参数
    ///
    /// - `prefix`: 键前缀
    ///
    /// # 返回
    ///
    /// `Result<Vec<UserConfigEntry>>` - 匹配的配置项列表
    pub fn get_all_by_prefix(&self, prefix: &str) -> Result<Vec<UserConfigEntry>> {
        let mut stmt = self.conn.prepare(
            "SELECT key, value, value_type, updated_at FROM user_config WHERE key LIKE ?1 || '%' ORDER BY key"
        )?;
        let rows = stmt.query_map(params![prefix], |row| {
            Ok(UserConfigEntry {
                key: row.get(0)?,
                value: row.get(1)?,
                value_type: row.get(2)?,
                updated_at: row.get(3)?,
            })
        })?;
        rows.collect::<Result<Vec<_>, _>>().map_err(|e| e.into())
    }

    /// 按类型获取配置项
    ///
    /// # 参数
    ///
    /// - `value_type`: 值类型（string, bool, int, float, json）
    ///
    /// # 返回
    ///
    /// `Result<Vec<UserConfigEntry>>` - 匹配的配置项列表
    pub fn get_all_by_type(&self, value_type: &str) -> Result<Vec<UserConfigEntry>> {
        let mut stmt = self.conn.prepare(
            "SELECT key, value, value_type, updated_at FROM user_config WHERE value_type = ?1 ORDER BY key"
        )?;
        let rows = stmt.query_map(params![value_type], |row| {
            Ok(UserConfigEntry {
                key: row.get(0)?,
                value: row.get(1)?,
                value_type: row.get(2)?,
                updated_at: row.get(3)?,
            })
        })?;
        rows.collect::<Result<Vec<_>, _>>().map_err(|e| e.into())
    }

    /// 获取所有配置项
    ///
    /// # 返回
    ///
    /// `Result<Vec<UserConfigEntry>>` - 所有配置项列表
    pub fn get_all(&self) -> Result<Vec<UserConfigEntry>> {
        let mut stmt = self
            .conn
            .prepare("SELECT key, value, value_type, updated_at FROM user_config ORDER BY key")?;
        let rows = stmt.query_map([], |row| {
            Ok(UserConfigEntry {
                key: row.get(0)?,
                value: row.get(1)?,
                value_type: row.get(2)?,
                updated_at: row.get(3)?,
            })
        })?;
        rows.collect::<Result<Vec<_>, _>>().map_err(|e| e.into())
    }

    /// 获取配置项数量
    ///
    /// # 返回
    ///
    /// `Result<i64>` - 配置项数量
    pub fn count(&self) -> Result<i64> {
        let mut stmt = self.conn.prepare("SELECT COUNT(*) FROM user_config")?;
        let count: i64 = stmt.query_row([], |row| row.get(0))?;
        Ok(count)
    }

    /// 清空所有配置项
    ///
    /// # 返回
    ///
    /// `Result<u64>` - 删除的配置项数量
    pub fn clear_all(&self) -> Result<u64> {
        let affected = self.conn.execute("DELETE FROM user_config", [])?;
        Ok(affected as u64)
    }

    /// 导出所有配置为 JSON
    ///
    /// # 返回
    ///
    /// `Result<String>` - JSON 字符串
    pub fn export_to_json(&self) -> Result<String> {
        let entries = self.get_all()?;
        let json = serde_json::to_string_pretty(&entries)?;
        Ok(json)
    }

    /// 从 JSON 导入配置
    ///
    /// # 参数
    ///
    /// - `json_str`: JSON 字符串
    ///
    /// # 返回
    ///
    /// `Result<u64>` - 导入的配置项数量
    pub fn import_from_json(&self, json_str: &str) -> Result<u64> {
        let entries: Vec<UserConfigEntry> = serde_json::from_str(json_str)?;
        let mut count = 0u64;

        for entry in entries {
            self.conn.execute(
                "INSERT OR REPLACE INTO user_config (key, value, value_type, updated_at) VALUES (?1, ?2, ?3, ?4)",
                params![entry.key, entry.value, entry.value_type, entry.updated_at],
            )?;
            count += 1;
        }

        Ok(count)
    }
}
