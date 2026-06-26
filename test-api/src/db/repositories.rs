use rusqlite::{Connection, params};
use serde::{Deserialize, Serialize};
use anyhow::Result;
use colored::Colorize;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlgorithmGroupRow {
    pub id: i32,
    pub name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlgorithmRow {
    pub id: i32,
    pub name: String,
    pub group_id: i32,
    #[serde(default)]
    pub price_coefficient: f64,
    #[serde(default)]
    pub orientation: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlgorithmFieldRow {
    pub id: i64,
    pub algorithm_id: i32,
    pub name: String,
    #[serde(default)]
    pub text: Option<String>,
    #[serde(default)]
    pub options: Option<String>,
    #[serde(default)]
    pub default_key: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskRow {
    pub hash: String,
    pub file_name: String,
    pub algorithm_id: i32,
    pub algorithm_name: String,
    pub model_id: Option<i32>,
    pub model_name: Option<String>,
    pub model2_id: Option<i32>,
    pub model2_name: Option<String>,
    pub model3_id: Option<i32>,
    pub model3_name: Option<String>,
    pub format: i32,
    pub status: String,
    pub progress: f64,
    pub created_at: i64,
    pub output_files: String,
    pub error: Option<String>,
    pub message: Option<String>,
    pub queue_count: Option<i32>,
    pub current_order: Option<i32>,
    pub phase: String,
    pub download_file_name: Option<String>,
    pub download_bytes: i64,
    pub download_total_bytes: Option<i64>,
    pub download_speed_bps: f64,
    pub download_percent: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskHistoryRow {
    pub id: String,
    pub file_name: String,
    pub algorithm_id: i32,
    pub algorithm_name: String,
    pub model_id: Option<i32>,
    pub model_name: Option<String>,
    pub model2_id: Option<i32>,
    pub model2_name: Option<String>,
    pub model3_id: Option<i32>,
    pub model3_name: Option<String>,
    pub format_id: i32,
    pub format_name: String,
    pub status: String,
    pub created_at: i64,
    pub completed_at: Option<i64>,
    pub output_files: String,
    pub output_path: Option<String>,
    pub error: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PresetRow {
    pub id: String,
    pub name: String,
    pub algorithm_id: i32,
    pub opt1: Option<i32>,
    pub opt2: Option<i32>,
    pub opt3: Option<i32>,
    pub format_id: i32,
    pub demo: bool,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ConfigRow {
    pub token: Option<String>,
    pub api_url: Option<String>,
    pub mirror: Option<String>,
    pub proxy_mode: Option<String>,
    pub proxy_host: Option<String>,
    pub proxy_port: Option<String>,
    pub output_dir: Option<String>,
    pub output_format: Option<i32>,
    pub poll_interval: Option<i32>,
    pub algorithm_auto_refresh_days: Option<i32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OutputFormatRow {
    pub id: i32,
    pub name: String,
    #[serde(default)]
    pub bits_per_sample: Option<i32>,
    #[serde(default)]
    pub extension: String,
    #[serde(default)]
    pub is_premium: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlgorithmFormatRow {
    pub algorithm_id: i32,
    pub format_id: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogEntryRow {
    pub id: i64,
    pub timestamp: String,
    pub level: String,
    pub message: String,
    pub source: String,
}

pub fn upsert_algorithm_group(conn: &Connection, group: &AlgorithmGroupRow) -> Result<()> {
    conn.execute(
        "INSERT OR REPLACE INTO algorithm_groups (id, name) VALUES (?1, ?2)",
        params![group.id, group.name],
    )?;
    Ok(())
}

pub fn upsert_algorithm(conn: &Connection, algo: &AlgorithmRow) -> Result<()> {
    conn.execute(
        "INSERT OR REPLACE INTO algorithms (id, name, group_id, price_coefficient, orientation) VALUES (?1, ?2, ?3, ?4, ?5)",
        params![algo.id, algo.name, algo.group_id, algo.price_coefficient, algo.orientation],
    )?;
    Ok(())
}

pub fn upsert_algorithm_field(conn: &Connection, field: &AlgorithmFieldRow) -> Result<()> {
    conn.execute(
        "INSERT OR REPLACE INTO algorithm_fields (id, algorithm_id, name, text, options, default_key) VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
        params![field.id, field.algorithm_id, field.name, field.text, field.options, field.default_key],
    )?;
    Ok(())
}

pub fn get_all_algorithms(conn: &Connection) -> Result<Vec<AlgorithmRow>> {
    let mut stmt = conn.prepare("SELECT id, name, group_id, price_coefficient, orientation FROM algorithms ORDER BY group_id, id")?;
    let rows = stmt.query_map([], |row| {
        Ok(AlgorithmRow {
            id: row.get(0)?,
            name: row.get(1)?,
            group_id: row.get(2)?,
            price_coefficient: row.get(3).unwrap_or(1.0),
            orientation: row.get(4).unwrap_or(0),
        })
    })?;
    rows.collect::<Result<Vec<_>, _>>().map_err(|e| e.into())
}

pub fn get_algorithm_by_id(conn: &Connection, id: i32) -> Result<Option<AlgorithmRow>> {
    let mut stmt = conn.prepare("SELECT id, name, group_id, price_coefficient, orientation FROM algorithms WHERE id = ?1")?;
    let mut rows = stmt.query_map(params![id], |row| {
        Ok(AlgorithmRow {
            id: row.get(0)?,
            name: row.get(1)?,
            group_id: row.get(2)?,
            price_coefficient: row.get(3).unwrap_or(1.0),
            orientation: row.get(4).unwrap_or(0),
        })
    })?;
    match rows.next() {
        Some(row) => Ok(Some(row?)),
        None => Ok(None),
    }
}

pub fn get_algorithm_fields(conn: &Connection, algorithm_id: i32) -> Result<Vec<AlgorithmFieldRow>> {
    let mut stmt = conn.prepare(
        "SELECT id, algorithm_id, name, text, options, default_key FROM algorithm_fields WHERE algorithm_id = ?1 ORDER BY id"
    )?;
    let rows = stmt.query_map(params![algorithm_id], |row| {
        Ok(AlgorithmFieldRow {
            id: row.get(0)?,
            algorithm_id: row.get(1)?,
            name: row.get(2)?,
            text: row.get(3)?,
            options: row.get(4)?,
            default_key: row.get(5)?,
        })
    })?;
    rows.collect::<Result<Vec<_>, _>>().map_err(|e| e.into())
}

pub fn get_algorithm_details_with_fields(conn: &Connection, algorithm_id: i32) -> Result<Option<(AlgorithmRow, Vec<AlgorithmFieldRow>)>> {
    let algo = get_algorithm_by_id(conn, algorithm_id)?;
    match algo {
        Some(a) => {
            let fields = get_algorithm_fields(conn, algorithm_id)?;
            Ok(Some((a, fields)))
        }
        None => Ok(None),
    }
}

pub fn get_all_algorithm_groups(conn: &Connection) -> Result<Vec<AlgorithmGroupRow>> {
    let mut stmt = conn.prepare("SELECT id, name FROM algorithm_groups ORDER BY id")?;
    let rows = stmt.query_map([], |row| {
        Ok(AlgorithmGroupRow {
            id: row.get(0)?,
            name: row.get(1)?,
        })
    })?;
    rows.collect::<Result<Vec<_>, _>>().map_err(|e| e.into())
}

pub fn count_algorithms(conn: &Connection) -> Result<i64> {
    let count: i64 = conn.query_row("SELECT COUNT(*) FROM algorithms", [], |row| row.get(0))?;
    Ok(count)
}

pub fn insert_task(conn: &Connection, task: &TaskRow) -> Result<()> {
    conn.execute(
        "INSERT OR REPLACE INTO tasks (hash, file_name, algorithm_id, algorithm_name, model_id, model_name, model2_id, model2_name, model3_id, model3_name, format, status, progress, created_at, output_files, error, message, queue_count, current_order, phase, download_file_name, download_bytes, download_total_bytes, download_speed_bps, download_percent) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16, ?17, ?18, ?19, ?20, ?21, ?22, ?23, ?24, ?25)",
        params![
            task.hash, task.file_name, task.algorithm_id, task.algorithm_name,
            task.model_id, task.model_name, task.model2_id, task.model2_name,
            task.model3_id, task.model3_name, task.format, task.status, task.progress,
            task.created_at, task.output_files, task.error, task.message,
            task.queue_count, task.current_order, task.phase, task.download_file_name,
            task.download_bytes, task.download_total_bytes, task.download_speed_bps, task.download_percent
        ],
    )?;
    Ok(())
}

pub fn get_task_by_hash(conn: &Connection, hash: &str) -> Result<Option<TaskRow>> {
    let mut stmt = conn.prepare(
        "SELECT hash, file_name, algorithm_id, algorithm_name, model_id, model_name, model2_id, model2_name, model3_id, model3_name, format, status, progress, created_at, output_files, error, message, queue_count, current_order, phase, download_file_name, download_bytes, download_total_bytes, download_speed_bps, download_percent FROM tasks WHERE hash = ?1"
    )?;
    let mut rows = stmt.query_map(params![hash], |row| {
        Ok(TaskRow {
            hash: row.get(0)?,
            file_name: row.get(1)?,
            algorithm_id: row.get(2)?,
            algorithm_name: row.get(3)?,
            model_id: row.get(4)?,
            model_name: row.get(5)?,
            model2_id: row.get(6)?,
            model2_name: row.get(7)?,
            model3_id: row.get(8)?,
            model3_name: row.get(9)?,
            format: row.get(10)?,
            status: row.get(11)?,
            progress: row.get(12)?,
            created_at: row.get(13)?,
            output_files: row.get(14)?,
            error: row.get(15)?,
            message: row.get(16)?,
            queue_count: row.get(17)?,
            current_order: row.get(18)?,
            phase: row.get(19)?,
            download_file_name: row.get(20)?,
            download_bytes: row.get(21)?,
            download_total_bytes: row.get(22)?,
            download_speed_bps: row.get(23)?,
            download_percent: row.get(24)?,
        })
    })?;
    match rows.next() {
        Some(row) => Ok(Some(row?)),
        None => Ok(None),
    }
}

pub fn get_all_tasks(conn: &Connection) -> Result<Vec<TaskRow>> {
    let mut stmt = conn.prepare(
        "SELECT hash, file_name, algorithm_id, algorithm_name, model_id, model_name, model2_id, model2_name, model3_id, model3_name, format, status, progress, created_at, output_files, error, message, queue_count, current_order, phase, download_file_name, download_bytes, download_total_bytes, download_speed_bps, download_percent FROM tasks ORDER BY created_at DESC"
    )?;
    let rows = stmt.query_map([], map_task_row)?;
    rows.collect::<Result<Vec<_>, _>>().map_err(|e| e.into())
}

pub fn update_task_status(conn: &Connection, hash: &str, status: &str, progress: f64, error: Option<&str>) -> Result<()> {
    conn.execute(
        "UPDATE tasks SET status = ?1, progress = ?2, error = ?3 WHERE hash = ?4",
        params![status, progress, error, hash],
    )?;
    Ok(())
}

/// Update the output_files JSON for a task (stores remote file info).
pub fn update_task_output_files(conn: &Connection, hash: &str, output_files_json: &str) -> Result<()> {
    conn.execute(
        "UPDATE tasks SET output_files = ?1 WHERE hash = ?2",
        params![output_files_json, hash],
    )?;
    Ok(())
}

/// Read the output_files JSON for a task.
pub fn get_task_output_files(conn: &Connection, hash: &str) -> Result<Option<String>> {
    let mut stmt = conn.prepare("SELECT output_files FROM tasks WHERE hash = ?1")?;
    let mut rows = stmt.query_map(params![hash], |row| row.get::<_, String>(0))?;
    match rows.next() {
        Some(row) => Ok(Some(row?)),
        None => Ok(None),
    }
}

pub fn delete_task(conn: &Connection, hash: &str) -> Result<bool> {
    let affected = conn.execute("DELETE FROM tasks WHERE hash = ?1", params![hash])?;
    Ok(affected > 0)
}

fn map_task_row(row: &rusqlite::Row<'_>) -> rusqlite::Result<TaskRow> {
    Ok(TaskRow {
        hash: row.get(0)?,
        file_name: row.get(1)?,
        algorithm_id: row.get(2)?,
        algorithm_name: row.get(3)?,
        model_id: row.get(4)?,
        model_name: row.get(5)?,
        model2_id: row.get(6)?,
        model2_name: row.get(7)?,
        model3_id: row.get(8)?,
        model3_name: row.get(9)?,
        format: row.get(10)?,
        status: row.get(11)?,
        progress: row.get(12)?,
        created_at: row.get(13)?,
        output_files: row.get(14)?,
        error: row.get(15)?,
        message: row.get(16)?,
        queue_count: row.get(17)?,
        current_order: row.get(18)?,
        phase: row.get(19)?,
        download_file_name: row.get(20)?,
        download_bytes: row.get(21)?,
        download_total_bytes: row.get(22)?,
        download_speed_bps: row.get(23)?,
        download_percent: row.get(24)?,
    })
}

pub fn insert_task_history(conn: &Connection, record: &TaskHistoryRow) -> Result<()> {
    conn.execute(
        "INSERT OR REPLACE INTO task_history (id, file_name, algorithm_id, algorithm_name, model_id, model_name, model2_id, model2_name, model3_id, model3_name, format_id, format_name, status, created_at, completed_at, output_files, output_path, error) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16, ?17, ?18)",
        params![
            record.id, record.file_name, record.algorithm_id, record.algorithm_name,
            record.model_id, record.model_name, record.model2_id, record.model2_name,
            record.model3_id, record.model3_name, record.format_id, record.format_name,
            record.status, record.created_at, record.completed_at, record.output_files,
            record.output_path, record.error
        ],
    )?;
    Ok(())
}

pub fn get_all_task_history(conn: &Connection) -> Result<Vec<TaskHistoryRow>> {
    let mut stmt = conn.prepare(
        "SELECT id, file_name, algorithm_id, algorithm_name, model_id, model_name, model2_id, model2_name, model3_id, model3_name, format_id, format_name, status, created_at, completed_at, output_files, output_path, error FROM task_history ORDER BY completed_at DESC, created_at DESC"
    )?;
    let rows = stmt.query_map([], map_history_row)?;
    rows.collect::<Result<Vec<_>, _>>().map_err(|e| e.into())
}

pub fn get_paginated_task_history(conn: &Connection, page: i64, page_size: i64) -> Result<(Vec<TaskHistoryRow>, i64)> {
    let total: i64 = conn.query_row("SELECT COUNT(*) FROM task_history", [], |row| row.get(0))?;

    let offset = page * page_size;
    let mut stmt = conn.prepare(
        "SELECT id, file_name, algorithm_id, algorithm_name, model_id, model_name, model2_id, model2_name, model3_id, model3_name, format_id, format_name, status, created_at, completed_at, output_files, output_path, error FROM task_history ORDER BY completed_at DESC, created_at DESC LIMIT ?1 OFFSET ?2"
    )?;
    let rows = stmt.query_map(params![page_size, offset], map_history_row)?;
    let records: Vec<TaskHistoryRow> = rows.filter_map(|r| r.ok()).collect();
    Ok((records, total))
}

pub fn delete_task_history(conn: &Connection, id: &str) -> Result<bool> {
    let affected = conn.execute("DELETE FROM task_history WHERE id = ?1", params![id])?;
    Ok(affected > 0)
}

pub fn clear_task_history(conn: &Connection) -> Result<u64> {
    let affected = conn.execute("DELETE FROM task_history", [])?;
    Ok(affected as u64)
}

pub fn trim_task_history(conn: &Connection, max_records: i64) -> Result<u64> {
    let count: i64 = conn.query_row("SELECT COUNT(*) FROM task_history", [], |row| row.get(0))?;
    if count > max_records {
        let to_delete = count - max_records;
        let affected = conn.execute(
            "DELETE FROM task_history WHERE id IN (SELECT id FROM task_history ORDER BY completed_at ASC, created_at ASC LIMIT ?1)",
            params![to_delete],
        )?;
        return Ok(affected as u64);
    }
    Ok(0)
}

fn map_history_row(row: &rusqlite::Row<'_>) -> rusqlite::Result<TaskHistoryRow> {
    Ok(TaskHistoryRow {
        id: row.get(0)?,
        file_name: row.get(1)?,
        algorithm_id: row.get(2)?,
        algorithm_name: row.get(3)?,
        model_id: row.get(4)?,
        model_name: row.get(5)?,
        model2_id: row.get(6)?,
        model2_name: row.get(7)?,
        model3_id: row.get(8)?,
        model3_name: row.get(9)?,
        format_id: row.get(10)?,
        format_name: row.get(11)?,
        status: row.get(12)?,
        created_at: row.get(13)?,
        completed_at: row.get(14)?,
        output_files: row.get(15)?,
        output_path: row.get(16)?,
        error: row.get(17)?,
    })
}

pub fn save_config(conn: &Connection, config: &ConfigRow) -> Result<()> {
    conn.execute(
        "INSERT OR REPLACE INTO config (id, token, api_url, mirror, proxy_mode, proxy_host, proxy_port, output_dir, output_format, poll_interval, algorithm_auto_refresh_days, updated_at) VALUES (1, ?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, datetime('now'))",
        params![
            config.token, config.api_url, config.mirror, config.proxy_mode,
            config.proxy_host, config.proxy_port, config.output_dir, config.output_format,
            config.poll_interval, config.algorithm_auto_refresh_days
        ],
    )?;
    Ok(())
}

pub fn get_config(conn: &Connection) -> Result<Option<ConfigRow>> {
    let mut stmt = conn.prepare(
        "SELECT token, api_url, mirror, proxy_mode, proxy_host, proxy_port, output_dir, output_format, poll_interval, algorithm_auto_refresh_days FROM config WHERE id = 1"
    )?;
    let mut rows = stmt.query_map([], |row| {
        Ok(ConfigRow {
            token: row.get(0)?,
            api_url: row.get(1)?,
            mirror: row.get(2)?,
            proxy_mode: row.get(3)?,
            proxy_host: row.get(4)?,
            proxy_port: row.get(5)?,
            output_dir: row.get(6)?,
            output_format: row.get(7)?,
            poll_interval: row.get(8)?,
            algorithm_auto_refresh_days: row.get(9)?,
        })
    })?;
    match rows.next() {
        Some(row) => Ok(Some(row?)),
        None => Ok(None),
    }
}

pub fn upsert_preset(conn: &Connection, preset: &PresetRow) -> Result<()> {
    conn.execute(
        "INSERT OR REPLACE INTO presets (id, name, algorithm_id, opt1, opt2, opt3, format_id, demo) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
        params![preset.id, preset.name, preset.algorithm_id, preset.opt1, preset.opt2, preset.opt3, preset.format_id, preset.demo],
    )?;
    Ok(())
}

pub fn get_all_presets(conn: &Connection) -> Result<Vec<PresetRow>> {
    let mut stmt = conn.prepare("SELECT id, name, algorithm_id, opt1, opt2, opt3, format_id, demo FROM presets ORDER BY id")?;
    let rows = stmt.query_map([], |row| {
        Ok(PresetRow {
            id: row.get(0)?,
            name: row.get(1)?,
            algorithm_id: row.get(2)?,
            opt1: row.get(3)?,
            opt2: row.get(4)?,
            opt3: row.get(5)?,
            format_id: row.get(6)?,
            demo: row.get::<_, i32>(7)? != 0,
        })
    })?;
    rows.collect::<Result<Vec<_>, _>>().map_err(|e| e.into())
}

pub fn delete_preset(conn: &Connection, id: &str) -> Result<bool> {
    let affected = conn.execute("DELETE FROM presets WHERE id = ?1", params![id])?;
    Ok(affected > 0)
}

pub fn upsert_output_format(conn: &Connection, fmt: &OutputFormatRow) -> Result<()> {
    conn.execute(
        "INSERT OR REPLACE INTO output_formats (id, name, bits_per_sample, extension, is_premium) VALUES (?1, ?2, ?3, ?4, ?5)",
        params![fmt.id, fmt.name, fmt.bits_per_sample, fmt.extension, fmt.is_premium as i32],
    )?;
    Ok(())
}

pub fn get_all_output_formats(conn: &Connection) -> Result<Vec<OutputFormatRow>> {
    let mut stmt = conn.prepare("SELECT id, name, bits_per_sample, extension, is_premium FROM output_formats ORDER BY id")?;
    let rows = stmt.query_map([], |row| {
        Ok(OutputFormatRow {
            id: row.get(0)?,
            name: row.get(1)?,
            bits_per_sample: row.get(2)?,
            extension: row.get(3)?,
            is_premium: row.get::<_, i32>(4)? != 0,
        })
    })?;
    rows.collect::<Result<Vec<_>, _>>().map_err(|e| e.into())
}

pub fn init_default_output_formats(conn: &Connection) -> Result<usize> {
    let defaults = vec![
        OutputFormatRow { id: 0, name: "MP3 (320 kbps)".into(), bits_per_sample: None, extension: "mp3".into(), is_premium: false },
        OutputFormatRow { id: 1, name: "WAV (16 bit)".into(), bits_per_sample: Some(16), extension: "wav".into(), is_premium: false },
        OutputFormatRow { id: 2, name: "FLAC (16 bit)".into(), bits_per_sample: Some(16), extension: "flac".into(), is_premium: false },
        OutputFormatRow { id: 3, name: "M4A (lossy)".into(), bits_per_sample: None, extension: "m4a".into(), is_premium: false },
        OutputFormatRow { id: 4, name: "WAV (32 bit)".into(), bits_per_sample: Some(32), extension: "wav".into(), is_premium: true },
        OutputFormatRow { id: 5, name: "FLAC (24 bit)".into(), bits_per_sample: Some(24), extension: "flac".into(), is_premium: true },
    ];
    let mut count = 0;
    for fmt in &defaults {
        if upsert_output_format(conn, fmt).is_ok() {
            count += 1;
        }
    }
    Ok(count)
}

/// 设置算法支持的输出格式列表（先清除旧关联，再写入新关联）
pub fn set_algorithm_output_formats(conn: &Connection, algorithm_id: i32, format_ids: &[i32]) -> Result<()> {
    remove_algorithm_output_formats(conn, algorithm_id)?;
    for &fid in format_ids {
        conn.execute(
            "INSERT OR IGNORE INTO algorithm_output_formats (algorithm_id, format_id) VALUES (?1, ?2)",
            params![algorithm_id, fid],
        )?;
    }
    Ok(())
}

/// 获取某个算法支持的所有输出格式
pub fn get_formats_for_algorithm(conn: &Connection, algorithm_id: i32) -> Result<Vec<OutputFormatRow>> {
    let mut stmt = conn.prepare(
        "SELECT f.id, f.name, f.bits_per_sample, f.extension, f.is_premium
         FROM output_formats f
         INNER JOIN algorithm_output_formats aof ON aof.format_id = f.id
         WHERE aof.algorithm_id = ?1
         ORDER BY f.id"
    )?;
    let rows = stmt.query_map(params![algorithm_id], |row| {
        Ok(OutputFormatRow {
            id: row.get(0)?,
            name: row.get(1)?,
            bits_per_sample: row.get(2)?,
            extension: row.get(3)?,
            is_premium: row.get::<_, i32>(4)? != 0,
        })
    })?;
    rows.collect::<Result<Vec<_>, _>>().map_err(|e| e.into())
}

/// 获取所有算法-格式关联
pub fn get_all_algorithm_format_associations(conn: &Connection) -> Result<Vec<AlgorithmFormatRow>> {
    let mut stmt = conn.prepare("SELECT algorithm_id, format_id FROM algorithm_output_formats ORDER BY algorithm_id, format_id")?;
    let rows = stmt.query_map([], |row| {
        Ok(AlgorithmFormatRow {
            algorithm_id: row.get(0)?,
            format_id: row.get(1)?,
        })
    })?;
    rows.collect::<Result<Vec<_>, _>>().map_err(|e| e.into())
}

/// 初始化默认算法-格式关联（所有算法关联所有格式）
pub fn init_default_algorithm_format_associations(conn: &Connection) -> Result<usize> {
    let count = conn.execute(
        "INSERT OR IGNORE INTO algorithm_output_formats (algorithm_id, format_id)
         SELECT a.id, f.id FROM algorithms a CROSS JOIN output_formats f",
        [],
    )?;
    Ok(count)
}

/// 清除某个算法的所有格式关联
pub fn remove_algorithm_output_formats(conn: &Connection, algorithm_id: i32) -> Result<()> {
    conn.execute(
        "DELETE FROM algorithm_output_formats WHERE algorithm_id = ?1",
        params![algorithm_id],
    )?;
    Ok(())
}

pub fn insert_log_entry(conn: &Connection, timestamp: &str, level: &str, message: &str, source: &str) -> Result<i64> {
    conn.execute(
        "INSERT INTO log_entries (timestamp, level, message, source) VALUES (?1, ?2, ?3, ?4)",
        params![timestamp, level, message, source],
    )?;
    Ok(conn.last_insert_rowid())
}

pub fn get_recent_logs(conn: &Connection, limit: i64) -> Result<Vec<LogEntryRow>> {
    let mut stmt = conn.prepare(
        "SELECT id, timestamp, level, message, source FROM log_entries ORDER BY id DESC LIMIT ?1"
    )?;
    let rows = stmt.query_map(params![limit], |row| {
        Ok(LogEntryRow {
            id: row.get(0)?,
            timestamp: row.get(1)?,
            level: row.get(2)?,
            message: row.get(3)?,
            source: row.get(4)?,
        })
    })?;
    rows.collect::<Result<Vec<_>, _>>().map_err(|e| e.into())
}

pub fn clear_old_logs(conn: &Connection, days: i64) -> Result<u64> {
    let affected = conn.execute(
        "DELETE FROM log_entries WHERE timestamp < datetime('now', '-' || ?1 || ' days')",
        params![days],
    )?;
    Ok(affected as u64)
}

pub fn print_table_stats(conn: &Connection) -> Result<()> {
    println!("\n{}", "═".repeat(50).cyan());
    println!("  {}", "📊 Database Statistics".cyan().bold());
    println!("{}", "═".repeat(50).cyan());

    let tables = [
        ("algorithm_groups", "算法分组"),
        ("algorithms", "算法"),
        ("algorithm_fields", "算法字段"),
        ("algorithm_output_formats", "算法-格式关联"),
        ("tasks", "任务"),
        ("task_history", "任务历史"),
        ("presets", "预设"),
        ("output_formats", "输出格式"),
        ("config", "配置"),
        ("log_entries", "日志"),
    ];

    for (table, label) in &tables {
        let count: i64 = conn.query_row(&format!("SELECT COUNT(*) FROM {}", table), [], |row| row.get(0)).unwrap_or(0);
        println!("  {:.<30} {}", format!("{}:", label), count.to_string().green());
    }

    println!("{}", "═".repeat(50).cyan());
    Ok(())
}
