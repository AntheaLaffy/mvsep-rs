// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use mvsep_api_tester::db::{
    repositories, tasks_db::TasksDatabase, user_config::UserConfigDB, Database,
};
use mvsep_api_tester::file_transfer;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, HashMap};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use tauri::{Emitter, Manager, State};
mod web_db;

// ============== 配置相关 ==============

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
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

impl Default for Config {
    fn default() -> Self {
        Self {
            token: None,
            api_url: Some("https://mvsep.com".to_string()),
            mirror: Some("main".to_string()),
            proxy_mode: Some("system".to_string()),
            proxy_host: Some("127.0.0.1".to_string()),
            proxy_port: Some("7897".to_string()),
            output_dir: Some("./output".to_string()),
            output_format: Some(1),
            poll_interval: Some(5),
            algorithm_auto_refresh_days: Some(15),
        }
    }
}

#[derive(Debug, Default, Clone, Copy)]
struct LegacyMainBackend;

type HttpClientCacheKey = (String, String, String);
type HttpClientCache = Option<(HttpClientCacheKey, reqwest::Client)>;
type BackendResult<T> = Result<T, BackendError>;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BackendError {
    pub context: Box<BackendErrorContext>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BackendErrorContext {
    pub operation: &'static str,
    pub message: String,
    pub endpoint: Option<String>,
    pub hash: Option<String>,
    pub path: Option<String>,
    pub http_status: Option<u16>,
    pub source: Option<String>,
}

impl BackendError {
    pub fn legacy(operation: &'static str, source: impl Into<String>) -> Self {
        let source = source.into();
        Self {
            context: Box::new(BackendErrorContext {
                operation,
                message: source.clone(),
                endpoint: None,
                hash: None,
                path: None,
                http_status: None,
                source: Some(source),
            }),
        }
    }

    pub fn with_endpoint(mut self, endpoint: impl Into<String>) -> Self {
        self.context.endpoint = Some(endpoint.into());
        self
    }

    pub fn with_hash(mut self, hash: impl Into<String>) -> Self {
        self.context.hash = Some(hash.into());
        self
    }

    pub fn with_path(mut self, path: impl Into<String>) -> Self {
        self.context.path = Some(path.into());
        self
    }

    pub fn with_http_status(mut self, status: u16) -> Self {
        self.context.http_status = Some(status);
        self
    }

    pub fn to_log_message(&self) -> String {
        let mut parts = vec![
            format!("operation={}", self.context.operation),
            format!("message={}", self.context.message),
        ];
        if let Some(path) = &self.context.path {
            parts.push(format!("path={}", path));
        }
        if let Some(endpoint) = &self.context.endpoint {
            parts.push(format!("endpoint={}", endpoint));
        }
        if let Some(hash) = &self.context.hash {
            parts.push(format!("hash={}", hash));
        }
        if let Some(status) = self.context.http_status {
            parts.push(format!("http_status={}", status));
        }
        if let Some(source) = &self.context.source {
            parts.push(format!("source={}", source));
        }
        parts.join(" ")
    }

    pub fn into_tauri_error(self) -> String {
        redact_sensitive(&self.context.message)
    }
}

#[derive(Debug, Clone)]
pub struct BackendPaths {
    pub app_config_dir: PathBuf,
    pub app_data_dir: PathBuf,
    pub legacy_config_json_path: PathBuf,
    pub mvsep_db_path: PathBuf,
    pub user_config_db_path: PathBuf,
    pub tasks_db_path: PathBuf,
    pub web_db_path: PathBuf,
}

impl BackendPaths {
    fn new(
        app_config_dir: PathBuf,
        app_data_dir: PathBuf,
        legacy_config_json_path: PathBuf,
    ) -> Self {
        Self {
            mvsep_db_path: app_data_dir.join("mvsep.db"),
            user_config_db_path: app_data_dir.join("user_config.db"),
            tasks_db_path: app_data_dir.join("tasks.db"),
            web_db_path: app_data_dir.join("web.db"),
            app_config_dir,
            app_data_dir,
            legacy_config_json_path,
        }
    }

    #[cfg(test)]
    fn fallback() -> Self {
        let legacy_config_json_path = get_config_path();
        let app_config_dir = legacy_config_json_path
            .parent()
            .map(Path::to_path_buf)
            .unwrap_or_else(|| PathBuf::from("."));
        let app_data_dir = dirs::data_dir()
            .unwrap_or_else(|| app_config_dir.clone())
            .join("mvsep-gui");
        Self::new(app_config_dir, app_data_dir, legacy_config_json_path)
    }
}

pub struct AppState {
    backend: LegacyMainBackend,
    pub paths: BackendPaths,
    pub config: Mutex<Config>,
    pub tasks: Mutex<HashMap<String, TaskInfo>>,
    pub backend_logs: Mutex<Vec<LogEntry>>,
    pub last_queue_info: Mutex<Option<(i32, i32)>>,
    pub download_cancellations: Mutex<HashMap<String, Arc<AtomicBool>>>,
    pub http_client_cache: Mutex<HttpClientCache>,
}

#[allow(clippy::too_many_arguments)]
trait AppBackend {
    fn load_config(&self, state: &AppState) -> BackendResult<Config>;
    fn save_config(&self, state: &AppState, config: Config) -> BackendResult<()>;
    fn resolve_path(&self, state: &AppState, path: String) -> BackendResult<String>;
    fn get_algorithm_cache_path_cmd(&self, state: &AppState) -> BackendResult<String>;
    async fn open_in_file_manager(&self, state: &AppState, path: String) -> BackendResult<()>;
    async fn test_connection(
        &self,
        state: &AppState,
        token: String,
        api_url: String,
    ) -> BackendResult<bool>;
    async fn fetch_latest_algorithm_info(
        &self,
        state: &AppState,
        api_url: String,
        token: String,
        proxy_mode: Option<String>,
        proxy_host: Option<String>,
        proxy_port: Option<String>,
    ) -> BackendResult<FetchLatestAlgorithmInfoResult>;
    fn refresh_algorithm_list_from_local(
        &self,
        state: &AppState,
    ) -> BackendResult<LocalAlgorithmListResponse>;
    fn get_algorithm_details_from_local(
        &self,
        state: &AppState,
        algorithm_id: i32,
    ) -> BackendResult<AlgorithmDetails>;
    async fn list_algorithms(
        &self,
        state: &AppState,
        keywords: Option<String>,
        group_id: Option<i32>,
        algorithm_id: Option<i32>,
        recursive: Option<bool>,
        api_url: String,
        token: String,
    ) -> BackendResult<serde_json::Value>;
    async fn get_algorithm_details(
        &self,
        state: &AppState,
        algorithm_id: i32,
        api_url: String,
        token: String,
    ) -> BackendResult<AlgorithmDetails>;
    async fn list_formats(
        &self,
        state: &AppState,
        api_url: String,
        token: String,
    ) -> BackendResult<Vec<OutputFormat>>;
    async fn query_name(
        &self,
        state: &AppState,
        algorithm_id: i32,
        model_id: Option<String>,
        api_url: String,
        token: String,
    ) -> BackendResult<String>;
    fn cancel_download(&self, state: &AppState, hash: String) -> BackendResult<()>;
    async fn get_queue_info(
        &self,
        state: &AppState,
        api_url: String,
        token: String,
    ) -> BackendResult<QueueStatus>;
    async fn get_remote_history(
        &self,
        state: &AppState,
        limit: Option<i32>,
        api_url: String,
        token: String,
    ) -> BackendResult<serde_json::Value>;
    async fn create_task(
        &self,
        state: &AppState,
        window: tauri::Window,
        file_path: String,
        sep_type: i32,
        options: std::collections::HashMap<String, Option<i32>>,
        output_format: Option<i32>,
        demo: bool,
        api_url: String,
        token: String,
    ) -> BackendResult<String>;
    async fn get_task_status(
        &self,
        state: &AppState,
        hash: String,
        api_url: String,
        token: String,
    ) -> BackendResult<TaskStatus>;
    async fn download_result(
        &self,
        state: &AppState,
        window: tauri::Window,
        hash: String,
        output_dir: String,
        file_index: Option<i32>,
        original_file_name: Option<String>,
        api_url: String,
        token: String,
    ) -> BackendResult<Vec<String>>;
    fn get_tasks(&self, state: &AppState) -> BackendResult<Vec<TaskInfo>>;
    fn add_task(&self, state: &AppState, task: TaskInfo) -> BackendResult<()>;
    fn replace_active_tasks(&self, state: &AppState, tasks: Vec<TaskInfo>) -> BackendResult<()>;
    fn update_task_status(
        &self,
        state: &AppState,
        hash: String,
        status: String,
        progress: f32,
        files: Option<Vec<String>>,
        error: Option<String>,
    ) -> BackendResult<()>;
    fn remove_task(&self, state: &AppState, hash: String) -> BackendResult<()>;
    fn get_task_history(&self, state: &AppState) -> BackendResult<Vec<TaskHistoryRecord>>;
    fn save_task_history(
        &self,
        state: &AppState,
        records: Vec<TaskHistoryRecord>,
    ) -> BackendResult<()>;
    fn complete_task(
        &self,
        state: &AppState,
        task: TaskInfo,
        record: TaskHistoryRecord,
    ) -> BackendResult<()>;
    fn get_backend_logs(&self, state: &AppState) -> BackendResult<Vec<LogEntry>>;
    fn frontend_debug_log(&self, state: &AppState, level: String, message: String);
}

fn legacy_backend_result<T>(
    operation: &'static str,
    result: Result<T, String>,
) -> BackendResult<T> {
    result.map_err(|e| BackendError::legacy(operation, e))
}

fn transfer_backend_error(
    operation: &'static str,
    err: file_transfer::TransferError,
    fallback_endpoint: impl Into<String>,
    hash: Option<String>,
    fallback_path: Option<PathBuf>,
) -> BackendError {
    let mut backend_error = BackendError::legacy(operation, err.to_string());
    if let Some(endpoint) = err.url() {
        backend_error = backend_error.with_endpoint(endpoint.to_string());
    } else {
        backend_error = backend_error.with_endpoint(fallback_endpoint.into());
    }
    if let Some(hash) = hash {
        backend_error = backend_error.with_hash(hash);
    }
    if let Some(path) = err.path().map(Path::to_path_buf).or(fallback_path) {
        backend_error = backend_error.with_path(path.to_string_lossy().into_owned());
    }
    if let Some(status) = err.http_status() {
        backend_error = backend_error.with_http_status(status);
    }
    backend_error
}

fn open_app_database(paths: &BackendPaths) -> Result<Database, String> {
    if let Some(parent) = paths.mvsep_db_path.parent() {
        fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }
    Database::new(Some(&paths.mvsep_db_path.to_string_lossy())).map_err(|e| e.to_string())
}

fn open_user_config_database(paths: &BackendPaths) -> Result<UserConfigDB, String> {
    if let Some(parent) = paths.user_config_db_path.parent() {
        fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }
    UserConfigDB::new(&paths.user_config_db_path.to_string_lossy()).map_err(|e| e.to_string())
}

fn open_tasks_database(paths: &BackendPaths) -> Result<TasksDatabase, String> {
    if let Some(parent) = paths.tasks_db_path.parent() {
        fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }
    TasksDatabase::new(Some(&paths.tasks_db_path.to_string_lossy())).map_err(|e| e.to_string())
}

fn resolve_backend_path(paths: &BackendPaths, path: impl AsRef<Path>) -> PathBuf {
    let path = path.as_ref();
    if path.is_absolute() {
        path.to_path_buf()
    } else {
        paths.app_data_dir.join(path)
    }
}

fn default_task_phase(status: &str) -> String {
    match status {
        "done" => "done",
        "failed" => "failed",
        "downloading" => "downloading",
        "processing" | "separating" => "separating",
        _ => "queueing",
    }
    .to_string()
}

fn is_terminal_task_status(status: &str) -> bool {
    matches!(status, "done" | "failed")
}

fn task_info_to_row(task: &TaskInfo) -> Result<repositories::TaskRow, String> {
    let output_files = serde_json::to_string(&task.output_files).map_err(|e| e.to_string())?;
    Ok(repositories::TaskRow {
        hash: task.hash.clone(),
        file_name: task.file_name.clone(),
        algorithm_id: task.algorithm_id,
        algorithm_name: task.algorithm_name.clone(),
        model_id: task.model_id,
        model_name: task.model_name.clone(),
        model2_id: task.model2_id,
        model2_name: task.model2_name.clone(),
        model3_id: task.model3_id,
        model3_name: task.model3_name.clone(),
        format: task.format,
        status: task.status.clone(),
        progress: task.progress as f64,
        created_at: task.created_at,
        output_files,
        error: task.error.clone(),
        message: task.message.clone(),
        queue_count: task.queue_count,
        current_order: task.current_order,
        phase: task
            .phase
            .clone()
            .unwrap_or_else(|| default_task_phase(&task.status)),
        download_file_name: task.download_file_name.clone(),
        download_bytes: task.download_bytes.unwrap_or(0),
        download_total_bytes: task.download_total_bytes,
        download_speed_bps: task.download_speed_bps.unwrap_or(0.0),
        download_percent: task.download_percent.unwrap_or(0.0),
    })
}

fn parse_output_files_json(raw: &str, row_id: &str) -> Result<Vec<String>, String> {
    let value = serde_json::from_str::<serde_json::Value>(raw)
        .map_err(|e| format!("invalid output_files JSON for {}: {}", row_id, e))?;
    let values = value
        .as_array()
        .ok_or_else(|| format!("output_files for {} must be an array", row_id))?;
    let mut files = Vec::with_capacity(values.len());
    for item in values {
        if let Some(file) = item.as_str() {
            files.push(file.to_string());
            continue;
        }
        if let Some(object) = item.as_object() {
            for key in ["local_path", "url", "remote_name", "name", "file_name"] {
                if let Some(file) = object.get(key).and_then(|v| v.as_str()) {
                    files.push(file.to_string());
                    break;
                }
            }
        }
    }
    Ok(files)
}

fn task_row_to_info(row: repositories::TaskRow) -> Result<TaskInfo, String> {
    let output_files = parse_output_files_json(&row.output_files, &row.hash)?;
    Ok(TaskInfo {
        hash: row.hash,
        file_name: row.file_name,
        algorithm_id: row.algorithm_id,
        algorithm_name: row.algorithm_name,
        model_id: row.model_id,
        model_name: row.model_name,
        model2_id: row.model2_id,
        model2_name: row.model2_name,
        model3_id: row.model3_id,
        model3_name: row.model3_name,
        format: row.format,
        status: row.status,
        progress: row.progress as f32,
        created_at: row.created_at,
        output_files,
        error: row.error,
        message: row.message,
        queue_count: row.queue_count,
        current_order: row.current_order,
        phase: Some(row.phase),
        download_file_name: row.download_file_name,
        download_bytes: Some(row.download_bytes),
        download_total_bytes: row.download_total_bytes,
        download_speed_bps: Some(row.download_speed_bps),
        download_percent: Some(row.download_percent),
    })
}

fn task_history_to_row(record: &TaskHistoryRecord) -> Result<repositories::TaskHistoryRow, String> {
    let output_files = serde_json::to_string(&record.output_files).map_err(|e| e.to_string())?;
    Ok(repositories::TaskHistoryRow {
        id: record.id.clone(),
        file_name: record.file_name.clone(),
        algorithm_id: record.algorithm_id,
        algorithm_name: record.algorithm_name.clone(),
        model_id: record.model_id,
        model_name: record.model_name.clone(),
        model2_id: record.model2_id,
        model2_name: record.model2_name.clone(),
        model3_id: record.model3_id,
        model3_name: record.model3_name.clone(),
        format_id: record.format_id,
        format_name: record.format_name.clone(),
        status: record.status.clone(),
        created_at: record.created_at,
        completed_at: record.completed_at,
        output_files,
        output_path: record.output_path.clone(),
        error: record.error.clone(),
    })
}

fn task_history_row_to_record(
    row: repositories::TaskHistoryRow,
) -> Result<TaskHistoryRecord, String> {
    let output_files = parse_output_files_json(&row.output_files, &row.id)?;
    Ok(TaskHistoryRecord {
        id: row.id,
        file_name: row.file_name,
        algorithm_id: row.algorithm_id,
        algorithm_name: row.algorithm_name,
        model_id: row.model_id,
        model_name: row.model_name,
        model2_id: row.model2_id,
        model2_name: row.model2_name,
        model3_id: row.model3_id,
        model3_name: row.model3_name,
        format_id: row.format_id,
        format_name: row.format_name,
        status: row.status,
        created_at: row.created_at,
        completed_at: row.completed_at,
        output_files,
        output_path: row.output_path,
        error: row.error,
    })
}

fn load_tasks_from_backend_store(state: &AppState) -> Result<Vec<TaskInfo>, String> {
    let db = open_tasks_database(&state.paths)?;
    let conn = db.conn.lock().map_err(|e| e.to_string())?;
    let rows = repositories::get_all_tasks(&conn).map_err(|e| e.to_string())?;
    rows.into_iter().map(task_row_to_info).collect()
}

fn insert_task_in_backend_store(state: &AppState, task: &TaskInfo) -> Result<(), String> {
    let db = open_tasks_database(&state.paths)?;
    let conn = db.conn.lock().map_err(|e| e.to_string())?;
    let row = task_info_to_row(task)?;
    repositories::insert_task(&conn, &row).map_err(|e| e.to_string())
}

fn replace_active_tasks_in_backend_store(
    state: &AppState,
    tasks: Vec<TaskInfo>,
) -> Result<(), String> {
    let db = open_tasks_database(&state.paths)?;
    let mut conn = db.conn.lock().map_err(|e| e.to_string())?;
    let tx = conn.transaction().map_err(|e| e.to_string())?;
    tx.execute(
        "DELETE FROM tasks WHERE status NOT IN ('done', 'failed')",
        [],
    )
    .map_err(|e| e.to_string())?;
    for task in tasks
        .into_iter()
        .filter(|task| !is_terminal_task_status(&task.status))
    {
        let row = task_info_to_row(&task)?;
        repositories::insert_task(&tx, &row).map_err(|e| e.to_string())?;
    }
    tx.commit().map_err(|e| e.to_string())
}

fn update_task_status_in_backend_store(
    state: &AppState,
    hash: String,
    status: String,
    progress: f32,
    files: Option<Vec<String>>,
    error: Option<String>,
) -> Result<(), String> {
    let db = open_tasks_database(&state.paths)?;
    let conn = db.conn.lock().map_err(|e| e.to_string())?;
    repositories::update_task_status(&conn, &hash, &status, progress as f64, error.as_deref())
        .map_err(|e| e.to_string())?;
    if let Some(files) = files {
        let output_files = serde_json::to_string(&files).map_err(|e| e.to_string())?;
        repositories::update_task_output_files(&conn, &hash, &output_files)
            .map_err(|e| e.to_string())?;
    }
    Ok(())
}

fn remove_task_from_backend_store(state: &AppState, hash: String) -> Result<(), String> {
    let db = open_tasks_database(&state.paths)?;
    let conn = db.conn.lock().map_err(|e| e.to_string())?;
    repositories::delete_task(&conn, &hash)
        .map(|_| ())
        .map_err(|e| e.to_string())
}

fn load_task_history_from_backend_store(
    state: &AppState,
) -> Result<Vec<TaskHistoryRecord>, String> {
    let db = open_tasks_database(&state.paths)?;
    let conn = db.conn.lock().map_err(|e| e.to_string())?;
    let rows = repositories::get_all_task_history(&conn).map_err(|e| e.to_string())?;
    rows.into_iter().map(task_history_row_to_record).collect()
}

fn replace_task_history_in_backend_store(
    state: &AppState,
    records: Vec<TaskHistoryRecord>,
) -> Result<(), String> {
    let db = open_tasks_database(&state.paths)?;
    let mut conn = db.conn.lock().map_err(|e| e.to_string())?;
    let tx = conn.transaction().map_err(|e| e.to_string())?;
    repositories::clear_task_history(&tx).map_err(|e| e.to_string())?;
    for record in records {
        let row = task_history_to_row(&record)?;
        repositories::insert_task_history(&tx, &row).map_err(|e| e.to_string())?;
    }
    repositories::trim_task_history(&tx, 100).map_err(|e| e.to_string())?;
    tx.commit().map_err(|e| e.to_string())
}

fn complete_task_in_backend_store(
    state: &AppState,
    task: &TaskInfo,
    record: &TaskHistoryRecord,
) -> Result<(), String> {
    let db = open_tasks_database(&state.paths)?;
    let mut conn = db.conn.lock().map_err(|e| e.to_string())?;
    let tx = conn.transaction().map_err(|e| e.to_string())?;
    if is_terminal_task_status(&task.status) {
        repositories::delete_task(&tx, &task.hash).map_err(|e| e.to_string())?;
    } else {
        let task_row = task_info_to_row(task)?;
        repositories::insert_task(&tx, &task_row).map_err(|e| e.to_string())?;
    }
    let history_row = task_history_to_row(record)?;
    repositories::insert_task_history(&tx, &history_row).map_err(|e| e.to_string())?;
    repositories::trim_task_history(&tx, 100).map_err(|e| e.to_string())?;
    tx.commit().map_err(|e| e.to_string())
}

fn merge_config(base: Config, patch: Config) -> Config {
    Config {
        token: patch.token.or(base.token),
        api_url: patch.api_url.or(base.api_url),
        mirror: patch.mirror.or(base.mirror),
        proxy_mode: patch.proxy_mode.or(base.proxy_mode),
        proxy_host: patch.proxy_host.or(base.proxy_host),
        proxy_port: patch.proxy_port.or(base.proxy_port),
        output_dir: patch.output_dir.or(base.output_dir),
        output_format: patch.output_format.or(base.output_format),
        poll_interval: patch.poll_interval.or(base.poll_interval),
        algorithm_auto_refresh_days: patch
            .algorithm_auto_refresh_days
            .or(base.algorithm_auto_refresh_days),
    }
}

fn read_legacy_config_json(path: &Path) -> Result<Option<Config>, String> {
    if !path.exists() {
        return Ok(None);
    }
    let content = fs::read_to_string(path).map_err(|e| e.to_string())?;
    serde_json::from_str(&content)
        .map(Some)
        .map_err(|e| e.to_string())
}

fn save_config_to_user_config_db(paths: &BackendPaths, config: &Config) -> Result<(), String> {
    let db = open_user_config_database(paths)?;
    db.set_json("config", config).map_err(|e| e.to_string())
}

fn load_config_from_backend_store(state: &AppState) -> Result<Config, String> {
    let db = open_user_config_database(&state.paths)?;

    if let Some(stored) = db.get_json::<Config>("config").map_err(|e| e.to_string())? {
        let config = merge_config(Config::default(), stored);
        if let Ok(mut guard) = state.config.lock() {
            *guard = config.clone();
        }
        return Ok(config);
    }

    let config = match read_legacy_config_json(&state.paths.legacy_config_json_path) {
        Ok(Some(legacy)) => merge_config(Config::default(), legacy),
        Ok(None) => Config::default(),
        Err(e) => {
            return Err(format!(
                "legacy config import failed at {}: {}",
                state.paths.legacy_config_json_path.to_string_lossy(),
                e
            ));
        }
    };
    db.set_json("config", &config).map_err(|e| e.to_string())?;

    if let Ok(mut guard) = state.config.lock() {
        *guard = config.clone();
    }
    Ok(config)
}

fn save_config_to_backend_store(state: &AppState, config: Config) -> Result<(), String> {
    let current = open_user_config_database(&state.paths)?
        .get_json::<Config>("config")
        .map_err(|e| e.to_string())?
        .unwrap_or_else(Config::default);
    let config = merge_config(merge_config(Config::default(), current), config);
    save_config_to_user_config_db(&state.paths, &config)?;
    if let Ok(mut guard) = state.config.lock() {
        *guard = config;
    }
    Ok(())
}

fn load_output_formats_from_backend_store(state: &AppState) -> Result<Vec<OutputFormat>, String> {
    let db = open_app_database(&state.paths)?;
    db.with_conn(|conn| {
        let mut formats = repositories::get_all_output_formats(conn)?;
        if formats.is_empty() {
            repositories::init_default_output_formats(conn)?;
            formats = repositories::get_all_output_formats(conn)?;
        }
        Ok(formats)
    })
    .map_err(|e| e.to_string())
    .map(|formats| {
        formats
            .into_iter()
            .map(|format| OutputFormat {
                id: format.id,
                name: format.name,
            })
            .collect()
    })
}

fn parse_algorithm_options(options: Option<&str>) -> HashMap<String, String> {
    let mut parsed = HashMap::new();
    let Some(options) = options else {
        return parsed;
    };
    let Ok(value) = serde_json::from_str::<serde_json::Value>(options) else {
        return parsed;
    };
    let Some(obj) = value.as_object() else {
        return parsed;
    };

    for (key, value) in obj {
        if let Some(text) = value.as_str() {
            parsed.insert(key.clone(), text.to_string());
        } else {
            parsed.insert(key.clone(), value.to_string());
        }
    }
    parsed
}

fn algorithm_cache_rows_from_values(
    algorithms: &[serde_json::Value],
) -> (
    Vec<repositories::AlgorithmGroupRow>,
    Vec<repositories::AlgorithmRow>,
    Vec<repositories::AlgorithmFieldRow>,
) {
    let mut groups = BTreeMap::<i32, String>::new();
    let mut algorithm_rows = Vec::new();
    let mut field_rows = Vec::new();

    for algo in algorithms {
        let algo_id = read_i32(algo.get("render_id")).unwrap_or(0);
        let algo_name = algo
            .get("name")
            .and_then(|v| v.as_str())
            .unwrap_or("Unknown")
            .to_string();
        let group_id = read_i32(algo.get("algorithm_group").and_then(|g| g.get("id"))).unwrap_or(0);
        let group_name = algo
            .get("algorithm_group")
            .and_then(|g| g.get("name"))
            .and_then(|v| v.as_str())
            .unwrap_or("Ungrouped")
            .to_string();
        let orientation = read_i32(algo.get("orientation")).unwrap_or(0);

        groups.entry(group_id).or_insert(group_name);
        algorithm_rows.push(repositories::AlgorithmRow {
            id: algo_id,
            name: algo_name,
            group_id,
            price_coefficient: 1.0,
            orientation,
        });

        if let Some(fields) = algo.get("algorithm_fields").and_then(|f| f.as_array()) {
            for field in fields {
                let field_id = read_i32(field.get("id")).unwrap_or(0);
                let field_name = field
                    .get("name")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string();
                let field_text = field
                    .get("text")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string();
                let field_options = field
                    .get("options")
                    .and_then(|v| v.as_str())
                    .unwrap_or("{}")
                    .to_string();
                let field_default = field
                    .get("default_key")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string();

                field_rows.push(repositories::AlgorithmFieldRow {
                    id: field_id as i64,
                    algorithm_id: algo_id,
                    name: field_name,
                    text: Some(field_text),
                    options: Some(field_options),
                    default_key: Some(field_default),
                });
            }
        }
    }

    let group_rows = groups
        .into_iter()
        .map(|(id, name)| repositories::AlgorithmGroupRow { id, name })
        .collect();
    (group_rows, algorithm_rows, field_rows)
}

fn algorithm_details_from_rows(
    algo: repositories::AlgorithmRow,
    mut fields: Vec<repositories::AlgorithmFieldRow>,
) -> AlgorithmDetails {
    fields.sort_by_key(|field| field.id);
    AlgorithmDetails {
        id: algo.id,
        name: algo.name,
        fields: fields
            .into_iter()
            .map(|field| AlgorithmField {
                name: field.name,
                text: field.text.unwrap_or_default(),
                options: parse_algorithm_options(field.options.as_deref()),
            })
            .collect(),
    }
}

fn save_algorithm_cache_updated_at(state: &AppState, updated_at: &str) -> Result<(), String> {
    let path = state.paths.user_config_db_path.to_string_lossy();
    let db = open_user_config_database(&state.paths)
        .map_err(|e| format!("algorithm cache metadata write failed at {}: {}", path, e))?;
    db.set_string("algorithm_cache_updated_at", updated_at)
        .map_err(|e| format!("algorithm cache metadata write failed at {}: {}", path, e))?;
    let seconds = updated_at
        .split('.')
        .next()
        .and_then(|v| v.parse::<i64>().ok())
        .unwrap_or(0);
    db.set_int("algorithm_last_fetched_at", seconds)
        .map_err(|e| format!("algorithm cache metadata write failed at {}: {}", path, e))
}

fn load_algorithm_cache_updated_at(state: &AppState) -> Result<String, String> {
    let path = state.paths.user_config_db_path.to_string_lossy();
    let db = open_user_config_database(&state.paths)
        .map_err(|e| format!("algorithm cache metadata failed at {}: {}", path, e))?;
    if let Some(updated_at) = db
        .get_string("algorithm_cache_updated_at")
        .map_err(|e| format!("algorithm cache metadata failed at {}: {}", path, e))?
    {
        return Ok(updated_at);
    }
    Ok(db
        .get_int("algorithm_last_fetched_at")
        .map_err(|e| format!("algorithm cache metadata failed at {}: {}", path, e))?
        .map(|value| value.to_string())
        .unwrap_or_default())
}

fn replace_algorithm_cache_in_backend_store(
    state: &AppState,
    algorithms: &[serde_json::Value],
) -> Result<(), String> {
    let (groups, algorithms, fields) = algorithm_cache_rows_from_values(algorithms);
    let path = state.paths.mvsep_db_path.to_string_lossy();
    let db = open_app_database(&state.paths)
        .map_err(|e| format!("algorithm cache db write failed at {}: {}", path, e))?;
    let mut conn = db
        .conn
        .lock()
        .map_err(|e| format!("algorithm cache db write failed at {}: {}", path, e))?;
    repositories::replace_algorithm_cache(&mut conn, &groups, &algorithms, &fields)
        .map_err(|e| format!("algorithm cache db write failed at {}: {}", path, e))
}

fn load_algorithm_list_from_backend_store(
    state: &AppState,
) -> Result<LocalAlgorithmListResponse, String> {
    let path = state.paths.mvsep_db_path.to_string_lossy();
    let db = open_app_database(&state.paths)
        .map_err(|e| format!("algorithm cache db read failed at {}: {}", path, e))?;
    let groups = db
        .with_conn(|conn| {
            let group_rows = repositories::get_all_algorithm_groups(conn)?;
            let algorithm_rows = repositories::get_all_algorithms(conn)?;
            let group_names: BTreeMap<i32, String> = group_rows
                .into_iter()
                .map(|group| (group.id, group.name))
                .collect();
            let mut grouped = BTreeMap::<i32, AlgorithmGroupData>::new();

            for algo in algorithm_rows {
                let group_name = group_names
                    .get(&algo.group_id)
                    .cloned()
                    .unwrap_or_else(|| "Ungrouped".to_string());
                grouped
                    .entry(algo.group_id)
                    .or_insert_with(|| AlgorithmGroupData {
                        id: algo.group_id,
                        name: group_name,
                        algorithms: Vec::new(),
                    })
                    .algorithms
                    .push(AlgorithmItem {
                        id: algo.id,
                        name: algo.name,
                        group_id: algo.group_id,
                    });
            }

            Ok(grouped.into_values().collect::<Vec<_>>())
        })
        .map_err(|e| format!("algorithm cache db read failed at {}: {}", path, e))?;
    let total_algorithms = get_total_algorithms(&groups);
    let updated_at = load_algorithm_cache_updated_at(state)?;
    Ok(LocalAlgorithmListResponse {
        updated_at,
        groups,
        total_algorithms,
    })
}

fn load_algorithm_details_from_backend_store(
    state: &AppState,
    algorithm_id: i32,
) -> Result<AlgorithmDetails, String> {
    let path = state.paths.mvsep_db_path.to_string_lossy();
    let db = open_app_database(&state.paths)
        .map_err(|e| format!("algorithm cache db read failed at {}: {}", path, e))?;
    let details = db
        .with_conn(|conn| repositories::get_algorithm_details_with_fields(conn, algorithm_id))
        .map_err(|e| format!("algorithm cache db read failed at {}: {}", path, e))?;
    details
        .map(|(algo, fields)| algorithm_details_from_rows(algo, fields))
        .ok_or_else(|| "Algorithm not found in local cache".to_string())
}

fn algorithm_cache_error_path(state: &AppState, message: &str) -> String {
    let user_config_path = state.paths.user_config_db_path.to_string_lossy();
    if message.contains(user_config_path.as_ref()) || message.contains("algorithm cache metadata") {
        return user_config_path.into_owned();
    }
    state.paths.mvsep_db_path.to_string_lossy().into_owned()
}

impl AppBackend for LegacyMainBackend {
    fn load_config(&self, state: &AppState) -> BackendResult<Config> {
        load_config_from_backend_store(state).map_err(|e| {
            let path = if e.starts_with("legacy config import failed") {
                &state.paths.legacy_config_json_path
            } else {
                &state.paths.user_config_db_path
            };
            BackendError::legacy("load_config", e).with_path(path.to_string_lossy().into_owned())
        })
    }

    fn save_config(&self, state: &AppState, config: Config) -> BackendResult<()> {
        save_config_to_backend_store(state, config).map_err(|e| {
            BackendError::legacy("save_config", e).with_path(
                state
                    .paths
                    .user_config_db_path
                    .to_string_lossy()
                    .into_owned(),
            )
        })
    }

    fn resolve_path(&self, state: &AppState, path: String) -> BackendResult<String> {
        let requested_path = path.clone();
        legacy_resolve_path(state, path)
            .map_err(|e| BackendError::legacy("resolve_path", e).with_path(requested_path))
    }

    fn get_algorithm_cache_path_cmd(&self, state: &AppState) -> BackendResult<String> {
        Ok(state.paths.mvsep_db_path.to_string_lossy().into_owned())
    }

    async fn open_in_file_manager(&self, state: &AppState, path: String) -> BackendResult<()> {
        let requested_path = path.clone();
        legacy_open_in_file_manager(state, path)
            .await
            .map_err(|e| BackendError::legacy("open_in_file_manager", e).with_path(requested_path))
    }

    async fn test_connection(
        &self,
        state: &AppState,
        token: String,
        api_url: String,
    ) -> BackendResult<bool> {
        let endpoint = build_api_url(&api_url, "/app/algorithms");
        legacy_test_connection(state, token, api_url)
            .await
            .map_err(|e| BackendError::legacy("test_connection", e).with_endpoint(endpoint))
    }

    async fn fetch_latest_algorithm_info(
        &self,
        state: &AppState,
        api_url: String,
        token: String,
        proxy_mode: Option<String>,
        proxy_host: Option<String>,
        proxy_port: Option<String>,
    ) -> BackendResult<FetchLatestAlgorithmInfoResult> {
        let endpoint = build_api_url(&api_url, "/app/algorithms");
        legacy_fetch_latest_algorithm_info(
            state, api_url, token, proxy_mode, proxy_host, proxy_port,
        )
        .await
        .map_err(|e| {
            if e.contains("algorithm cache db") || e.contains("algorithm cache metadata") {
                let path = algorithm_cache_error_path(state, &e);
                BackendError::legacy("fetch_latest_algorithm_info", e).with_path(path)
            } else {
                BackendError::legacy("fetch_latest_algorithm_info", e).with_endpoint(endpoint)
            }
        })
    }

    fn refresh_algorithm_list_from_local(
        &self,
        state: &AppState,
    ) -> BackendResult<LocalAlgorithmListResponse> {
        load_algorithm_list_from_backend_store(state).map_err(|e| {
            let path = algorithm_cache_error_path(state, &e);
            BackendError::legacy("refresh_algorithm_list_from_local", e).with_path(path)
        })
    }

    fn get_algorithm_details_from_local(
        &self,
        state: &AppState,
        algorithm_id: i32,
    ) -> BackendResult<AlgorithmDetails> {
        load_algorithm_details_from_backend_store(state, algorithm_id).map_err(|e| {
            BackendError::legacy("get_algorithm_details_from_local", e)
                .with_path(state.paths.mvsep_db_path.to_string_lossy().into_owned())
        })
    }

    async fn list_algorithms(
        &self,
        state: &AppState,
        keywords: Option<String>,
        group_id: Option<i32>,
        algorithm_id: Option<i32>,
        recursive: Option<bool>,
        api_url: String,
        token: String,
    ) -> BackendResult<serde_json::Value> {
        let endpoint = build_api_url(&api_url, "/app/algorithms");
        legacy_list_algorithms(
            state,
            keywords,
            group_id,
            algorithm_id,
            recursive,
            api_url,
            token,
        )
        .await
        .map_err(|e| BackendError::legacy("list_algorithms", e).with_endpoint(endpoint))
    }

    async fn get_algorithm_details(
        &self,
        state: &AppState,
        algorithm_id: i32,
        api_url: String,
        token: String,
    ) -> BackendResult<AlgorithmDetails> {
        let endpoint = build_api_url(&api_url, "/app/algorithms");
        legacy_get_algorithm_details(state, algorithm_id, api_url, token)
            .await
            .map_err(|e| BackendError::legacy("get_algorithm_details", e).with_endpoint(endpoint))
    }

    async fn list_formats(
        &self,
        state: &AppState,
        api_url: String,
        token: String,
    ) -> BackendResult<Vec<OutputFormat>> {
        let _ = (api_url, token);
        load_output_formats_from_backend_store(state).map_err(|e| {
            BackendError::legacy("list_formats", e)
                .with_path(state.paths.mvsep_db_path.to_string_lossy().into_owned())
        })
    }

    async fn query_name(
        &self,
        state: &AppState,
        algorithm_id: i32,
        model_id: Option<String>,
        api_url: String,
        token: String,
    ) -> BackendResult<String> {
        let endpoint = build_api_url(&api_url, "/app/algorithms");
        legacy_query_name(state, algorithm_id, model_id, api_url, token)
            .await
            .map_err(|e| BackendError::legacy("query_name", e).with_endpoint(endpoint))
    }

    fn cancel_download(&self, state: &AppState, hash: String) -> BackendResult<()> {
        let task_hash = hash.clone();
        legacy_cancel_download(state, hash)
            .map_err(|e| BackendError::legacy("cancel_download", e).with_hash(task_hash))
    }

    async fn get_queue_info(
        &self,
        state: &AppState,
        api_url: String,
        token: String,
    ) -> BackendResult<QueueStatus> {
        let endpoint = build_api_url(&api_url, "/app/queue");
        legacy_get_queue_info(state, api_url, token)
            .await
            .map_err(|e| BackendError::legacy("get_queue_info", e).with_endpoint(endpoint))
    }

    async fn get_remote_history(
        &self,
        state: &AppState,
        limit: Option<i32>,
        api_url: String,
        token: String,
    ) -> BackendResult<serde_json::Value> {
        let endpoint = build_api_url(&api_url, "/app/separation_history");
        legacy_get_remote_history(state, limit, api_url, token)
            .await
            .map_err(|e| BackendError::legacy("get_remote_history", e).with_endpoint(endpoint))
    }

    async fn create_task(
        &self,
        state: &AppState,
        window: tauri::Window,
        file_path: String,
        sep_type: i32,
        options: std::collections::HashMap<String, Option<i32>>,
        output_format: Option<i32>,
        demo: bool,
        api_url: String,
        token: String,
    ) -> BackendResult<String> {
        legacy_create_task(
            state,
            window,
            file_path,
            sep_type,
            options,
            output_format,
            demo,
            api_url,
            token,
        )
        .await
    }

    async fn get_task_status(
        &self,
        state: &AppState,
        hash: String,
        api_url: String,
        token: String,
    ) -> BackendResult<TaskStatus> {
        let endpoint = build_api_url(&api_url, "/separation/get");
        let task_hash = hash.clone();
        legacy_get_task_status(state, hash, api_url, token)
            .await
            .map_err(|e| {
                BackendError::legacy("get_task_status", e)
                    .with_endpoint(endpoint)
                    .with_hash(task_hash)
            })
    }

    async fn download_result(
        &self,
        state: &AppState,
        window: tauri::Window,
        hash: String,
        output_dir: String,
        file_index: Option<i32>,
        original_file_name: Option<String>,
        api_url: String,
        token: String,
    ) -> BackendResult<Vec<String>> {
        legacy_download_result(
            state,
            window,
            hash,
            output_dir,
            file_index,
            original_file_name,
            api_url,
            token,
        )
        .await
    }

    fn get_tasks(&self, state: &AppState) -> BackendResult<Vec<TaskInfo>> {
        load_tasks_from_backend_store(state).map_err(|e| {
            BackendError::legacy("get_tasks", e)
                .with_path(state.paths.tasks_db_path.to_string_lossy().into_owned())
        })
    }

    fn add_task(&self, state: &AppState, task: TaskInfo) -> BackendResult<()> {
        insert_task_in_backend_store(state, &task).map_err(|e| {
            BackendError::legacy("add_task", e)
                .with_hash(task.hash)
                .with_path(state.paths.tasks_db_path.to_string_lossy().into_owned())
        })
    }

    fn replace_active_tasks(&self, state: &AppState, tasks: Vec<TaskInfo>) -> BackendResult<()> {
        replace_active_tasks_in_backend_store(state, tasks).map_err(|e| {
            BackendError::legacy("replace_active_tasks", e)
                .with_path(state.paths.tasks_db_path.to_string_lossy().into_owned())
        })
    }

    fn update_task_status(
        &self,
        state: &AppState,
        hash: String,
        status: String,
        progress: f32,
        files: Option<Vec<String>>,
        error: Option<String>,
    ) -> BackendResult<()> {
        let task_hash = hash.clone();
        update_task_status_in_backend_store(state, hash, status, progress, files, error).map_err(
            |e| {
                BackendError::legacy("update_task_status", e)
                    .with_hash(task_hash)
                    .with_path(state.paths.tasks_db_path.to_string_lossy().into_owned())
            },
        )
    }

    fn remove_task(&self, state: &AppState, hash: String) -> BackendResult<()> {
        let task_hash = hash.clone();
        remove_task_from_backend_store(state, hash).map_err(|e| {
            BackendError::legacy("remove_task", e)
                .with_hash(task_hash)
                .with_path(state.paths.tasks_db_path.to_string_lossy().into_owned())
        })
    }

    fn get_task_history(&self, state: &AppState) -> BackendResult<Vec<TaskHistoryRecord>> {
        load_task_history_from_backend_store(state).map_err(|e| {
            BackendError::legacy("get_task_history", e)
                .with_path(state.paths.tasks_db_path.to_string_lossy().into_owned())
        })
    }

    fn save_task_history(
        &self,
        state: &AppState,
        records: Vec<TaskHistoryRecord>,
    ) -> BackendResult<()> {
        replace_task_history_in_backend_store(state, records).map_err(|e| {
            BackendError::legacy("save_task_history", e)
                .with_path(state.paths.tasks_db_path.to_string_lossy().into_owned())
        })
    }

    fn complete_task(
        &self,
        state: &AppState,
        task: TaskInfo,
        record: TaskHistoryRecord,
    ) -> BackendResult<()> {
        let task_hash = task.hash.clone();
        complete_task_in_backend_store(state, &task, &record).map_err(|e| {
            BackendError::legacy("complete_task", e)
                .with_hash(task_hash)
                .with_path(state.paths.tasks_db_path.to_string_lossy().into_owned())
        })
    }

    fn get_backend_logs(&self, state: &AppState) -> BackendResult<Vec<LogEntry>> {
        legacy_backend_result("get_backend_logs", legacy_get_backend_logs(state))
    }

    fn frontend_debug_log(&self, state: &AppState, level: String, message: String) {
        legacy_frontend_debug_log(state, level, message)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskInfo {
    pub hash: String,
    pub file_name: String,
    pub algorithm_id: i32,
    pub algorithm_name: String,
    pub model_id: Option<i32>,
    pub model_name: Option<String>,
    #[serde(default)]
    pub model2_id: Option<i32>,
    #[serde(default)]
    pub model2_name: Option<String>,
    #[serde(default)]
    pub model3_id: Option<i32>,
    #[serde(default)]
    pub model3_name: Option<String>,
    pub format: i32,
    pub status: String,
    pub progress: f32,
    pub created_at: i64,
    pub output_files: Vec<String>,
    pub error: Option<String>,
    #[serde(default)]
    pub message: Option<String>,
    #[serde(default)]
    pub queue_count: Option<i32>,
    #[serde(default)]
    pub current_order: Option<i32>,
    #[serde(default)]
    pub phase: Option<String>,
    #[serde(default)]
    pub download_file_name: Option<String>,
    #[serde(default)]
    pub download_bytes: Option<i64>,
    #[serde(default)]
    pub download_total_bytes: Option<i64>,
    #[serde(default)]
    pub download_speed_bps: Option<f64>,
    #[serde(default)]
    pub download_percent: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TaskHistoryRecord {
    pub id: String,
    pub file_name: String,
    pub algorithm_id: i32,
    pub algorithm_name: String,
    pub model_id: Option<i32>,
    pub model_name: Option<String>,
    #[serde(default)]
    pub model2_id: Option<i32>,
    #[serde(default)]
    pub model2_name: Option<String>,
    #[serde(default)]
    pub model3_id: Option<i32>,
    #[serde(default)]
    pub model3_name: Option<String>,
    pub format_id: i32,
    pub format_name: String,
    pub status: String,
    pub created_at: i64,
    pub completed_at: Option<i64>,
    pub output_files: Vec<String>,
    pub output_path: Option<String>,
    pub error: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogEntry {
    pub timestamp: String,
    pub level: String,
    pub message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OutputFormat {
    pub id: i32,
    pub name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AlgorithmItem {
    #[serde(default)]
    pub id: i32,
    #[serde(default)]
    pub name: String,
    #[serde(default)]
    pub group_id: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AlgorithmGroupData {
    #[serde(default)]
    pub id: i32,
    #[serde(default)]
    pub name: String,
    #[serde(default)]
    pub algorithms: Vec<AlgorithmItem>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskStatus {
    pub status: String,
    pub progress: f32,
    pub message: Option<String>,
    pub files: Option<Vec<String>>,
    pub queue_count: Option<i32>,
    pub current_order: Option<i32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueueStatus {
    pub active: i32,
    pub queued: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DownloadProgressPayload {
    pub hash: String,
    pub file_name: String,
    pub downloaded_bytes: u64,
    pub total_bytes: Option<u64>,
    pub speed_bps: f64,
    pub percent: f32,
    pub done: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UploadProgressPayload {
    pub file_name: String,
    pub uploaded_bytes: u64,
    pub total_bytes: u64,
    pub speed_bps: f64,
    pub percent: f32,
    pub done: bool,
    pub failed: bool,
}

fn get_config_path() -> PathBuf {
    dirs::config_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("mvsep-gui")
        .join("config.json")
}

fn build_api_url(base: &str, path: &str) -> String {
    let trimmed = base.trim_end_matches('/');
    let has_api_suffix = trimmed.ends_with("/api");
    let api_base = if trimmed.ends_with("/api") {
        trimmed.to_string()
    } else {
        format!("{}/api", trimmed)
    };

    if path.starts_with("/api/") {
        if has_api_suffix {
            format!("{}{}", trimmed, &path[4..])
        } else {
            format!("{}{}", trimmed, path)
        }
    } else {
        format!("{}{}", api_base, path)
    }
}

fn build_http_client(state: &AppState) -> Result<reqwest::Client, String> {
    let config = state.config.lock().map_err(|e| e.to_string())?.clone();
    let proxy_mode = config
        .proxy_mode
        .as_deref()
        .unwrap_or("system")
        .trim()
        .to_string();
    let proxy_host = config
        .proxy_host
        .as_deref()
        .unwrap_or_default()
        .trim()
        .to_string();
    let proxy_port = config
        .proxy_port
        .as_deref()
        .unwrap_or_default()
        .trim()
        .to_string();
    let cache_key = (proxy_mode.clone(), proxy_host.clone(), proxy_port.clone());

    if let Ok(cache) = state.http_client_cache.lock() {
        if let Some((cached_key, cached_client)) = cache.as_ref() {
            if cached_key == &cache_key {
                return Ok(cached_client.clone());
            }
        }
    }

    let mut builder = reqwest::Client::builder();

    match proxy_mode.as_str() {
        "none" => {
            builder = builder.no_proxy();
        }
        "manual" if !proxy_host.is_empty() && !proxy_port.is_empty() => {
            let proxy_url = if proxy_host.starts_with("http://")
                || proxy_host.starts_with("https://")
                || proxy_host.starts_with("socks5://")
            {
                format!("{}:{}", proxy_host.trim_end_matches('/'), proxy_port)
            } else {
                format!("http://{}:{}", proxy_host, proxy_port)
            };
            let proxy = reqwest::Proxy::all(&proxy_url).map_err(|e| e.to_string())?;
            builder = builder.proxy(proxy);
        }
        _ => {}
    }

    let client = builder.build().map_err(|e| e.to_string())?;
    if let Ok(mut cache) = state.http_client_cache.lock() {
        *cache = Some((cache_key, client.clone()));
    }
    Ok(client)
}

fn build_download_output_name(
    original_file_name: Option<&str>,
    remote_name: &str,
) -> String {
    let original_base = original_file_name
        .and_then(|s| Path::new(s).file_name().and_then(|n| n.to_str()))
        .unwrap_or("output");
    mvsep_api_tester::file_transfer::build_local_name(original_base, remote_name)
}

async fn parse_json_value(resp: reqwest::Response) -> Result<serde_json::Value, String> {
    let text = resp.text().await.map_err(|e| e.to_string())?;
    serde_json::from_str(&text).map_err(|e| {
        let preview = text.chars().take(120).collect::<String>();
        format!(
            "error decoding response body: {}; body preview: {}",
            e, preview
        )
    })
}

async fn get_json_with_fallback(
    client: &reqwest::Client,
    api_url: &str,
    token: &str,
    paths: &[&str],
    extra_query: Vec<(String, String)>,
) -> Result<serde_json::Value, String> {
    let mut attempts: Vec<String> = Vec::new();

    for path in paths {
        let url = build_api_url(api_url, path);
        let mut query_pairs = vec![("api_token".to_string(), token.to_string())];
        query_pairs.extend(extra_query.clone());
        let response = client
            .get(&url)
            .query(&query_pairs)
            .send()
            .await
            .map_err(|e| e.to_string())?;
        let status = response.status();
        if !status.is_success() {
            attempts.push(format!("{} -> {}", url, status));
            continue;
        }
        match parse_json_value(response).await {
            Ok(v) => return Ok(v),
            Err(err) => attempts.push(format!("{} -> {}", url, err)),
        }
    }

    Err(format!(
        "all candidate endpoints failed: {}",
        attempts.join(" | ")
    ))
}

fn parse_algorithms_from_value(value: &serde_json::Value) -> Vec<serde_json::Value> {
    if let Some(arr) = value.as_array() {
        if arr.first().and_then(|v| v.get("algorithms")).is_some() {
            let mut flattened = Vec::new();
            for group in arr {
                let gid = group.get("id").cloned().unwrap_or(serde_json::json!(0));
                let gname = group
                    .get("name")
                    .cloned()
                    .unwrap_or(serde_json::json!("Ungrouped"));
                if let Some(algos) = group.get("algorithms").and_then(|v| v.as_array()) {
                    for algo in algos {
                        let mut mapped = algo.clone();
                        if mapped.get("algorithm_group").is_none() {
                            mapped["algorithm_group"] = serde_json::json!({
                                "id": gid,
                                "name": gname
                            });
                        }
                        if mapped.get("render_id").is_none() {
                            mapped["render_id"] =
                                mapped.get("id").cloned().unwrap_or(serde_json::json!(0));
                        }
                        flattened.push(mapped);
                    }
                }
            }
            return flattened;
        }
        return arr.to_vec();
    }
    if let Some(arr) = value.get("data").and_then(|v| v.as_array()) {
        return arr.to_vec();
    }
    Vec::new()
}

fn parse_algorithm_details_from_value(algo: &serde_json::Value) -> AlgorithmDetails {
    let id = read_i32(algo.get("render_id")).unwrap_or(0);
    let name = algo
        .get("name")
        .and_then(|v| v.as_str())
        .unwrap_or("Unknown")
        .to_string();
    let mut fields: Vec<AlgorithmField> = Vec::new();

    if let Some(algorithm_fields) = algo.get("algorithm_fields").and_then(|f| f.as_array()) {
        for field in algorithm_fields {
            let field_name = field["name"].as_str().unwrap_or("");
            if field_name != "add_opt1" && field_name != "add_opt2" && field_name != "add_opt3" {
                continue;
            }

            let field_text = field["text"].as_str().unwrap_or("");
            let mut options: HashMap<String, String> = HashMap::new();
            if let Some(options_str) = field["options"].as_str() {
                if let Ok(opts) = serde_json::from_str::<serde_json::Value>(options_str) {
                    if let Some(obj) = opts.as_object() {
                        for (k, v) in obj {
                            if let Some(s) = v.as_str() {
                                options.insert(k.clone(), s.to_string());
                            } else {
                                options.insert(k.clone(), v.to_string());
                            }
                        }
                    }
                }
            }

            fields.push(AlgorithmField {
                name: field_name.to_string(),
                text: field_text.to_string(),
                options,
            });
        }
    }

    AlgorithmDetails { id, name, fields }
}

fn normalize_algorithm_groups_and_details(
    algorithms: Vec<serde_json::Value>,
) -> (Vec<AlgorithmGroupData>, BTreeMap<i32, AlgorithmDetails>) {
    let mut grouped: BTreeMap<i32, (String, Vec<AlgorithmItem>)> = BTreeMap::new();
    let mut details_by_id: BTreeMap<i32, AlgorithmDetails> = BTreeMap::new();

    for algo in algorithms {
        let id = read_i32(algo.get("render_id")).unwrap_or(0);
        let name = algo
            .get("name")
            .and_then(|v| v.as_str())
            .unwrap_or("Unknown")
            .to_string();
        let group_id = read_i32(algo.get("algorithm_group").and_then(|g| g.get("id"))).unwrap_or(0);
        let group_name = algo
            .get("algorithm_group")
            .and_then(|g| g.get("name"))
            .and_then(|v| v.as_str())
            .unwrap_or("Ungrouped")
            .to_string();

        grouped
            .entry(group_id)
            .or_insert_with(|| (group_name, Vec::new()))
            .1
            .push(AlgorithmItem { id, name, group_id });
        details_by_id.insert(id, parse_algorithm_details_from_value(&algo));
    }

    let groups: Vec<AlgorithmGroupData> = grouped
        .into_iter()
        .map(|(id, (name, algorithms))| AlgorithmGroupData {
            id,
            name,
            algorithms,
        })
        .collect();

    (groups, details_by_id)
}

async fn fetch_remote_algorithms_raw(
    state: &AppState,
    api_url: &str,
    _token: &str,
) -> Result<Vec<serde_json::Value>, String> {
    let client = build_http_client(state)?;
    let body = get_json_no_token(
        &client,
        api_url,
        &["/app/algorithms", "/algorithm_groups"],
        vec![("scopes".to_string(), "single_upload".to_string())],
    )
    .await?;
    Ok(parse_algorithms_from_value(&body))
}

async fn get_json_no_token(
    client: &reqwest::Client,
    api_url: &str,
    paths: &[&str],
    extra_query: Vec<(String, String)>,
) -> Result<serde_json::Value, String> {
    let mut attempts: Vec<String> = Vec::new();

    for path in paths {
        let url = build_api_url(api_url, path);
        let response = client
            .get(&url)
            .query(&extra_query)
            .send()
            .await
            .map_err(|e| e.to_string())?;
        let status = response.status();
        if !status.is_success() {
            attempts.push(format!("{} -> {}", url, status));
            continue;
        }
        match parse_json_value(response).await {
            Ok(v) => return Ok(v),
            Err(err) => attempts.push(format!("{} -> {}", url, err)),
        }
    }

    Err(format!(
        "all candidate endpoints failed: {}",
        attempts.join(" | ")
    ))
}

fn get_total_algorithms(groups: &[AlgorithmGroupData]) -> usize {
    groups.iter().map(|g| g.algorithms.len()).sum()
}

fn read_i64(value: Option<&serde_json::Value>) -> Option<i64> {
    if let Some(v) = value {
        if let Some(n) = v.as_i64() {
            return Some(n);
        }
        if let Some(s) = v.as_str() {
            return s.parse::<i64>().ok();
        }
    }
    None
}

fn read_i32(value: Option<&serde_json::Value>) -> Option<i32> {
    read_i64(value).map(|value| value as i32)
}

fn now_timestamp() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    match SystemTime::now().duration_since(UNIX_EPOCH) {
        Ok(duration) => format!("{}.{:03}", duration.as_secs(), duration.subsec_millis()),
        Err(_) => "0.000".to_string(),
    }
}

fn find_case_insensitive(haystack: &str, needle: &str) -> Option<usize> {
    haystack
        .to_ascii_lowercase()
        .find(&needle.to_ascii_lowercase())
}

fn redact_after_marker(input: &str, marker: &str, terminators: &[char]) -> String {
    let mut redacted = String::with_capacity(input.len());
    let mut rest = input;

    while let Some(pos) = find_case_insensitive(rest, marker) {
        let value_start = pos + marker.len();
        redacted.push_str(&rest[..value_start]);

        let value = &rest[value_start..];
        let value_end = value
            .find(|ch| terminators.contains(&ch))
            .unwrap_or(value.len());
        if value_end > 0 {
            redacted.push_str("[REDACTED]");
        }
        rest = &value[value_end..];
    }

    redacted.push_str(rest);
    redacted
}

fn redact_sensitive(input: &str) -> String {
    let quoted_markers = [
        "\"api_token\":\"",
        "\"token\":\"",
        "\"access_token\":\"",
        "\"refresh_token\":\"",
        "\"authorization\":\"",
        "\"password\":\"",
        "\"secret\":\"",
    ];
    let unquoted_markers = [
        "api_token=",
        "token=",
        "access_token=",
        "refresh_token=",
        "authorization=",
        "password=",
        "secret=",
        "api_token:",
        "token:",
        "access_token:",
        "refresh_token:",
        "authorization:",
        "password:",
        "secret:",
        "bearer ",
    ];

    let mut redacted = input.to_string();
    for marker in quoted_markers {
        redacted = redact_after_marker(&redacted, marker, &['"']);
    }
    for marker in unquoted_markers {
        redacted = redact_after_marker(
            &redacted,
            marker,
            &[' ', '\t', '\n', '\r', '&', ',', ')', ']', '}', '"', '\''],
        );
    }
    redacted
}

fn to_tauri_result<T>(state: &AppState, result: BackendResult<T>) -> Result<T, String> {
    result.map_err(|error| {
        push_backend_log(state, "ERROR", error.to_log_message());
        error.into_tauri_error()
    })
}

fn push_backend_log(state: &AppState, level: &str, message: impl Into<String>) {
    if let Ok(mut logs) = state.backend_logs.lock() {
        logs.push(LogEntry {
            timestamp: now_timestamp(),
            level: level.to_string(),
            message: redact_sensitive(&message.into()),
        });
        if logs.len() > 2000 {
            let drain = logs.len() - 2000;
            logs.drain(0..drain);
        }
    }
}

#[tauri::command]
fn load_config(state: State<'_, AppState>) -> Result<Config, String> {
    let app = state.inner();
    to_tauri_result(app, app.backend.load_config(app))
}

#[tauri::command]
fn save_config(state: State<'_, AppState>, config: Config) -> Result<(), String> {
    let app = state.inner();
    to_tauri_result(app, app.backend.save_config(app, config))
}

#[tauri::command]
fn resolve_path(state: State<'_, AppState>, path: String) -> Result<String, String> {
    let app = state.inner();
    to_tauri_result(app, app.backend.resolve_path(app, path))
}

#[tauri::command]
fn get_algorithm_cache_path_cmd(state: State<'_, AppState>) -> Result<String, String> {
    let app = state.inner();
    to_tauri_result(app, app.backend.get_algorithm_cache_path_cmd(app))
}

#[tauri::command]
async fn open_in_file_manager(state: State<'_, AppState>, path: String) -> Result<(), String> {
    let app = state.inner();
    to_tauri_result(app, app.backend.open_in_file_manager(app, path).await)
}

#[tauri::command]
async fn test_connection(
    state: State<'_, AppState>,
    token: String,
    api_url: String,
) -> Result<bool, String> {
    let app = state.inner();
    to_tauri_result(app, app.backend.test_connection(app, token, api_url).await)
}

#[tauri::command]
async fn fetch_latest_algorithm_info(
    state: State<'_, AppState>,
    api_url: String,
    token: String,
    proxy_mode: Option<String>,
    proxy_host: Option<String>,
    proxy_port: Option<String>,
) -> Result<FetchLatestAlgorithmInfoResult, String> {
    let app = state.inner();
    to_tauri_result(
        app,
        app.backend
            .fetch_latest_algorithm_info(app, api_url, token, proxy_mode, proxy_host, proxy_port)
            .await,
    )
}

#[tauri::command]
fn refresh_algorithm_list_from_local(
    state: State<'_, AppState>,
) -> Result<LocalAlgorithmListResponse, String> {
    let app = state.inner();
    to_tauri_result(app, app.backend.refresh_algorithm_list_from_local(app))
}

#[tauri::command]
fn get_algorithm_details_from_local(
    state: State<'_, AppState>,
    algorithm_id: i32,
) -> Result<AlgorithmDetails, String> {
    let app = state.inner();
    to_tauri_result(
        app,
        app.backend
            .get_algorithm_details_from_local(app, algorithm_id),
    )
}

#[tauri::command]
async fn list_algorithms(
    state: State<'_, AppState>,
    keywords: Option<String>,
    _group_id: Option<i32>,
    algorithm_id: Option<i32>,
    _recursive: Option<bool>,
    api_url: String,
    token: String,
) -> Result<serde_json::Value, String> {
    let app = state.inner();
    to_tauri_result(
        app,
        app.backend
            .list_algorithms(
                app,
                keywords,
                _group_id,
                algorithm_id,
                _recursive,
                api_url,
                token,
            )
            .await,
    )
}

#[tauri::command]
async fn get_algorithm_details(
    state: State<'_, AppState>,
    algorithm_id: i32,
    api_url: String,
    token: String,
) -> Result<AlgorithmDetails, String> {
    let app = state.inner();
    to_tauri_result(
        app,
        app.backend
            .get_algorithm_details(app, algorithm_id, api_url, token)
            .await,
    )
}

#[tauri::command]
async fn list_formats(
    state: State<'_, AppState>,
    api_url: String,
    token: String,
) -> Result<Vec<OutputFormat>, String> {
    let app = state.inner();
    to_tauri_result(app, app.backend.list_formats(app, api_url, token).await)
}

#[tauri::command]
async fn query_name(
    state: State<'_, AppState>,
    algorithm_id: i32,
    model_id: Option<String>,
    api_url: String,
    token: String,
) -> Result<String, String> {
    let app = state.inner();
    to_tauri_result(
        app,
        app.backend
            .query_name(app, algorithm_id, model_id, api_url, token)
            .await,
    )
}

#[tauri::command]
fn cancel_download(state: State<'_, AppState>, hash: String) -> Result<(), String> {
    let app = state.inner();
    to_tauri_result(app, app.backend.cancel_download(app, hash))
}

#[tauri::command]
async fn get_queue_info(
    state: State<'_, AppState>,
    api_url: String,
    token: String,
) -> Result<QueueStatus, String> {
    let app = state.inner();
    to_tauri_result(app, app.backend.get_queue_info(app, api_url, token).await)
}

#[tauri::command]
async fn get_remote_history(
    state: State<'_, AppState>,
    limit: Option<i32>,
    api_url: String,
    token: String,
) -> Result<serde_json::Value, String> {
    let app = state.inner();
    to_tauri_result(
        app,
        app.backend
            .get_remote_history(app, limit, api_url, token)
            .await,
    )
}

#[tauri::command]
#[allow(clippy::too_many_arguments)]
async fn create_task(
    state: State<'_, AppState>,
    window: tauri::Window,
    file_path: String,
    sep_type: i32,
    options: std::collections::HashMap<String, Option<i32>>,
    output_format: Option<i32>,
    demo: bool,
    api_url: String,
    token: String,
) -> Result<String, String> {
    let app = state.inner();
    to_tauri_result(
        app,
        app.backend
            .create_task(
                app,
                window,
                file_path,
                sep_type,
                options,
                output_format,
                demo,
                api_url,
                token,
            )
            .await,
    )
}

#[tauri::command]
async fn get_task_status(
    state: State<'_, AppState>,
    hash: String,
    api_url: String,
    token: String,
) -> Result<TaskStatus, String> {
    let app = state.inner();
    to_tauri_result(
        app,
        app.backend.get_task_status(app, hash, api_url, token).await,
    )
}

#[tauri::command]
#[allow(clippy::too_many_arguments)]
async fn download_result(
    state: State<'_, AppState>,
    window: tauri::Window,
    hash: String,
    output_dir: String,
    file_index: Option<i32>,
    original_file_name: Option<String>,
    api_url: String,
    token: String,
) -> Result<Vec<String>, String> {
    let app = state.inner();
    to_tauri_result(
        app,
        app.backend
            .download_result(
                app,
                window,
                hash,
                output_dir,
                file_index,
                original_file_name,
                api_url,
                token,
            )
            .await,
    )
}

#[tauri::command]
fn get_tasks(state: State<'_, AppState>) -> Result<Vec<TaskInfo>, String> {
    let app = state.inner();
    to_tauri_result(app, app.backend.get_tasks(app))
}

#[tauri::command]
fn add_task(state: State<'_, AppState>, task: TaskInfo) -> Result<(), String> {
    let app = state.inner();
    to_tauri_result(app, app.backend.add_task(app, task))
}

#[tauri::command]
fn replace_active_tasks(state: State<'_, AppState>, tasks: Vec<TaskInfo>) -> Result<(), String> {
    let app = state.inner();
    to_tauri_result(app, app.backend.replace_active_tasks(app, tasks))
}

#[tauri::command]
fn update_task_status(
    state: State<'_, AppState>,
    hash: String,
    status: String,
    progress: f32,
    files: Option<Vec<String>>,
    error: Option<String>,
) -> Result<(), String> {
    let app = state.inner();
    to_tauri_result(
        app,
        app.backend
            .update_task_status(app, hash, status, progress, files, error),
    )
}

#[tauri::command]
fn remove_task(state: State<'_, AppState>, hash: String) -> Result<(), String> {
    let app = state.inner();
    to_tauri_result(app, app.backend.remove_task(app, hash))
}

#[tauri::command]
fn get_task_history(state: State<'_, AppState>) -> Result<Vec<TaskHistoryRecord>, String> {
    let app = state.inner();
    to_tauri_result(app, app.backend.get_task_history(app))
}

#[tauri::command]
fn save_task_history(
    state: State<'_, AppState>,
    records: Vec<TaskHistoryRecord>,
) -> Result<(), String> {
    let app = state.inner();
    to_tauri_result(app, app.backend.save_task_history(app, records))
}

#[tauri::command]
fn complete_task(
    state: State<'_, AppState>,
    task: TaskInfo,
    record: TaskHistoryRecord,
) -> Result<(), String> {
    let app = state.inner();
    to_tauri_result(app, app.backend.complete_task(app, task, record))
}

#[tauri::command]
fn get_backend_logs(state: State<'_, AppState>) -> Result<Vec<LogEntry>, String> {
    let app = state.inner();
    to_tauri_result(app, app.backend.get_backend_logs(app))
}

#[tauri::command]
fn frontend_debug_log(state: State<'_, AppState>, level: String, message: String) {
    let app = state.inner();
    app.backend.frontend_debug_log(app, level, message);
}

#[tauri::command]
fn web_storage_get(state: State<'_, AppState>, key: String) -> Result<Option<String>, String> {
    let paths = &state.inner().paths;
    let db = web_db::open_web_database(&paths.web_db_path).map_err(|e| e.to_string())?;
    db.get_string(&key).map_err(|e| e.to_string())
}

#[tauri::command]
fn web_storage_set(state: State<'_, AppState>, key: String, value: String) -> Result<(), String> {
    let paths = &state.inner().paths;
    let db = web_db::open_web_database(&paths.web_db_path).map_err(|e| e.to_string())?;
    db.set_string(&key, &value).map_err(|e| e.to_string())
}

#[tauri::command]
fn web_storage_delete(state: State<'_, AppState>, key: String) -> Result<bool, String> {
    let paths = &state.inner().paths;
    let db = web_db::open_web_database(&paths.web_db_path).map_err(|e| e.to_string())?;
    db.delete(&key).map_err(|e| e.to_string())
}

#[tauri::command]
fn web_storage_exists(state: State<'_, AppState>, key: String) -> Result<bool, String> {
    let paths = &state.inner().paths;
    let db = web_db::open_web_database(&paths.web_db_path).map_err(|e| e.to_string())?;
    db.exists(&key).map_err(|e| e.to_string())
}

#[tauri::command]
fn web_storage_get_all(state: State<'_, AppState>) -> Result<Vec<web_db::WebConfigEntry>, String> {
    let paths = &state.inner().paths;
    let db = web_db::open_web_database(&paths.web_db_path).map_err(|e| e.to_string())?;
    db.get_all().map_err(|e| e.to_string())
}

#[tauri::command]
fn web_storage_clear_all(state: State<'_, AppState>) -> Result<u64, String> {
    let paths = &state.inner().paths;
    let db = web_db::open_web_database(&paths.web_db_path).map_err(|e| e.to_string())?;
    db.clear_all().map_err(|e| e.to_string())
}

fn legacy_resolve_path(state: &AppState, path: String) -> Result<String, String> {
    let absolute = resolve_backend_path(&state.paths, PathBuf::from(path));
    Ok(absolute.to_string_lossy().to_string())
}

async fn legacy_open_in_file_manager(state: &AppState, path: String) -> Result<(), String> {
    let input = PathBuf::from(path);
    let mut target = resolve_backend_path(&state.paths, input);

    if !target.exists() {
        if let Some(parent) = target.parent() {
            if parent.exists() {
                target = parent.to_path_buf();
            }
        }
    }
    push_backend_log(
        state,
        "INFO",
        format!("open_in_file_manager start: {}", target.to_string_lossy()),
    );
    eprintln!(
        "[backend:INFO] open_in_file_manager start: {}",
        target.to_string_lossy()
    );

    #[cfg(target_os = "linux")]
    {
        use ashpd::desktop::open_uri::OpenDirectoryRequest;
        use std::fs::File;
        use std::os::fd::AsFd;

        let target_s = target.to_string_lossy().to_string();
        let mut attempts: Vec<String> = Vec::new();

        match File::open(&target) {
            Ok(dir_file) => {
                match OpenDirectoryRequest::default()
                    .send(&dir_file.as_fd())
                    .await
                {
                    Ok(_) => {
                        push_backend_log(
                            state,
                            "INFO",
                            format!(
                                "open_in_file_manager success via xdg-desktop-portal {}",
                                target_s
                            ),
                        );
                        eprintln!(
                            "[backend:INFO] open_in_file_manager success via xdg-desktop-portal {}",
                            target_s
                        );
                        return Ok(());
                    }
                    Err(e) => {
                        attempts.push(format!("xdg-desktop-portal open_directory -> {}", e));
                    }
                }
            }
            Err(e) => {
                attempts.push(format!("open directory fd failed {} -> {}", target_s, e));
            }
        }

        let candidates: Vec<(&str, Vec<String>)> = vec![
            ("xdg-open", vec![target_s.clone()]),
            ("gio", vec!["open".to_string(), target_s.clone()]),
            ("kioclient5", vec!["exec".to_string(), target_s.clone()]),
            ("nautilus", vec![target_s.clone()]),
            ("thunar", vec![target_s.clone()]),
            ("dolphin", vec![target_s.clone()]),
        ];

        for (bin, args) in candidates {
            let output = Command::new(bin).args(&args).output();
            match output {
                Ok(out) if out.status.success() => {
                    push_backend_log(
                        state,
                        "INFO",
                        format!("open_in_file_manager success via {} {:?}", bin, args),
                    );
                    eprintln!(
                        "[backend:INFO] open_in_file_manager success via {} {:?}",
                        bin, args
                    );
                    return Ok(());
                }
                Ok(out) => attempts.push(format!(
                    "{} {:?} -> exit {} stderr={}",
                    bin,
                    args,
                    out.status.code().unwrap_or(-1),
                    String::from_utf8_lossy(&out.stderr)
                )),
                Err(e) => attempts.push(format!("{} {:?} -> {}", bin, args, e)),
            }
        }

        let error = format!(
            "failed to open file manager for {}. attempts: {}",
            target.to_string_lossy(),
            attempts.join(" | ")
        );
        push_backend_log(state, "ERROR", error.clone());
        eprintln!("[backend:ERROR] {}", error);
        return Err(error);
    }

    #[cfg(target_os = "windows")]
    {
        let target_s = target.to_string_lossy().to_string();
        let mut cmd = Command::new("explorer");
        if target.is_file() {
            cmd.arg(format!("/select,{}", target_s));
        } else {
            cmd.arg(target_s);
        }
        let status = cmd.status().map_err(|e| e.to_string())?;
        if status.success() {
            push_backend_log(state, "INFO", "open_in_file_manager success via explorer");
            eprintln!("[backend:INFO] open_in_file_manager success via explorer");
            return Ok(());
        }
        push_backend_log(state, "ERROR", "open_in_file_manager failed via explorer");
        eprintln!("[backend:ERROR] open_in_file_manager failed via explorer");
        return Err("failed to open with explorer".to_string());
    }

    #[cfg(target_os = "macos")]
    {
        let target_s = target.to_string_lossy().to_string();
        let status = Command::new("open")
            .arg(target_s)
            .status()
            .map_err(|e| e.to_string())?;
        if status.success() {
            push_backend_log(state, "INFO", "open_in_file_manager success via open");
            eprintln!("[backend:INFO] open_in_file_manager success via open");
            return Ok(());
        }
        push_backend_log(state, "ERROR", "open_in_file_manager failed via open");
        eprintln!("[backend:ERROR] open_in_file_manager failed via open");
        return Err("failed to open with open".to_string());
    }

    #[allow(unreachable_code)]
    Err("unsupported platform".to_string())
}

// ============== API 相关 ==============

async fn legacy_test_connection(
    state: &AppState,
    token: String,
    api_url: String,
) -> Result<bool, String> {
    push_backend_log(state, "INFO", format!("test_connection start: {}", api_url));
    let client = build_http_client(state)?;
    let result = get_json_with_fallback(
        &client,
        &api_url,
        &token,
        &["/app/algorithms", "/algorithm_groups"],
        vec![("scopes".to_string(), "single_upload".to_string())],
    )
    .await;
    let ok = result.is_ok();
    push_backend_log(
        state,
        if ok { "INFO" } else { "ERROR" },
        format!("test_connection result: {}", ok),
    );
    Ok(ok)
}

async fn legacy_fetch_latest_algorithm_info(
    state: &AppState,
    api_url: String,
    token: String,
    proxy_mode: Option<String>,
    proxy_host: Option<String>,
    proxy_port: Option<String>,
) -> Result<FetchLatestAlgorithmInfoResult, String> {
    push_backend_log(state, "INFO", "fetch_latest_algorithm_info start");
    eprintln!("[backend:INFO] fetch_latest_algorithm_info start");
    let _ = (proxy_mode, proxy_host, proxy_port);

    eprintln!(
        "[backend:INFO] fetch_latest_algorithm_info remote fetch start: {}",
        api_url
    );
    let algorithms = fetch_remote_algorithms_raw(state, &api_url, &token)
        .await
        .map_err(|e| {
            push_backend_log(
                state,
                "ERROR",
                format!("fetch_latest_algorithm_info remote fetch failed: {}", e),
            );
            eprintln!(
                "[backend:ERROR] fetch_latest_algorithm_info remote fetch failed: {}",
                e
            );
            e
        })?;
    replace_algorithm_cache_in_backend_store(state, &algorithms).map_err(|e| {
        push_backend_log(
            state,
            "ERROR",
            format!("fetch_latest_algorithm_info save db cache failed: {}", e),
        );
        e
    })?;
    let updated_at = now_timestamp();
    save_algorithm_cache_updated_at(state, &updated_at).map_err(|e| {
        push_backend_log(
            state,
            "ERROR",
            format!(
                "fetch_latest_algorithm_info save cache metadata failed: {}",
                e
            ),
        );
        e
    })?;

    let local = load_algorithm_list_from_backend_store(state).map_err(|e| {
        push_backend_log(
            state,
            "ERROR",
            format!("fetch_latest_algorithm_info reload db cache failed: {}", e),
        );
        e
    })?;
    push_backend_log(
        state,
        "INFO",
        format!(
            "fetch_latest_algorithm_info success: groups={}, algorithms={}",
            local.groups.len(),
            local.total_algorithms
        ),
    );
    eprintln!(
        "[backend:INFO] fetch_latest_algorithm_info success: groups={}, algorithms={}",
        local.groups.len(),
        local.total_algorithms
    );
    Ok(FetchLatestAlgorithmInfoResult {
        updated_at,
        total_groups: local.groups.len(),
        total_algorithms: local.total_algorithms,
        cli_exit_code: 0,
    })
}

async fn legacy_list_algorithms(
    state: &AppState,
    keywords: Option<String>,
    _group_id: Option<i32>,
    algorithm_id: Option<i32>,
    _recursive: Option<bool>,
    api_url: String,
    token: String,
) -> Result<serde_json::Value, String> {
    push_backend_log(
        state,
        "INFO",
        format!(
            "list_algorithms start: keyword={}",
            keywords.clone().unwrap_or_default()
        ),
    );
    let mut algorithms = fetch_remote_algorithms_raw(state, &api_url, &token)
        .await
        .map_err(|e| {
            push_backend_log(state, "ERROR", format!("list_algorithms failed: {}", e));
            e
        })?;

    if let Some(kw) = keywords {
        let kw_lower = kw.to_lowercase();
        algorithms.retain(|algo| {
            algo.get("name")
                .and_then(|v| v.as_str())
                .map(|name| name.to_lowercase().contains(&kw_lower))
                .unwrap_or(false)
        });
        let results: Vec<serde_json::Value> = algorithms
            .iter()
            .map(|algo| {
                serde_json::json!({
                    "id": read_i32(algo.get("render_id")).unwrap_or(0),
                    "name": algo.get("name").and_then(|v| v.as_str()).unwrap_or("Unknown"),
                    "group_id": read_i32(algo.get("algorithm_group").and_then(|g| g.get("id"))).unwrap_or(0)
                })
            })
            .collect();
        push_backend_log(
            state,
            "INFO",
            format!("list_algorithms search result: {}", results.len()),
        );
        return Ok(serde_json::json!({ "algorithms": results }));
    }

    if let Some(alg_id) = algorithm_id {
        if let Some(found) = algorithms
            .into_iter()
            .find(|algo| read_i32(algo.get("render_id")) == Some(alg_id))
        {
            push_backend_log(
                state,
                "INFO",
                format!("list_algorithms single algorithm: {}", alg_id),
            );
            return Ok(serde_json::json!({
                "id": read_i32(found.get("render_id")).unwrap_or(0),
                "name": found.get("name").and_then(|v| v.as_str()).unwrap_or("Unknown"),
                "group_id": read_i32(found.get("algorithm_group").and_then(|g| g.get("id"))).unwrap_or(0)
            }));
        }
        push_backend_log(
            state,
            "ERROR",
            format!("list_algorithms algorithm not found: {}", alg_id),
        );
        return Err("Algorithm not found".to_string());
    }

    let (groups, _) = normalize_algorithm_groups_and_details(algorithms);
    push_backend_log(
        state,
        "INFO",
        format!("list_algorithms grouped result: {} groups", groups.len()),
    );
    serde_json::to_value(groups).map_err(|e| e.to_string())
}
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AlgorithmField {
    #[serde(default)]
    pub name: String,
    #[serde(default)]
    pub text: String,
    #[serde(default)]
    pub options: std::collections::HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AlgorithmDetails {
    #[serde(default)]
    pub id: i32,
    #[serde(default)]
    pub name: String,
    #[serde(default)]
    pub fields: Vec<AlgorithmField>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct LocalAlgorithmListResponse {
    pub updated_at: String,
    pub groups: Vec<AlgorithmGroupData>,
    pub total_algorithms: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct FetchLatestAlgorithmInfoResult {
    pub updated_at: String,
    pub total_groups: usize,
    pub total_algorithms: usize,
    pub cli_exit_code: i32,
}

async fn legacy_get_algorithm_details(
    state: &AppState,
    algorithm_id: i32,
    api_url: String,
    token: String,
) -> Result<AlgorithmDetails, String> {
    push_backend_log(
        state,
        "INFO",
        format!("get_algorithm_details start: {}", algorithm_id),
    );
    let algorithms = fetch_remote_algorithms_raw(state, &api_url, &token)
        .await
        .map_err(|e| {
            push_backend_log(
                state,
                "ERROR",
                format!("get_algorithm_details failed: {}", e),
            );
            e
        })?;

    for algo in algorithms {
        if read_i32(algo.get("render_id")) == Some(algorithm_id) {
            let details = parse_algorithm_details_from_value(&algo);
            push_backend_log(
                state,
                "INFO",
                format!(
                    "get_algorithm_details ok: {}, {} fields",
                    algorithm_id,
                    details.fields.len()
                ),
            );
            return Ok(details);
        }
    }

    push_backend_log(
        state,
        "ERROR",
        format!("get_algorithm_details not found: {}", algorithm_id),
    );
    Err("Algorithm not found".to_string())
}

async fn legacy_query_name(
    state: &AppState,
    algorithm_id: i32,
    model_id: Option<String>,
    api_url: String,
    token: String,
) -> Result<String, String> {
    let client = build_http_client(state)?;
    let body = get_json_with_fallback(
        &client,
        &api_url,
        &token,
        &["/app/algorithms", "/algorithm_groups"],
        vec![("scopes".to_string(), "single_upload".to_string())],
    )
    .await?;
    let algorithms = parse_algorithms_from_value(&body);

    for algo in algorithms {
        if algo["render_id"].as_i64().map(|v| v as i32) == Some(algorithm_id) {
            if let Some(model) = model_id {
                let target = model.parse::<i32>().unwrap_or(-1);
                if let Some(fields) = algo["algorithm_fields"].as_array() {
                    for field in fields {
                        let options_str = field["options"].as_str().unwrap_or("{}");
                        if let Ok(opts) = serde_json::from_str::<serde_json::Value>(options_str) {
                            if let Some(obj) = opts.as_object() {
                                for (k, v) in obj {
                                    if k.parse::<i32>().unwrap_or(-1) == target {
                                        return Ok(v.as_str().unwrap_or("").to_string());
                                    }
                                }
                            }
                        }
                    }
                }
                return Err(format!("Model '{}' not found", model));
            }
            return Ok(algo["name"].as_str().unwrap_or("Unknown").to_string());
        }
    }

    Err("Algorithm not found".to_string())
}

fn register_download_token(
    state: &AppState,
    hash: &str,
    token: Arc<AtomicBool>,
) -> Result<(), String> {
    let mut guard = state
        .download_cancellations
        .lock()
        .map_err(|e| e.to_string())?;
    if guard.contains_key(hash) {
        return Err(format!("Download already in progress for task {}", hash));
    }
    guard.insert(hash.to_string(), token);
    Ok(())
}

fn unregister_download_token(state: &AppState, hash: &str, token: &Arc<AtomicBool>) {
    if let Ok(mut map) = state.download_cancellations.lock() {
        if map
            .get(hash)
            .is_some_and(|current| Arc::ptr_eq(current, token))
        {
            map.remove(hash);
        }
    }
}

fn legacy_cancel_download(state: &AppState, hash: String) -> Result<(), String> {
    let map = state
        .download_cancellations
        .lock()
        .map_err(|e| e.to_string())?;
    if let Some(token) = map.get(&hash) {
        token.store(true, Ordering::SeqCst);
    }
    Ok(())
}

async fn legacy_get_queue_info(
    state: &AppState,
    api_url: String,
    token: String,
) -> Result<QueueStatus, String> {
    let client = build_http_client(state)?;
    let info = get_json_with_fallback(&client, &api_url, &token, &["/app/queue", "/queue"], vec![])
        .await
        .unwrap_or_else(|_| serde_json::json!({}));
    let active = info
        .get("active")
        .or_else(|| info.get("active_count"))
        .and_then(|v| read_i32(Some(v)))
        .unwrap_or(0);
    let queued = info
        .get("queued")
        .or_else(|| info.get("queue_count"))
        .and_then(|v| read_i32(Some(v)))
        .unwrap_or(0);

    if let Ok(mut last) = state.last_queue_info.lock() {
        let current = (active, queued);
        if last.as_ref() != Some(&current) {
            *last = Some(current);
            push_backend_log(
                state,
                "INFO",
                format!("get_queue_info active={}, queued={}", active, queued),
            );
        }
    }
    Ok(QueueStatus { active, queued })
}

async fn legacy_get_remote_history(
    state: &AppState,
    limit: Option<i32>,
    api_url: String,
    token: String,
) -> Result<serde_json::Value, String> {
    push_backend_log(
        state,
        "INFO",
        format!("get_remote_history start: limit={}", limit.unwrap_or(20)),
    );
    let client = build_http_client(state)?;
    let url = build_api_url(&api_url, "/app/separation_history");

    let response = client
        .get(&url)
        .query(&[
            ("api_token", token),
            ("start", "0".to_string()),
            ("limit", limit.unwrap_or(20).to_string()),
        ])
        .send()
        .await
        .map_err(|e| {
            let msg = e.to_string();
            push_backend_log(
                state,
                "ERROR",
                format!("get_remote_history request failed: {}", msg),
            );
            msg
        })?;

    response.json().await.map_err(|e| {
        let msg = e.to_string();
        push_backend_log(
            state,
            "ERROR",
            format!("get_remote_history decode failed: {}", msg),
        );
        msg
    })
}

#[allow(clippy::too_many_arguments)]
async fn legacy_create_task(
    state: &AppState,
    window: tauri::Window,
    file_path: String,
    sep_type: i32,
    options: std::collections::HashMap<String, Option<i32>>,
    output_format: Option<i32>,
    demo: bool,
    api_url: String,
    token: String,
) -> BackendResult<String> {
    push_backend_log(state, "INFO", format!("create_task start: {}", file_path));
    eprintln!("[backend:INFO] create_task start: {}", file_path);
    let url = build_api_url(&api_url, "/separation/create");
    let client = build_http_client(state).map_err(|e| {
        BackendError::legacy("create_task", e)
            .with_endpoint(url.clone())
            .with_path(file_path.clone())
    })?;

    let demo_str = if demo { "1" } else { "0" };
    let mut fields = vec![
        ("sep_type".to_string(), sep_type.to_string()),
        (
            "output_format".to_string(),
            output_format.unwrap_or(1).to_string(),
        ),
        ("is_demo".to_string(), demo_str.to_string()),
        ("api_token".to_string(), token.clone()),
    ];

    for (name, value) in options {
        if let Some(v) = value {
            fields.push((name, v.to_string()));
        }
    }

    let file_path_buf = PathBuf::from(&file_path);
    let last_progress: Arc<Mutex<Option<file_transfer::TransferProgress>>> =
        Arc::new(Mutex::new(None));
    let last_progress_for_emit = last_progress.clone();
    let window_for_progress = window.clone();
    let hash = file_transfer::upload_file_async(
        &client,
        &url,
        &file_path_buf,
        fields,
        None,
        move |progress| {
            if let Ok(mut last) = last_progress_for_emit.lock() {
                *last = Some(progress.clone());
            }
            let payload = UploadProgressPayload {
                file_name: progress.file_name,
                uploaded_bytes: progress.bytes,
                total_bytes: progress.total_bytes.unwrap_or(0),
                speed_bps: progress.speed_bps,
                percent: progress.percent,
                done: progress.done,
                failed: progress.failed,
            };
            let _ = window_for_progress.emit("upload-progress", payload);
        },
    )
    .await
    .map_err(|e| {
        if let Ok(last) = last_progress.lock() {
            let progress = last.clone();
            let file_name = progress
                .as_ref()
                .map(|p| p.file_name.clone())
                .or_else(|| {
                    file_path_buf
                        .file_name()
                        .and_then(|n| n.to_str())
                        .map(|s| s.to_string())
                })
                .unwrap_or_else(|| "audio.wav".to_string());
            let payload = UploadProgressPayload {
                file_name,
                uploaded_bytes: progress.as_ref().map(|p| p.bytes).unwrap_or(0),
                total_bytes: progress.as_ref().and_then(|p| p.total_bytes).unwrap_or(0),
                speed_bps: progress.as_ref().map(|p| p.speed_bps).unwrap_or(0.0),
                percent: progress.as_ref().map(|p| p.percent).unwrap_or(0.0),
                done: true,
                failed: true,
            };
            let _ = window.emit("upload-progress", payload);
        }
        let backend_error =
            transfer_backend_error("create_task", e, &url, None, Some(file_path_buf.clone()));
        eprintln!(
            "[backend:ERROR] create_task upload failed: {}",
            redact_sensitive(&backend_error.to_log_message())
        );
        backend_error
    })?;

    push_backend_log(state, "INFO", format!("create_task success: hash={}", hash));
    eprintln!("[backend:INFO] create_task success: hash={}", hash);
    Ok(hash)
}

async fn legacy_get_task_status(
    state: &AppState,
    hash: String,
    api_url: String,
    token: String,
) -> Result<TaskStatus, String> {
    let client = build_http_client(state)?;
    let url = build_api_url(&api_url, "/separation/get");

    let response = client
        .get(&url)
        .query(&[
            ("hash".to_string(), hash.clone()),
            ("api_token".to_string(), token),
        ])
        .send()
        .await
        .map_err(|e| {
            let msg = e.to_string();
            push_backend_log(
                state,
                "ERROR",
                format!("get_task_status request failed: hash={}, {}", hash, msg),
            );
            msg
        })?;

    let body = parse_json_value(response).await?;

    let status = body
        .get("status")
        .and_then(|v| v.as_str())
        .unwrap_or("unknown")
        .to_string();

    let message = body
        .get("data")
        .and_then(|d| d.get("message"))
        .and_then(|v| v.as_str())
        .map(|s| s.to_string())
        .or_else(|| {
            body.get("message")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string())
        });

    let finished = body
        .get("data")
        .and_then(|d| d.get("finished_chunks"))
        .and_then(|v| read_i32(Some(v)))
        .unwrap_or(0);
    let total = body
        .get("data")
        .and_then(|d| d.get("all_chunks"))
        .and_then(|v| read_i32(Some(v)))
        .unwrap_or(0);
    let progress = if total > 0 {
        (finished as f32 / total as f32) * 100.0
    } else if status == "done" {
        100.0
    } else {
        0.0
    };

    let files = body
        .get("data")
        .and_then(|d| d.get("files"))
        .and_then(|v| v.as_array())
        .map(|arr| {
            arr.iter()
                .filter_map(|v| {
                    v.get("name")
                        .and_then(|n| n.as_str())
                        .map(|s| s.to_string())
                        .or_else(|| {
                            v.get("url")
                                .and_then(|u| u.as_str())
                                .and_then(|u| u.split('/').next_back().map(|s| s.to_string()))
                        })
                })
                .collect()
        });

    let queue_count = body
        .get("data")
        .and_then(|d| d.get("queue_count"))
        .and_then(|v| read_i32(Some(v)));

    let current_order = body
        .get("data")
        .and_then(|d| d.get("current_order"))
        .and_then(|v| read_i32(Some(v)));

    let result = TaskStatus {
        status,
        progress,
        message,
        files,
        queue_count,
        current_order,
    };
    push_backend_log(
        state,
        "INFO",
        format!(
            "get_task_status hash={} status={} progress={:.2}",
            hash, result.status, result.progress
        ),
    );
    Ok(result)
}

#[allow(clippy::too_many_arguments)]
async fn legacy_download_result(
    state: &AppState,
    window: tauri::Window,
    hash: String,
    output_dir: String,
    file_index: Option<i32>,
    original_file_name: Option<String>,
    api_url: String,
    token: String,
) -> BackendResult<Vec<String>> {
    push_backend_log(
        state,
        "INFO",
        format!("download_result start: hash={}", hash),
    );
    eprintln!("[backend:INFO] download_result start: hash={}", hash);
    let cancel_token = Arc::new(AtomicBool::new(false));
    register_download_token(state, &hash, cancel_token.clone())
        .map_err(|e| BackendError::legacy("download_result", e).with_hash(hash.clone()))?;
    let download_job = async {
        let url = build_api_url(&api_url, "/separation/get");
        let client = build_http_client(state).map_err(|e| {
            BackendError::legacy("download_result", e)
                .with_endpoint(url.clone())
                .with_hash(hash.clone())
                .with_path(output_dir.clone())
        })?;
        let response = client
            .get(&url)
            .query(&[
                ("hash".to_string(), hash.clone()),
                ("api_token".to_string(), token.clone()),
            ])
            .send()
            .await
            .map_err(|e| {
                let msg = e.to_string();
                let err = BackendError::legacy("download_result", msg)
                    .with_endpoint(url.clone())
                    .with_hash(hash.clone())
                    .with_path(output_dir.clone());
                eprintln!(
                    "[backend:ERROR] download_result query failed: {}",
                    redact_sensitive(&err.to_log_message())
                );
                err
            })?;
        let status = response.status();
        if !status.is_success() {
            return Err(BackendError::legacy(
                "download_result",
                format!("HTTP {} while querying task files", status),
            )
            .with_endpoint(url.clone())
            .with_hash(hash.clone())
            .with_path(output_dir.clone())
            .with_http_status(status.as_u16()));
        }
        let body = parse_json_value(response).await.map_err(|e| {
            BackendError::legacy("download_result", e)
                .with_endpoint(url.clone())
                .with_hash(hash.clone())
                .with_path(output_dir.clone())
        })?;
        let files = body
            .get("data")
            .and_then(|d| d.get("files"))
            .and_then(|v| v.as_array())
            .ok_or_else(|| {
                BackendError::legacy("download_result", "No files to download")
                    .with_endpoint(url.clone())
                    .with_hash(hash.clone())
                    .with_path(output_dir.clone())
            })?;

        let mut downloaded: Vec<String> = Vec::new();
        let output_dir_path = PathBuf::from(&output_dir);
        let normalized_output_dir = resolve_backend_path(&state.paths, output_dir_path);

        fs::create_dir_all(&normalized_output_dir).map_err(|e| {
            BackendError::legacy("download_result", e.to_string())
                .with_endpoint(url.clone())
                .with_hash(hash.clone())
                .with_path(normalized_output_dir.to_string_lossy().into_owned())
        })?;
        let original_ref = original_file_name.as_deref();

        for (i, file_info) in files.iter().enumerate() {
            if let Some(idx) = file_index {
                if i != idx as usize {
                    continue;
                }
            }

            if cancel_token.load(Ordering::SeqCst) {
                push_backend_log(
                    state,
                    "WARN",
                    format!("download_result cancelled for hash={}", hash),
                );
                eprintln!("[backend:WARN] download_result cancelled for hash={}", hash);
                return Err(
                    BackendError::legacy("download_result", "Download cancelled")
                        .with_endpoint(url.clone())
                        .with_hash(hash.clone())
                        .with_path(output_dir.clone()),
                );
            }

            let file_url = file_info
                .get("url")
                .and_then(|v| v.as_str())
                .ok_or_else(|| {
                    BackendError::legacy("download_result", "File URL missing")
                        .with_endpoint(url.clone())
                        .with_hash(hash.clone())
                        .with_path(output_dir.clone())
                })?;
            let file_name = file_info
                .get("remote_name")
                .and_then(|v| v.as_str())
                .or_else(|| file_url.split('/').next_back())
                .unwrap_or("output.bin");
            let local_file_name = build_download_output_name(original_ref, file_name);
            let output_path = normalized_output_dir.join(&local_file_name);

            if output_path.exists() {
                push_backend_log(
                    state,
                    "INFO",
                    format!("download_result file already exists, skipping: {}", local_file_name),
                );
                downloaded.push(output_path.to_string_lossy().to_string());
                continue;
            }

            let resume_from = file_transfer::get_resume_info(&output_path, file_url);
            if resume_from > 0 {
                push_backend_log(
                    state,
                    "INFO",
                    format!(
                        "download_result resume attempt: hash={}, file={}, offset={}",
                        hash, local_file_name, resume_from
                    ),
                );
            }

            let hash_for_progress = hash.clone();
            let window_for_progress = window.clone();
            file_transfer::download_file_async(
                &client,
                file_url,
                &output_path,
                file_name,
                Some(cancel_token.clone()),
                move |progress| {
                    let payload = DownloadProgressPayload {
                        hash: hash_for_progress.clone(),
                        file_name: progress.file_name,
                        downloaded_bytes: progress.bytes,
                        total_bytes: progress.total_bytes,
                        speed_bps: progress.speed_bps,
                        percent: progress.percent,
                        done: progress.done,
                    };
                    let _ = window_for_progress.emit("download-progress", payload);
                },
            )
            .await
            .map_err(|e| {
                let is_cancelled = e.is_cancelled();
                let err = transfer_backend_error(
                    "download_result",
                    e,
                    &url,
                    Some(hash.clone()),
                    Some(output_path.clone()),
                );
                if is_cancelled {
                    push_backend_log(
                        state,
                        "WARN",
                        format!("download_result cancelled mid-stream for hash={}", hash),
                    );
                    eprintln!(
                        "[backend:WARN] download_result cancelled mid-stream for hash={}",
                        hash
                    );
                } else {
                    eprintln!(
                        "[backend:ERROR] download_result stream failed: {}",
                        redact_sensitive(&err.to_log_message())
                    );
                }
                err
            })?;

            downloaded.push(output_path.to_string_lossy().to_string());
        }

        push_backend_log(
            state,
            "INFO",
            format!("download_result success: {} files", downloaded.len()),
        );
        eprintln!(
            "[backend:INFO] download_result success: {} files",
            downloaded.len()
        );
        Ok(downloaded)
    };

    let result = download_job.await;
    unregister_download_token(state, &hash, &cancel_token);
    result
}
fn legacy_get_backend_logs(state: &AppState) -> Result<Vec<LogEntry>, String> {
    let logs = state.backend_logs.lock().map_err(|e| e.to_string())?;
    Ok(logs.clone())
}

fn legacy_frontend_debug_log(state: &AppState, level: String, message: String) {
    let normalized = match level.to_uppercase().as_str() {
        "ERROR" => "ERROR",
        "WARN" => "WARN",
        "DEBUG" => "DEBUG",
        _ => "INFO",
    };
    let safe_message = redact_sensitive(&message);
    let line = format!("[frontend] {}", safe_message);
    push_backend_log(state, normalized, line.clone());
    eprintln!("[frontend:{}] {}", normalized, safe_message);
}

// ============== 主程序 ==============

#[cfg(test)]
fn new_app_state() -> AppState {
    new_app_state_with_paths(BackendPaths::fallback())
}

fn new_app_state_with_paths(paths: BackendPaths) -> AppState {
    AppState {
        backend: LegacyMainBackend,
        paths,
        config: Mutex::new(Config::default()),
        tasks: Mutex::new(HashMap::new()),
        backend_logs: Mutex::new(Vec::new()),
        last_queue_info: Mutex::new(None),
        download_cancellations: Mutex::new(HashMap::new()),
        http_client_cache: Mutex::new(None),
    }
}

fn backend_paths_from_app<R: tauri::Runtime>(app: &tauri::App<R>) -> Result<BackendPaths, String> {
    let app_config_dir = app.path().app_config_dir().map_err(|e| e.to_string())?;
    let app_data_dir = app.path().app_data_dir().map_err(|e| e.to_string())?;
    Ok(BackendPaths::new(
        app_config_dir,
        app_data_dir,
        get_config_path(),
    ))
}

fn main() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_shell::init())
        .plugin(
            tauri_plugin_log::Builder::new()
                .level(log::LevelFilter::Info)
                .level_for("reqwest", log::LevelFilter::Info)
                .level_for("reqwest::connect", log::LevelFilter::Info)
                .level_for("reqwest::retry", log::LevelFilter::Info)
                .level_for("hyper", log::LevelFilter::Info)
                .level_for("hyper_util", log::LevelFilter::Info)
                .target(tauri_plugin_log::Target::new(
                    tauri_plugin_log::TargetKind::LogDir {
                        file_name: Some("mvsep".into()),
                    },
                ))
                .build(),
        )
        .setup(|app| {
            let paths = backend_paths_from_app(app).map_err(std::io::Error::other)?;
            app.manage(new_app_state_with_paths(paths));
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            load_config,
            save_config,
            resolve_path,
            get_algorithm_cache_path_cmd,
            open_in_file_manager,
            test_connection,
            fetch_latest_algorithm_info,
            refresh_algorithm_list_from_local,
            get_algorithm_details_from_local,
            list_algorithms,
            query_name,
            get_algorithm_details,
            list_formats,
            get_queue_info,
            get_remote_history,
            create_task,
            get_task_status,
            download_result,
            cancel_download,
            get_tasks,
            add_task,
            replace_active_tasks,
            update_task_status,
            remove_task,
            get_task_history,
            save_task_history,
            complete_task,
            get_backend_logs,
            frontend_debug_log,
            web_storage_get,
            web_storage_set,
            web_storage_delete,
            web_storage_exists,
            web_storage_get_all,
            web_storage_clear_all,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_task(hash: &str) -> TaskInfo {
        TaskInfo {
            hash: hash.to_string(),
            file_name: "song.wav".to_string(),
            algorithm_id: 49,
            algorithm_name: "Test algorithm".to_string(),
            model_id: Some(7),
            model_name: Some("Test model".to_string()),
            model2_id: Some(8),
            model2_name: Some("Secondary model".to_string()),
            model3_id: Some(9),
            model3_name: Some("Tertiary model".to_string()),
            format: 1,
            status: "queued".to_string(),
            progress: 0.0,
            created_at: 1_725_000_000,
            output_files: Vec::new(),
            error: None,
            message: Some("Waiting in queue".to_string()),
            queue_count: Some(3),
            current_order: Some(2),
            phase: Some("queueing".to_string()),
            download_file_name: None,
            download_bytes: Some(0),
            download_total_bytes: None,
            download_speed_bps: Some(0.0),
            download_percent: Some(0.0),
        }
    }

    fn sample_history_record(id: &str) -> TaskHistoryRecord {
        TaskHistoryRecord {
            id: id.to_string(),
            file_name: "song.wav".to_string(),
            algorithm_id: 49,
            algorithm_name: "Test algorithm".to_string(),
            model_id: Some(7),
            model_name: Some("Test model".to_string()),
            model2_id: Some(8),
            model2_name: Some("Secondary model".to_string()),
            model3_id: Some(9),
            model3_name: Some("Tertiary model".to_string()),
            format_id: 1,
            format_name: "WAV".to_string(),
            status: "done".to_string(),
            created_at: 1_725_000_000,
            completed_at: Some(1_725_000_100),
            output_files: vec!["song_vocals.wav".to_string()],
            output_path: Some("/tmp/output".to_string()),
            error: None,
        }
    }

    #[test]
    fn app_backend_task_facade_persists_tasks_in_backend_store() {
        let (root, paths) = temp_backend_paths("task-facade");
        let state = new_app_state_with_paths(paths);
        let backend = state.backend;
        let task = sample_task("hash-1");

        backend.add_task(&state, task).unwrap();
        backend
            .update_task_status(
                &state,
                "hash-1".to_string(),
                "done".to_string(),
                100.0,
                Some(vec!["song_vocals.wav".to_string()]),
                Some("kept for parity".to_string()),
            )
            .unwrap();

        let tasks = backend.get_tasks(&state).unwrap();
        assert_eq!(tasks.len(), 1);
        let updated = &tasks[0];
        assert_eq!(updated.hash, "hash-1");
        assert_eq!(updated.status, "done");
        assert_eq!(updated.progress, 100.0);
        assert_eq!(updated.output_files, vec!["song_vocals.wav"]);
        assert_eq!(updated.error.as_deref(), Some("kept for parity"));
        assert_eq!(updated.model2_id, Some(8));
        assert_eq!(updated.model3_id, Some(9));

        backend.remove_task(&state, "hash-1".to_string()).unwrap();
        assert!(backend.get_tasks(&state).unwrap().is_empty());
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn task_store_restores_active_tasks_from_tasks_db_after_restart() {
        let (root, paths) = temp_backend_paths("task-restart");
        let state = new_app_state_with_paths(paths.clone());
        state
            .backend
            .replace_active_tasks(&state, vec![sample_task("hash-restore")])
            .unwrap();
        drop(state);

        let restarted = new_app_state_with_paths(paths.clone());
        let tasks = restarted.backend.get_tasks(&restarted).unwrap();
        assert_eq!(tasks.len(), 1);
        let restored = &tasks[0];
        assert_eq!(restored.hash, "hash-restore");
        assert_eq!(restored.status, "queued");
        assert_eq!(restored.model2_id, Some(8));
        assert_eq!(restored.model3_id, Some(9));
        assert_eq!(restored.message.as_deref(), Some("Waiting in queue"));
        assert_eq!(restored.queue_count, Some(3));
        assert_eq!(restored.phase.as_deref(), Some("queueing"));
        assert!(paths.tasks_db_path.exists());
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn task_history_upsert_keeps_one_record_per_hash() {
        let (root, paths) = temp_backend_paths("task-history-upsert");
        let state = new_app_state_with_paths(paths);
        let first = sample_history_record("hash-history");
        let mut downloaded = first.clone();
        downloaded.completed_at = Some(1_725_000_200);
        downloaded.output_files = vec![
            "/tmp/output/song_vocals.wav".to_string(),
            "/tmp/output/song_inst.wav".to_string(),
        ];
        downloaded.output_path = Some("/tmp/output".to_string());

        state
            .backend
            .save_task_history(&state, vec![first, downloaded])
            .unwrap();

        let history = state.backend.get_task_history(&state).unwrap();
        assert_eq!(history.len(), 1);
        assert_eq!(history[0].id, "hash-history");
        assert_eq!(history[0].completed_at, Some(1_725_000_200));
        assert_eq!(history[0].output_files.len(), 2);
        assert_eq!(history[0].output_path.as_deref(), Some("/tmp/output"));
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn complete_task_atomically_moves_active_task_to_history() {
        let (root, paths) = temp_backend_paths("task-complete");
        let state = new_app_state_with_paths(paths);
        let mut task = sample_task("hash-complete");
        state.backend.add_task(&state, task.clone()).unwrap();
        task.status = "done".to_string();
        task.phase = Some("done".to_string());
        task.output_files = vec!["/tmp/output/song_vocals.wav".to_string()];
        let mut record = sample_history_record("hash-complete");
        record.output_files = task.output_files.clone();
        record.output_path = Some("/tmp/output".to_string());

        state.backend.complete_task(&state, task, record).unwrap();

        assert!(state.backend.get_tasks(&state).unwrap().is_empty());
        let history = state.backend.get_task_history(&state).unwrap();
        assert_eq!(history.len(), 1);
        assert_eq!(history[0].id, "hash-complete");
        assert_eq!(
            history[0].output_files,
            vec!["/tmp/output/song_vocals.wav".to_string()]
        );
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn retry_history_roundtrip_preserves_algorithm_options_and_format() {
        let (root, paths) = temp_backend_paths("task-history-retry");
        let state = new_app_state_with_paths(paths);
        state
            .backend
            .save_task_history(&state, vec![sample_history_record("hash-retry")])
            .unwrap();
        let restarted = new_app_state_with_paths(state.paths.clone());
        let history = restarted.backend.get_task_history(&restarted).unwrap();
        assert_eq!(history.len(), 1);
        let record = &history[0];
        assert_eq!(record.algorithm_id, 49);
        assert_eq!(record.model_id, Some(7));
        assert_eq!(record.model2_id, Some(8));
        assert_eq!(record.model3_id, Some(9));
        assert_eq!(record.format_id, 1);
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn task_output_files_adapter_accepts_richer_backend_json_rows() {
        let (root, paths) = temp_backend_paths("task-output-files-json");
        let state = new_app_state_with_paths(paths);
        let db = open_tasks_database(&state.paths).unwrap();
        let conn = db.conn.lock().unwrap();

        let mut task_row = task_info_to_row(&sample_task("hash-object-files")).unwrap();
        task_row.output_files = serde_json::json!([
            {
                "remote_name": "vocals.wav",
                "url": "https://cdn.example.test/vocals.wav",
                "downloaded": false,
                "local_path": null
            },
            {
                "remote_name": "inst.wav",
                "url": "https://cdn.example.test/inst.wav",
                "downloaded": true,
                "local_path": "/tmp/output/inst.wav"
            }
        ])
        .to_string();
        repositories::insert_task(&conn, &task_row).unwrap();

        let mut history_row =
            task_history_to_row(&sample_history_record("hash-object-files")).unwrap();
        history_row.output_files = serde_json::json!([
            {
                "remote_name": "vocals.wav",
                "url": "https://cdn.example.test/vocals.wav",
                "downloaded": true,
                "local_path": "/tmp/output/vocals.wav"
            }
        ])
        .to_string();
        repositories::insert_task_history(&conn, &history_row).unwrap();
        drop(conn);

        let tasks = state.backend.get_tasks(&state).unwrap();
        assert_eq!(
            tasks[0].output_files,
            vec![
                "https://cdn.example.test/vocals.wav".to_string(),
                "/tmp/output/inst.wav".to_string()
            ]
        );

        let history = state.backend.get_task_history(&state).unwrap();
        assert_eq!(
            history[0].output_files,
            vec!["/tmp/output/vocals.wav".to_string()]
        );
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn progress_event_payload_field_names_stay_stable() {
        let upload = serde_json::to_value(UploadProgressPayload {
            file_name: "song.wav".to_string(),
            uploaded_bytes: 10,
            total_bytes: 20,
            speed_bps: 5.0,
            percent: 50.0,
            done: false,
            failed: false,
        })
        .unwrap();
        assert!(upload.get("file_name").is_some());
        assert!(upload.get("uploaded_bytes").is_some());
        assert!(upload.get("total_bytes").is_some());
        assert!(upload.get("speed_bps").is_some());

        let download = serde_json::to_value(DownloadProgressPayload {
            hash: "hash-1".to_string(),
            file_name: "song_vocals.wav".to_string(),
            downloaded_bytes: 10,
            total_bytes: Some(20),
            speed_bps: 5.0,
            percent: 50.0,
            done: false,
        })
        .unwrap();
        assert!(download.get("hash").is_some());
        assert!(download.get("file_name").is_some());
        assert!(download.get("downloaded_bytes").is_some());
        assert!(download.get("total_bytes").is_some());
        assert!(download.get("speed_bps").is_some());
    }

    #[test]
    fn backend_error_keeps_context_until_tauri_edge() {
        let state = new_app_state();
        let err = BackendError::legacy("download_result", "network failed")
            .with_endpoint("https://mvsep.com/api/separation/get")
            .with_hash("hash-1")
            .with_path("/tmp/output");

        assert_eq!(err.context.operation, "download_result");
        assert_eq!(
            err.context.endpoint.as_deref(),
            Some("https://mvsep.com/api/separation/get")
        );
        assert_eq!(err.context.hash.as_deref(), Some("hash-1"));
        assert_eq!(err.context.path.as_deref(), Some("/tmp/output"));
        assert_eq!(err.clone().into_tauri_error(), "network failed");
        let tauri_result: Result<(), String> = to_tauri_result(&state, Err(err));
        assert_eq!(tauri_result, Err("network failed".to_string()));
        let logs = state.backend_logs.lock().unwrap();
        assert!(logs[0].message.contains("operation=download_result"));
        assert!(logs[0].message.contains("path=/tmp/output"));
    }

    #[test]
    fn tauri_error_payload_redacts_transfer_tokens() {
        let state = new_app_state();
        let err = BackendError::legacy(
            "download_result",
            "request failed api_token=secret-token&x=1",
        )
        .with_endpoint("https://mvsep.com/api/separation/get?api_token=secret-token")
        .with_hash("hash-1")
        .with_path("/tmp/output");

        let tauri_result: Result<(), String> = to_tauri_result(&state, Err(err));
        let message = tauri_result.unwrap_err();
        assert!(!message.contains("secret-token"));
        assert!(message.contains("[REDACTED]"));

        let logs = state.backend_logs.lock().unwrap();
        assert!(!logs[0].message.contains("secret-token"));
        assert!(logs[0].message.contains("[REDACTED]"));
    }

    #[test]
    fn transfer_backend_error_preserves_status_url_hash_and_path() {
        let err = file_transfer::TransferError::new("HTTP 403 while downloading file")
            .with_url("https://cdn.example.test/file.wav")
            .with_path("/tmp/song_vocals.wav")
            .with_http_status(403);

        let backend_error = transfer_backend_error(
            "download_result",
            err,
            "https://mvsep.com/api/separation/get",
            Some("hash-1".to_string()),
            Some(PathBuf::from("/tmp/fallback")),
        );

        assert_eq!(backend_error.context.operation, "download_result");
        assert_eq!(
            backend_error.context.endpoint.as_deref(),
            Some("https://cdn.example.test/file.wav")
        );
        assert_eq!(backend_error.context.hash.as_deref(), Some("hash-1"));
        assert_eq!(
            backend_error.context.path.as_deref(),
            Some("/tmp/song_vocals.wav")
        );
        assert_eq!(backend_error.context.http_status, Some(403));
    }

    #[test]
    fn download_cancellation_registry_rejects_duplicate_hashes() {
        let state = new_app_state();
        let first = Arc::new(AtomicBool::new(false));
        let second = Arc::new(AtomicBool::new(false));

        register_download_token(&state, "hash-1", first.clone()).unwrap();
        let err = register_download_token(&state, "hash-1", second.clone()).unwrap_err();
        assert!(err.contains("Download already in progress"));

        legacy_cancel_download(&state, "hash-1".to_string()).unwrap();
        assert!(first.load(Ordering::SeqCst));
        assert!(!second.load(Ordering::SeqCst));

        unregister_download_token(&state, "hash-1", &second);
        assert!(state
            .download_cancellations
            .lock()
            .unwrap()
            .contains_key("hash-1"));

        unregister_download_token(&state, "hash-1", &first);
        assert!(!state
            .download_cancellations
            .lock()
            .unwrap()
            .contains_key("hash-1"));
    }

    #[test]
    fn backend_logs_redact_token_like_values() {
        let state = new_app_state();
        push_backend_log(
            &state,
            "ERROR",
            r#"request failed api_token=secret-1&x=1 {"token":"secret-2"} Bearer secret-3"#,
        );

        let logs = state.backend_logs.lock().unwrap();
        let message = &logs[0].message;
        assert!(!message.contains("secret-1"));
        assert!(!message.contains("secret-2"));
        assert!(!message.contains("secret-3"));
        assert!(message.contains("[REDACTED]"));
    }

    fn temp_backend_paths(name: &str) -> (PathBuf, BackendPaths) {
        let nanos = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let root = std::env::temp_dir().join(format!(
            "mvsep-rs-{}-{}-{}",
            name,
            std::process::id(),
            nanos
        ));
        let paths = BackendPaths::new(
            root.join("app-config"),
            root.join("app-data"),
            root.join("legacy").join("config.json"),
        );
        (root, paths)
    }

    #[test]
    fn relative_paths_resolve_under_injected_app_data_dir() {
        let (root, paths) = temp_backend_paths("path-resolution");
        let state = new_app_state_with_paths(paths.clone());

        let resolved = state
            .backend
            .resolve_path(&state, "./output".to_string())
            .unwrap();
        assert_eq!(PathBuf::from(resolved), paths.app_data_dir.join("./output"));

        let absolute = root.join("absolute-output");
        let resolved_absolute = state
            .backend
            .resolve_path(&state, absolute.to_string_lossy().into_owned())
            .unwrap();
        assert_eq!(PathBuf::from(resolved_absolute), absolute);
        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn download_output_relative_paths_use_backend_app_data_dir() {
        let (root, paths) = temp_backend_paths("download-output-path");
        let relative = PathBuf::from("./output");
        let resolved = resolve_backend_path(&paths, &relative);
        assert_eq!(resolved, paths.app_data_dir.join("./output"));
        assert!(!resolved.starts_with(std::env::current_dir().unwrap()));
        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn config_store_imports_legacy_json_once() {
        let (root, paths) = temp_backend_paths("config-import");
        fs::create_dir_all(paths.legacy_config_json_path.parent().unwrap()).unwrap();

        let legacy_config = Config {
            token: Some("legacy-token".to_string()),
            proxy_host: Some("10.0.0.1".to_string()),
            ..Config::default()
        };
        fs::write(
            &paths.legacy_config_json_path,
            serde_json::to_string_pretty(&legacy_config).unwrap(),
        )
        .unwrap();

        let state = new_app_state_with_paths(paths.clone());
        let imported = state.backend.load_config(&state).unwrap();
        assert_eq!(imported.token.as_deref(), Some("legacy-token"));
        assert_eq!(imported.proxy_host.as_deref(), Some("10.0.0.1"));
        assert!(paths.user_config_db_path.exists());
        assert!(!paths.mvsep_db_path.exists());

        let mut saved = imported.clone();
        saved.token = Some("db-token".to_string());
        state.backend.save_config(&state, saved).unwrap();

        let mut stale_legacy_config = legacy_config;
        stale_legacy_config.token = Some("stale-legacy-token".to_string());
        fs::write(
            &paths.legacy_config_json_path,
            serde_json::to_string_pretty(&stale_legacy_config).unwrap(),
        )
        .unwrap();

        let reloaded = state.backend.load_config(&state).unwrap();
        assert_eq!(reloaded.token.as_deref(), Some("db-token"));
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn config_store_merges_partial_saves_with_existing_and_defaults() {
        let (root, paths) = temp_backend_paths("config-partial");
        let state = new_app_state_with_paths(paths);

        state
            .backend
            .save_config(
                &state,
                Config {
                    token: Some("first-token".to_string()),
                    proxy_host: Some("10.0.0.2".to_string()),
                    ..Config::default()
                },
            )
            .unwrap();

        state
            .backend
            .save_config(
                &state,
                Config {
                    token: Some("partial-token".to_string()),
                    api_url: None,
                    mirror: None,
                    proxy_mode: None,
                    proxy_host: None,
                    proxy_port: None,
                    output_dir: None,
                    output_format: None,
                    poll_interval: None,
                    algorithm_auto_refresh_days: None,
                },
            )
            .unwrap();

        let reloaded = state.backend.load_config(&state).unwrap();
        assert_eq!(reloaded.token.as_deref(), Some("partial-token"));
        assert_eq!(reloaded.proxy_host.as_deref(), Some("10.0.0.2"));
        assert_eq!(reloaded.api_url.as_deref(), Some("https://mvsep.com"));
        assert_eq!(reloaded.output_dir.as_deref(), Some("./output"));
        assert_eq!(reloaded.output_format, Some(1));
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn legacy_config_import_error_logs_legacy_path() {
        let (root, paths) = temp_backend_paths("legacy-config-error");
        fs::create_dir_all(paths.legacy_config_json_path.parent().unwrap()).unwrap();
        fs::write(&paths.legacy_config_json_path, "{not-json").unwrap();

        let state = new_app_state_with_paths(paths.clone());
        let result = to_tauri_result(&state, state.backend.load_config(&state));
        assert!(result.is_err());

        let logs = state.backend_logs.lock().unwrap();
        let message = &logs[0].message;
        assert!(message.contains("operation=load_config"));
        assert!(message.contains(&format!(
            "path={}",
            paths.legacy_config_json_path.to_string_lossy()
        )));
        assert!(!message.contains(&format!("path={}", paths.mvsep_db_path.to_string_lossy())));
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn output_formats_store_preserves_frontend_shape() {
        let (root, paths) = temp_backend_paths("formats-shape");
        let state = new_app_state_with_paths(paths);

        let formats = load_output_formats_from_backend_store(&state).unwrap();
        assert_eq!(formats.len(), 6);
        assert_eq!(formats[0].id, 0);
        assert_eq!(formats[0].name, "MP3 (320 kbps)");

        let serialized = serde_json::to_value(&formats[0]).unwrap();
        assert!(serialized.get("id").is_some());
        assert!(serialized.get("name").is_some());
        assert!(serialized.get("extension").is_none());
        assert!(serialized.get("bits_per_sample").is_none());
        assert!(serialized.get("is_premium").is_none());
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn algorithm_cache_missing_db_returns_empty_list() {
        let (root, paths) = temp_backend_paths("algorithm-cache-empty");
        let state = new_app_state_with_paths(paths.clone());

        let result = state
            .backend
            .refresh_algorithm_list_from_local(&state)
            .unwrap();

        assert_eq!(result.updated_at, "");
        assert_eq!(result.total_algorithms, 0);
        assert!(result.groups.is_empty());
        assert!(paths.mvsep_db_path.exists());
        assert_eq!(
            state.backend.get_algorithm_cache_path_cmd(&state).unwrap(),
            paths.mvsep_db_path.to_string_lossy()
        );
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn algorithm_cache_db_store_preserves_frontend_payload_shape() {
        let (root, paths) = temp_backend_paths("algorithm-cache-shape");
        let state = new_app_state_with_paths(paths);
        let raw = serde_json::json!([
            {
                "id": 7,
                "name": "Vocals",
                "algorithms": [
                    {
                        "id": 26,
                        "name": "Ensemble",
                        "price_coefficient": 1.5,
                        "orientation": 2,
                        "algorithm_fields": [
                            {
                                "name": "add_opt1",
                                "text": "Output files",
                                "options": "{\"0\":\"Standard\",\"1\":2}",
                                "default_key": "0"
                            },
                            {
                                "name": "unsupported",
                                "text": "Hidden",
                                "options": "{\"x\":\"Hidden\"}"
                            },
                            {
                                "name": "add_opt2",
                                "text": "Model",
                                "options": { "7": "HQ" },
                                "default_key": "7"
                            }
                        ]
                    }
                ]
            }
        ]);
        let algorithms = parse_algorithms_from_value(&raw);
        replace_algorithm_cache_in_backend_store(&state, &algorithms).unwrap();
        save_algorithm_cache_updated_at(&state, "123.456").unwrap();

        let list = state
            .backend
            .refresh_algorithm_list_from_local(&state)
            .unwrap();
        assert_eq!(list.updated_at, "123.456");
        assert_eq!(list.total_algorithms, 1);
        assert_eq!(list.groups[0].id, 7);
        assert_eq!(list.groups[0].name, "Vocals");
        assert_eq!(list.groups[0].algorithms[0].id, 26);
        assert_eq!(list.groups[0].algorithms[0].group_id, 7);

        let serialized_algorithm = serde_json::to_value(&list.groups[0].algorithms[0]).unwrap();
        assert!(serialized_algorithm.get("id").is_some());
        assert!(serialized_algorithm.get("name").is_some());
        assert!(serialized_algorithm.get("group_id").is_some());
        assert!(serialized_algorithm.get("price_coefficient").is_none());
        assert!(serialized_algorithm.get("orientation").is_none());

        let details = state
            .backend
            .get_algorithm_details_from_local(&state, 26)
            .unwrap();
        assert_eq!(details.id, 26);
        assert_eq!(details.name, "Ensemble");
        assert_eq!(details.fields.len(), 2);
        assert_eq!(details.fields[0].name, "add_opt1");
        assert_eq!(
            details.fields[0].options.get("0").map(String::as_str),
            Some("Standard")
        );
        assert_eq!(
            details.fields[0].options.get("1").map(String::as_str),
            Some("2")
        );
        assert_eq!(details.fields[1].name, "add_opt2");
        assert_eq!(
            details.fields[1].options.get("7").map(String::as_str),
            Some("HQ")
        );
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn algorithm_cache_db_refresh_replaces_stale_algorithms() {
        let (root, paths) = temp_backend_paths("algorithm-cache-replace");
        let state = new_app_state_with_paths(paths);
        let first = parse_algorithms_from_value(&serde_json::json!([
            {
                "render_id": 1,
                "name": "Old",
                "algorithm_group": { "id": 1, "name": "Old Group" },
                "algorithm_fields": []
            }
        ]));
        replace_algorithm_cache_in_backend_store(&state, &first).unwrap();

        let second = parse_algorithms_from_value(&serde_json::json!([
            {
                "render_id": 2,
                "name": "New",
                "algorithm_group": { "id": 2, "name": "New Group" },
                "algorithm_fields": []
            }
        ]));
        replace_algorithm_cache_in_backend_store(&state, &second).unwrap();

        let list = state
            .backend
            .refresh_algorithm_list_from_local(&state)
            .unwrap();
        assert_eq!(list.total_algorithms, 1);
        assert_eq!(list.groups[0].id, 2);
        assert_eq!(list.groups[0].algorithms[0].id, 2);
        assert!(state
            .backend
            .get_algorithm_details_from_local(&state, 1)
            .is_err());
        assert_eq!(
            state
                .backend
                .get_algorithm_details_from_local(&state, 2)
                .unwrap()
                .name,
            "New"
        );
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn algorithm_cache_field_ids_are_scoped_per_algorithm() {
        let (root, paths) = temp_backend_paths("algorithm-cache-field-ids");
        let state = new_app_state_with_paths(paths);
        let raw = serde_json::json!([
            {
                "render_id": 10,
                "name": "Algo A",
                "algorithm_group": { "id": 1, "name": "Group" },
                "algorithm_fields": [
                    {
                        "id": 1,
                        "name": "add_opt1",
                        "text": "A Field",
                        "options": "{\"a\":\"A\"}"
                    }
                ]
            },
            {
                "render_id": 20,
                "name": "Algo B",
                "algorithm_group": { "id": 1, "name": "Group" },
                "algorithm_fields": [
                    {
                        "id": 1,
                        "name": "add_opt1",
                        "text": "B Field",
                        "options": "{\"b\":\"B\"}"
                    }
                ]
            }
        ]);
        let algorithms = parse_algorithms_from_value(&raw);
        replace_algorithm_cache_in_backend_store(&state, &algorithms).unwrap();

        let a = state
            .backend
            .get_algorithm_details_from_local(&state, 10)
            .unwrap();
        let b = state
            .backend
            .get_algorithm_details_from_local(&state, 20)
            .unwrap();
        assert_eq!(a.fields.len(), 1);
        assert_eq!(b.fields.len(), 1);
        assert_eq!(a.fields[0].text, "A Field");
        assert_eq!(b.fields[0].text, "B Field");
        assert_eq!(a.fields[0].options.get("a").map(String::as_str), Some("A"));
        assert_eq!(b.fields[0].options.get("b").map(String::as_str), Some("B"));
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn algorithm_cache_metadata_error_logs_user_config_path() {
        let (root, paths) = temp_backend_paths("algorithm-cache-metadata-error");
        fs::create_dir_all(&paths.user_config_db_path).unwrap();
        let state = new_app_state_with_paths(paths.clone());

        let result: Result<LocalAlgorithmListResponse, String> = to_tauri_result(
            &state,
            state.backend.refresh_algorithm_list_from_local(&state),
        );
        assert!(result.is_err());

        let logs = state.backend_logs.lock().unwrap();
        let message = &logs[0].message;
        assert!(message.contains("operation=refresh_algorithm_list_from_local"));
        assert!(message.contains(&format!(
            "path={}",
            paths.user_config_db_path.to_string_lossy()
        )));
        assert!(!message.contains(&format!("path={}", paths.mvsep_db_path.to_string_lossy())));
        fs::remove_dir_all(root).unwrap();
    }
}
