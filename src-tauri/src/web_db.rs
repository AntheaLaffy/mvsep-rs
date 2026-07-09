use rusqlite::{params, Connection};
use serde::{Deserialize, Serialize};
use std::path::Path;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebConfigEntry {
    pub key: String,
    pub value: String,
    pub updated_at: String,
}

pub struct WebDatabase {
    conn: Connection,
}

impl WebDatabase {
    pub fn new(path: &str) -> anyhow::Result<Self> {
        let conn = Connection::open(path)?;
        let db = Self { conn };
        db.init()?;
        Ok(db)
    }

    fn init(&self) -> anyhow::Result<()> {
        self.conn.execute_batch(
            "
            CREATE TABLE IF NOT EXISTS web_config (
                key TEXT PRIMARY KEY NOT NULL,
                value TEXT NOT NULL DEFAULT '',
                updated_at TEXT DEFAULT (datetime('now'))
            );

            CREATE INDEX IF NOT EXISTS idx_web_config_key ON web_config(key);
            ",
        )?;
        Ok(())
    }

    pub fn set_string(&self, key: &str, value: &str) -> anyhow::Result<()> {
        self.conn.execute(
            "INSERT OR REPLACE INTO web_config (key, value, updated_at) VALUES (?1, ?2, datetime('now'))",
            params![key, value],
        )?;
        Ok(())
    }

    pub fn get_string(&self, key: &str) -> anyhow::Result<Option<String>> {
        let mut stmt = self.conn.prepare("SELECT value FROM web_config WHERE key = ?1")?;
        let mut rows = stmt.query_map(params![key], |row| row.get(0))?;
        match rows.next() {
            Some(result) => Ok(Some(result?)),
            None => Ok(None),
        }
    }

    pub fn delete(&self, key: &str) -> anyhow::Result<bool> {
        let rows_affected = self.conn.execute("DELETE FROM web_config WHERE key = ?1", params![key])?;
        Ok(rows_affected > 0)
    }

    pub fn exists(&self, key: &str) -> anyhow::Result<bool> {
        let mut stmt = self.conn.prepare("SELECT COUNT(*) FROM web_config WHERE key = ?1")?;
        let count: i64 = stmt.query_row(params![key], |row| row.get(0))?;
        Ok(count > 0)
    }

    pub fn get_all(&self) -> anyhow::Result<Vec<WebConfigEntry>> {
        let mut stmt = self.conn.prepare("SELECT key, value, updated_at FROM web_config ORDER BY key")?;
        let rows = stmt.query_map([], |row| {
            Ok(WebConfigEntry {
                key: row.get(0)?,
                value: row.get(1)?,
                updated_at: row.get(2)?,
            })
        })?;
        rows.collect::<Result<Vec<_>, _>>().map_err(|e| e.into())
    }

    pub fn clear_all(&self) -> anyhow::Result<u64> {
        let affected = self.conn.execute("DELETE FROM web_config", [])?;
        Ok(affected as u64)
    }
}

pub fn open_web_database(db_path: &Path) -> anyhow::Result<WebDatabase> {
    if let Some(parent) = db_path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    WebDatabase::new(&db_path.to_string_lossy())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::sync::atomic::{AtomicUsize, Ordering};

    static TEST_COUNTER: AtomicUsize = AtomicUsize::new(0);

    fn create_temp_db_path() -> String {
        let temp_dir = std::env::temp_dir();
        let counter = TEST_COUNTER.fetch_add(1, Ordering::Relaxed);
        let unique_name = format!("mvsep_web_db_test_{}_{}.db", std::process::id(), counter);
        let path = temp_dir.join(unique_name);
        if path.exists() {
            fs::remove_file(&path).unwrap();
        }
        path.to_string_lossy().to_string()
    }

    #[test]
    fn test_set_and_get_string() {
        let db_path = create_temp_db_path();
        let db = WebDatabase::new(&db_path).unwrap();
        
        db.set_string("locale", "zh-CN").unwrap();
        let result = db.get_string("locale").unwrap();
        assert_eq!(result, Some("zh-CN".to_string()));
        
        fs::remove_file(&db_path).unwrap();
    }

    #[test]
    fn test_get_nonexistent_key() {
        let db_path = create_temp_db_path();
        let db = WebDatabase::new(&db_path).unwrap();
        
        let result = db.get_string("nonexistent").unwrap();
        assert_eq!(result, None);
        
        fs::remove_file(&db_path).unwrap();
    }

    #[test]
    fn test_update_existing_key() {
        let db_path = create_temp_db_path();
        let db = WebDatabase::new(&db_path).unwrap();
        
        db.set_string("theme", "dark").unwrap();
        db.set_string("theme", "fresh-cyan").unwrap();
        let result = db.get_string("theme").unwrap();
        assert_eq!(result, Some("fresh-cyan".to_string()));
        
        fs::remove_file(&db_path).unwrap();
    }

    #[test]
    fn test_delete_key() {
        let db_path = create_temp_db_path();
        let db = WebDatabase::new(&db_path).unwrap();
        
        db.set_string("test_key", "test_value").unwrap();
        assert!(db.exists("test_key").unwrap());
        
        let deleted = db.delete("test_key").unwrap();
        assert!(deleted);
        assert!(!db.exists("test_key").unwrap());
        
        fs::remove_file(&db_path).unwrap();
    }

    #[test]
    fn test_get_all_entries() {
        let db_path = create_temp_db_path();
        let db = WebDatabase::new(&db_path).unwrap();
        
        db.set_string("key1", "value1").unwrap();
        db.set_string("key2", "value2").unwrap();
        
        let entries = db.get_all().unwrap();
        assert_eq!(entries.len(), 2);
        
        let keys: Vec<String> = entries.into_iter().map(|e| e.key).collect();
        assert!(keys.contains(&"key1".to_string()));
        assert!(keys.contains(&"key2".to_string()));
        
        fs::remove_file(&db_path).unwrap();
    }

    #[test]
    fn test_clear_all() {
        let db_path = create_temp_db_path();
        let db = WebDatabase::new(&db_path).unwrap();
        
        db.set_string("key1", "value1").unwrap();
        db.set_string("key2", "value2").unwrap();
        
        let cleared = db.clear_all().unwrap();
        assert_eq!(cleared, 2);
        
        let entries = db.get_all().unwrap();
        assert_eq!(entries.len(), 0);
        
        fs::remove_file(&db_path).unwrap();
    }

    #[test]
    fn test_persistence_across_restarts() {
        let db_path = create_temp_db_path();
        
        {
            let db = WebDatabase::new(&db_path).unwrap();
            db.set_string("theme", "dark").unwrap();
            db.set_string("locale", "en-US").unwrap();
        }
        
        {
            let db = WebDatabase::new(&db_path).unwrap();
            assert_eq!(db.get_string("theme").unwrap(), Some("dark".to_string()));
            assert_eq!(db.get_string("locale").unwrap(), Some("en-US".to_string()));
        }
        
        fs::remove_file(&db_path).unwrap();
    }
}