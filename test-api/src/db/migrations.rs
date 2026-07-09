use anyhow::Result;
use colored::Colorize;
use rusqlite::Connection;

const SCHEMA_VERSION: i32 = 3;

pub fn run_migrations(conn: &Connection) -> Result<()> {
    let user_version: i32 = conn.pragma_query_value(None, "user_version", |row| row.get(0))?;

    if user_version < 1 {
        create_tables(conn)?;
        create_indexes(conn)?;
        set_version(conn, 1)?;
        println!("{}", "✅ Database schema v1 created".green());
    }

    if user_version < 2 {
        migrate_v2(conn)?;
        set_version(conn, 2)?;
        println!("{}", "✅ Database schema migrated to v2".green());
    }

    if user_version < 3 {
        migrate_v3(conn)?;
        set_version(conn, SCHEMA_VERSION)?;
        println!("{}", "✅ Database schema migrated to v3".green());
    }

    Ok(())
}

fn create_tables(conn: &Connection) -> Result<()> {
    conn.execute_batch(
        "
        CREATE TABLE IF NOT EXISTS algorithm_groups (
            id INTEGER PRIMARY KEY,
            name TEXT NOT NULL
        );

        CREATE TABLE IF NOT EXISTS algorithms (
            id INTEGER PRIMARY KEY,
            name TEXT NOT NULL,
            group_id INTEGER DEFAULT 0,
            price_coefficient REAL DEFAULT 1.0,
            orientation INTEGER DEFAULT 0,
            is_cached INTEGER NOT NULL DEFAULT 1,
            FOREIGN KEY (group_id) REFERENCES algorithm_groups(id)
        );

        CREATE TABLE IF NOT EXISTS algorithm_fields (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            algorithm_id INTEGER NOT NULL,
            name TEXT NOT NULL,
            text TEXT,
            options TEXT,
            default_key TEXT,
            FOREIGN KEY (algorithm_id) REFERENCES algorithms(id) ON DELETE CASCADE
        );

        CREATE TABLE IF NOT EXISTS output_formats (
            id INTEGER PRIMARY KEY,
            name TEXT NOT NULL
        );

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
            status TEXT NOT NULL DEFAULT 'waiting',
            progress REAL DEFAULT 0,
            created_at INTEGER NOT NULL,
            output_files TEXT DEFAULT '[]',
            error TEXT,
            message TEXT,
            queue_count INTEGER,
            current_order INTEGER,
            phase TEXT DEFAULT 'queueing',
            download_file_name TEXT,
            download_bytes INTEGER DEFAULT 0,
            download_total_bytes INTEGER,
            download_speed_bps REAL DEFAULT 0,
            download_percent REAL DEFAULT 0,
            FOREIGN KEY (algorithm_id) REFERENCES algorithms(id)
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

        CREATE TABLE IF NOT EXISTS presets (
            id TEXT PRIMARY KEY,
            name TEXT NOT NULL,
            algorithm_id INTEGER NOT NULL,
            opt1 INTEGER,
            opt2 INTEGER,
            opt3 INTEGER,
            format_id INTEGER NOT NULL,
            demo INTEGER DEFAULT 0,
            FOREIGN KEY (algorithm_id) REFERENCES algorithms(id)
        );

        CREATE TABLE IF NOT EXISTS config (
            id INTEGER PRIMARY KEY CHECK (id = 1),
            token TEXT,
            api_url TEXT DEFAULT 'https://mvsep.com',
            mirror TEXT DEFAULT 'main',
            proxy_mode TEXT DEFAULT 'system',
            proxy_host TEXT DEFAULT 'localhost',
            proxy_port TEXT DEFAULT '7897',
            output_dir TEXT DEFAULT 'output',
            output_format INTEGER DEFAULT 1,
            poll_interval INTEGER DEFAULT 5,
            algorithm_auto_refresh_days INTEGER DEFAULT 15,
            updated_at TEXT DEFAULT (datetime('now'))
        );

        CREATE TABLE IF NOT EXISTS log_entries (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            timestamp TEXT NOT NULL,
            level TEXT NOT NULL,
            message TEXT NOT NULL,
            source TEXT DEFAULT 'backend' CHECK(source IN ('frontend', 'backend'))
        );
        ",
    )?;
    Ok(())
}

fn migrate_v3(conn: &Connection) -> Result<()> {
    let _ = conn.execute(
        "ALTER TABLE algorithms ADD COLUMN is_cached INTEGER NOT NULL DEFAULT 1",
        [],
    );
    Ok(())
}

