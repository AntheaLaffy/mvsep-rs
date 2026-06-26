use rusqlite::{Connection, params};
use anyhow::Result;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserConfigEntry {
    pub key: String,
    pub value: String,
    pub value_type: String,
    pub updated_at: String,
}

pub struct UserConfigDB {
    conn: Connection,
}

impl UserConfigDB {
    pub fn new(path: &str) -> Result<Self> {
        let conn = Connection::open(path)?;
        let db = Self { conn };
        db.init()?;
        Ok(db)
    }

    pub fn default() -> Result<Self> {
        let path = format!("{}/user_config.db", 
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

    pub fn set_string(&self, key: &str, value: &str) -> Result<()> {
        self.conn.execute(
            "INSERT OR REPLACE INTO user_config (key, value, value_type, updated_at) VALUES (?1, ?2, 'string', datetime('now'))",
            params![key, value],
        )?;
        Ok(())
    }

    pub fn get_string(&self, key: &str) -> Result<Option<String>> {
        let mut stmt = self.conn.prepare(
            "SELECT value FROM user_config WHERE key = ?1"
        )?;
        let mut rows = stmt.query_map(params![key], |row| row.get(0))?;
        match rows.next() {
            Some(result) => Ok(Some(result?)),
            None => Ok(None),
        }
    }

    pub fn set_bool(&self, key: &str, value: bool) -> Result<()> {
        self.conn.execute(
            "INSERT OR REPLACE INTO user_config (key, value, value_type, updated_at) VALUES (?1, ?2, 'bool', datetime('now'))",
            params![key, if value { "true" } else { "false" }],
        )?;
        Ok(())
    }

    pub fn get_bool(&self, key: &str) -> Result<Option<bool>> {
        match self.get_string(key)? {
            Some(value) => Ok(Some(value == "true")),
            None => Ok(None),
        }
    }

    pub fn set_int(&self, key: &str, value: i64) -> Result<()> {
        self.conn.execute(
            "INSERT OR REPLACE INTO user_config (key, value, value_type, updated_at) VALUES (?1, ?2, 'int', datetime('now'))",
            params![key, value.to_string()],
        )?;
        Ok(())
    }

    pub fn get_int(&self, key: &str) -> Result<Option<i64>> {
        match self.get_string(key)? {
            Some(value) => match value.parse::<i64>() {
                Ok(v) => Ok(Some(v)),
                Err(_) => Ok(None),
            },
            None => Ok(None),
        }
    }

    pub fn set_float(&self, key: &str, value: f64) -> Result<()> {
        self.conn.execute(
            "INSERT OR REPLACE INTO user_config (key, value, value_type, updated_at) VALUES (?1, ?2, 'float', datetime('now'))",
            params![key, value.to_string()],
        )?;
        Ok(())
    }

    pub fn get_float(&self, key: &str) -> Result<Option<f64>> {
        match self.get_string(key)? {
            Some(value) => match value.parse::<f64>() {
                Ok(v) => Ok(Some(v)),
                Err(_) => Ok(None),
            },
            None => Ok(None),
        }
    }

    pub fn set_json<T: Serialize>(&self, key: &str, value: &T) -> Result<()> {
        let json_str = serde_json::to_string(value)?;
        self.conn.execute(
            "INSERT OR REPLACE INTO user_config (key, value, value_type, updated_at) VALUES (?1, ?2, 'json', datetime('now'))",
            params![key, json_str],
        )?;
        Ok(())
    }

    pub fn get_json<T: for<'de> Deserialize<'de>>(&self, key: &str) -> Result<Option<T>> {
        match self.get_string(key)? {
            Some(value) => match serde_json::from_str(&value) {
                Ok(v) => Ok(Some(v)),
                Err(_) => Ok(None),
            },
            None => Ok(None),
        }
    }

    pub fn delete(&self, key: &str) -> Result<bool> {
        let rows_affected = self.conn.execute(
            "DELETE FROM user_config WHERE key = ?1",
            params![key],
        )?;
        Ok(rows_affected > 0)
    }

    pub fn exists(&self, key: &str) -> Result<bool> {
        let mut stmt = self.conn.prepare(
            "SELECT COUNT(*) FROM user_config WHERE key = ?1"
        )?;
        let count: i64 = stmt.query_row(params![key], |row| row.get(0))?;
        Ok(count > 0)
    }

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

    pub fn get_all(&self) -> Result<Vec<UserConfigEntry>> {
        let mut stmt = self.conn.prepare(
            "SELECT key, value, value_type, updated_at FROM user_config ORDER BY key"
        )?;
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

    pub fn count(&self) -> Result<i64> {
        let mut stmt = self.conn.prepare("SELECT COUNT(*) FROM user_config")?;
        let count: i64 = stmt.query_row([], |row| row.get(0))?;
        Ok(count)
    }

    pub fn clear_all(&self) -> Result<u64> {
        let affected = self.conn.execute("DELETE FROM user_config", [])?;
        Ok(affected as u64)
    }

    pub fn export_to_json(&self) -> Result<String> {
        let entries = self.get_all()?;
        let json = serde_json::to_string_pretty(&entries)?;
        Ok(json)
    }

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