fn create_indexes(conn: &Connection) -> Result<()> {
    conn.execute_batch(
        "
        CREATE INDEX IF NOT EXISTS idx_tasks_status ON tasks(status);
        CREATE INDEX IF NOT EXISTS idx_tasks_created_at ON tasks(created_at);
        CREATE INDEX IF NOT EXISTS idx_task_history_completed_at ON task_history(completed_at);
        CREATE INDEX IF NOT EXISTS idx_task_history_algorithm_id ON task_history(algorithm_id);
        CREATE INDEX IF NOT EXISTS idx_algorithm_fields_algorithm_id ON algorithm_fields(algorithm_id);
        CREATE INDEX IF NOT EXISTS idx_algorithms_group_id ON algorithms(group_id);
        CREATE INDEX IF NOT EXISTS idx_log_entries_timestamp ON log_entries(timestamp);
        CREATE INDEX IF NOT EXISTS idx_log_entries_level ON log_entries(level);
        ",
    )?;
    Ok(())
}

fn set_version(conn: &Connection, version: i32) -> Result<()> {
    conn.pragma_update(None, "user_version", version)?;
    Ok(())
}

fn migrate_v2(conn: &Connection) -> Result<()> {
    // Add new columns to output_formats
    let _ = conn.execute(
        "ALTER TABLE output_formats ADD COLUMN bits_per_sample INTEGER",
        [],
    );
    let _ = conn.execute(
        "ALTER TABLE output_formats ADD COLUMN extension TEXT NOT NULL DEFAULT ''",
        [],
    );
    let _ = conn.execute(
        "ALTER TABLE output_formats ADD COLUMN is_premium INTEGER NOT NULL DEFAULT 0",
        [],
    );

    // Create algorithm-output_format junction table
    conn.execute_batch(
        "
        CREATE TABLE IF NOT EXISTS algorithm_output_formats (
            algorithm_id INTEGER NOT NULL,
            format_id INTEGER NOT NULL,
            PRIMARY KEY (algorithm_id, format_id),
            FOREIGN KEY (algorithm_id) REFERENCES algorithms(id) ON DELETE CASCADE,
            FOREIGN KEY (format_id) REFERENCES output_formats(id) ON DELETE CASCADE
        );

        CREATE INDEX IF NOT EXISTS idx_algo_fmt_algorithm_id ON algorithm_output_formats(algorithm_id);
        CREATE INDEX IF NOT EXISTS idx_algo_fmt_format_id ON algorithm_output_formats(format_id);
        ",
    )?;

    // Update default format data with bit depth and premium info
    conn.execute(
        "INSERT OR REPLACE INTO output_formats (id, name, bits_per_sample, extension, is_premium) VALUES (?1, ?2, ?3, ?4, ?5)",
        rusqlite::params![0, "MP3 (320 kbps)", rusqlite::types::Null, "mp3", 0],
    )?;
    conn.execute(
        "INSERT OR REPLACE INTO output_formats (id, name, bits_per_sample, extension, is_premium) VALUES (?1, ?2, ?3, ?4, ?5)",
        rusqlite::params![1, "WAV (16 bit)", 16, "wav", 0],
    )?;
    conn.execute(
        "INSERT OR REPLACE INTO output_formats (id, name, bits_per_sample, extension, is_premium) VALUES (?1, ?2, ?3, ?4, ?5)",
        rusqlite::params![2, "FLAC (16 bit)", 16, "flac", 0],
    )?;
    conn.execute(
        "INSERT OR REPLACE INTO output_formats (id, name, bits_per_sample, extension, is_premium) VALUES (?1, ?2, ?3, ?4, ?5)",
        rusqlite::params![3, "M4A (lossy)", rusqlite::types::Null, "m4a", 0],
    )?;
    conn.execute(
        "INSERT OR REPLACE INTO output_formats (id, name, bits_per_sample, extension, is_premium) VALUES (?1, ?2, ?3, ?4, ?5)",
        rusqlite::params![4, "WAV (32 bit)", 32, "wav", 1],
    )?;
    conn.execute(
        "INSERT OR REPLACE INTO output_formats (id, name, bits_per_sample, extension, is_premium) VALUES (?1, ?2, ?3, ?4, ?5)",
        rusqlite::params![5, "FLAC (24 bit)", 24, "flac", 1],
    )?;

    // Populate algorithm_output_formats: associate all formats with all algorithms
    conn.execute_batch(
        "
        INSERT OR IGNORE INTO algorithm_output_formats (algorithm_id, format_id)
        SELECT a.id, f.id FROM algorithms a CROSS JOIN output_formats f;
        ",
    )?;

    Ok(())
}
