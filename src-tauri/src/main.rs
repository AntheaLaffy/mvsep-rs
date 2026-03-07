// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use futures_util::StreamExt;
use futures_util::TryStreamExt;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, HashMap};
use std::fs;
use std::path::{Path, PathBuf};
#[cfg(unix)]
use std::os::unix::process::ExitStatusExt;
#[cfg(windows)]
use std::os::windows::process::ExitStatusExt;
use std::process::{Command, ExitStatus, Output};
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::{Arc, Mutex};
use std::time::Instant;
use tauri::{Emitter, State};
use tokio::io::AsyncWriteExt;
use tokio_util::io::ReaderStream;

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

pub struct AppState {
    pub config: Mutex<Config>,
    pub tasks: Mutex<HashMap<String, TaskInfo>>,
    pub backend_logs: Mutex<Vec<LogEntry>>,
    pub last_queue_info: Mutex<Option<(i32, i32)>>,
    pub download_cancellations: Mutex<HashMap<String, Arc<AtomicBool>>>,
    pub http_client_cache: Mutex<Option<((String, String, String), reqwest::Client)>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskInfo {
    pub hash: String,
    pub file_name: String,
    pub algorithm_id: i32,
    pub algorithm_name: String,
    pub model_id: Option<i32>,
    pub model_name: Option<String>,
    pub format: i32,
    pub status: String,
    pub progress: f32,
    pub created_at: i64,
    pub output_files: Vec<String>,
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

#[derive(Debug, Clone, Serialize, Deserialize)]
struct PartialDownloadMeta {
    file_url: String,
    remote_file_name: String,
    updated_at: String,
}

fn get_config_path() -> PathBuf {
    dirs::config_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("mvsep-gui")
        .join("config.json")
}

fn ensure_config_dir() -> std::io::Result<()> {
    let config_path = get_config_path();
    let config_dir = config_path
        .parent()
        .ok_or_else(|| std::io::Error::new(std::io::ErrorKind::Other, "invalid config path"))?
        .to_path_buf();
    fs::create_dir_all(config_dir)?;
    Ok(())
}

fn ensure_parent_dir(path: &Path) -> std::io::Result<()> {
    if let Some(parent) = path.parent() {
        if !parent.as_os_str().is_empty() {
            fs::create_dir_all(parent)?;
        }
    }
    Ok(())
}

fn get_algorithm_cache_path() -> PathBuf {
    get_config_path()
        .parent()
        .map(|p| p.join("algorithms_cache.json"))
        .unwrap_or_else(|| PathBuf::from("./algorithms_cache.json"))
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

fn build_http_client(state: &State<'_, AppState>) -> Result<reqwest::Client, String> {
    let config = state.config.lock().map_err(|e| e.to_string())?.clone();
    let proxy_mode = config.proxy_mode.as_deref().unwrap_or("system").trim().to_string();
    let proxy_host = config.proxy_host.as_deref().unwrap_or_default().trim().to_string();
    let proxy_port = config.proxy_port.as_deref().unwrap_or_default().trim().to_string();
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
        "manual" => {
            if !proxy_host.is_empty() && !proxy_port.is_empty() {
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
        }
        _ => {}
    }

    let client = builder.build().map_err(|e| e.to_string())?;
    if let Ok(mut cache) = state.http_client_cache.lock() {
        *cache = Some((cache_key, client.clone()));
    }
    Ok(client)
}

fn sanitize_file_component(value: &str) -> String {
    let mut out = String::with_capacity(value.len());
    for ch in value.chars() {
        let bad = matches!(ch, '<' | '>' | ':' | '"' | '/' | '\\' | '|' | '?' | '*');
        if bad || ch.is_control() {
            out.push('_');
        } else {
            out.push(ch);
        }
    }
    let trimmed = out.trim().trim_matches('.').trim_matches('_').to_string();
    if trimmed.is_empty() { "track".to_string() } else { trimmed }
}

fn split_basename_and_ext(name: &str) -> (String, Option<String>) {
    let file_name = Path::new(name)
        .file_name()
        .and_then(|s| s.to_str())
        .unwrap_or(name);
    let stem = Path::new(file_name)
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("output")
        .to_string();
    let ext = Path::new(file_name)
        .extension()
        .and_then(|s| s.to_str())
        .map(|s| s.to_string());
    (stem, ext)
}

fn build_download_output_name(
    original_file_name: Option<&str>,
    remote_name: &str,
    used_names: &mut HashMap<String, usize>,
) -> String {
    let original_base = original_file_name
        .and_then(|s| Path::new(s).file_name().and_then(|n| n.to_str()))
        .unwrap_or("input");
    let (orig_stem_raw, _) = split_basename_and_ext(original_base);
    let (remote_stem_raw, remote_ext) = split_basename_and_ext(remote_name);
    let base_stem = sanitize_file_component(&orig_stem_raw);
    let suffix = sanitize_file_component(&remote_stem_raw);
    let merged = format!("{}_{}", base_stem, suffix);
    let key = merged.to_lowercase();
    let seq = used_names.entry(key).or_insert(0);
    *seq += 1;
    let deduped = if *seq > 1 {
        format!("{}_{}", merged, *seq)
    } else {
        merged
    };
    match remote_ext {
        Some(ext) if !ext.is_empty() => format!("{}.{}", deduped, ext),
        _ => deduped,
    }
}

fn get_partial_file_path(final_path: &Path) -> PathBuf {
    PathBuf::from(format!("{}.part", final_path.to_string_lossy()))
}

fn get_partial_meta_path(part_path: &Path) -> PathBuf {
    PathBuf::from(format!("{}.meta.json", part_path.to_string_lossy()))
}

fn load_partial_download_meta(meta_path: &Path) -> Option<PartialDownloadMeta> {
    let content = fs::read_to_string(meta_path).ok()?;
    serde_json::from_str::<PartialDownloadMeta>(&content).ok()
}

fn save_partial_download_meta(meta_path: &Path, meta: &PartialDownloadMeta) -> Result<(), String> {
    ensure_parent_dir(meta_path).map_err(|e| e.to_string())?;
    let content = serde_json::to_string_pretty(meta).map_err(|e| e.to_string())?;
    fs::write(meta_path, content).map_err(|e| e.to_string())
}

fn parse_total_bytes_from_content_range(response: &reqwest::Response) -> Option<u64> {
    let value = response.headers().get(reqwest::header::CONTENT_RANGE)?;
    let raw = value.to_str().ok()?;
    let slash = raw.rfind('/')?;
    raw[(slash + 1)..].trim().parse::<u64>().ok()
}

fn default_formats() -> Vec<OutputFormat> {
    vec![
        OutputFormat { id: 0, name: "MP3 (320 kbps)".to_string() },
        OutputFormat { id: 1, name: "WAV (16 bit)".to_string() },
        OutputFormat { id: 2, name: "FLAC (16 bit)".to_string() },
        OutputFormat { id: 3, name: "M4A (lossy)".to_string() },
        OutputFormat { id: 4, name: "WAV (32 bit)".to_string() },
        OutputFormat { id: 5, name: "FLAC (24 bit)".to_string() },
    ]
}

async fn parse_json_value(resp: reqwest::Response) -> Result<serde_json::Value, String> {
    let text = resp.text().await.map_err(|e| e.to_string())?;
    serde_json::from_str(&text).map_err(|e| {
        let preview = text.chars().take(120).collect::<String>();
        format!("error decoding response body: {}; body preview: {}", e, preview)
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

    Err(format!("all candidate endpoints failed: {}", attempts.join(" | ")))
}

fn parse_algorithms_from_value(value: &serde_json::Value) -> Vec<serde_json::Value> {
    if let Some(arr) = value.as_array() {
        if arr.first().and_then(|v| v.get("algorithms")).is_some() {
            let mut flattened = Vec::new();
            for group in arr {
                let gid = group.get("id").cloned().unwrap_or(serde_json::json!(0));
                let gname = group.get("name").cloned().unwrap_or(serde_json::json!("Ungrouped"));
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
                            mapped["render_id"] = mapped.get("id").cloned().unwrap_or(serde_json::json!(0));
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
    let name = algo.get("name").and_then(|v| v.as_str()).unwrap_or("Unknown").to_string();
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
        let name = algo.get("name").and_then(|v| v.as_str()).unwrap_or("Unknown").to_string();
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
        .map(|(id, (name, algorithms))| AlgorithmGroupData { id, name, algorithms })
        .collect();

    (groups, details_by_id)
}

async fn fetch_remote_algorithms_raw(
    state: &State<'_, AppState>,
    api_url: &str,
    token: &str,
) -> Result<Vec<serde_json::Value>, String> {
    let client = build_http_client(state)?;
    let body = get_json_with_fallback(
        &client,
        api_url,
        token,
        &["/app/algorithms", "/algorithm_groups"],
        vec![("scopes".to_string(), "single_upload".to_string())],
    )
    .await?;
    Ok(parse_algorithms_from_value(&body))
}

fn load_algorithm_cache_file() -> Result<AlgorithmCacheFile, String> {
    let path = get_algorithm_cache_path();
    ensure_config_dir().map_err(|e| e.to_string())?;
    ensure_parent_dir(&path).map_err(|e| e.to_string())?;
    if !path.exists() {
        let initial = serde_json::to_string_pretty(&AlgorithmCacheFile::default()).map_err(|e| e.to_string())?;
        fs::write(&path, initial).map_err(|e| e.to_string())?;
    }
    let content = fs::read_to_string(&path).map_err(|e| e.to_string())?;
    match serde_json::from_str::<AlgorithmCacheFile>(&content) {
        Ok(cache) => Ok(cache),
        Err(parse_err) => {
            let parent = path.parent().unwrap_or_else(|| Path::new("."));
            let backup_name = format!(
                "algorithms_cache.broken.{}.json",
                now_timestamp().replace('.', "_")
            );
            let backup_path = parent.join(backup_name);
            if let Err(copy_err) = fs::copy(&path, &backup_path) {
                eprintln!(
                    "[backend:WARN] failed to backup broken algorithm cache {} -> {}: {}",
                    path.to_string_lossy(),
                    backup_path.to_string_lossy(),
                    copy_err
                );
            } else {
                eprintln!(
                    "[backend:WARN] broken algorithm cache backed up: {}",
                    backup_path.to_string_lossy()
                );
            }

            let recovered = AlgorithmCacheFile::default();
            let recovered_content =
                serde_json::to_string_pretty(&recovered).map_err(|e| e.to_string())?;
            fs::write(&path, recovered_content).map_err(|e| e.to_string())?;
            eprintln!(
                "[backend:WARN] algorithm cache was invalid and has been recovered: {} ({})",
                path.to_string_lossy(),
                parse_err
            );
            Ok(recovered)
        }
    }
}

fn save_algorithm_cache_file(cache: &AlgorithmCacheFile) -> Result<(), String> {
    ensure_config_dir().map_err(|e| e.to_string())?;
    let path = get_algorithm_cache_path();
    let content = serde_json::to_string_pretty(cache).map_err(|e| e.to_string())?;
    fs::write(path, content).map_err(|e| e.to_string())
}

fn get_total_algorithms(groups: &[AlgorithmGroupData]) -> usize {
    groups.iter().map(|g| g.algorithms.len()).sum()
}

fn read_i32(value: Option<&serde_json::Value>) -> Option<i32> {
    if let Some(v) = value {
        if let Some(n) = v.as_i64() {
            return Some(n as i32);
        }
        if let Some(s) = v.as_str() {
            return s.parse::<i32>().ok();
        }
    }
    None
}

fn now_timestamp() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    match SystemTime::now().duration_since(UNIX_EPOCH) {
        Ok(duration) => format!("{}.{:03}", duration.as_secs(), duration.subsec_millis()),
        Err(_) => "0.000".to_string(),
    }
}

fn push_backend_log(state: &State<'_, AppState>, level: &str, message: impl Into<String>) {
    if let Ok(mut logs) = state.backend_logs.lock() {
        logs.push(LogEntry {
            timestamp: now_timestamp(),
            level: level.to_string(),
            message: message.into(),
        });
        if logs.len() > 2000 {
            let drain = logs.len() - 2000;
            logs.drain(0..drain);
        }
    }
}

#[tauri::command]
fn load_config() -> Result<Config, String> {
    let path = get_config_path();
    if path.exists() {
        let content = fs::read_to_string(&path).map_err(|e| e.to_string())?;
        serde_json::from_str(&content).map_err(|e| e.to_string())
    } else {
        Ok(Config::default())
    }
}

#[tauri::command]
fn save_config(state: State<'_, AppState>, config: Config) -> Result<(), String> {
    ensure_config_dir().map_err(|e| e.to_string())?;
    let path = get_config_path();
    let content = serde_json::to_string_pretty(&config).map_err(|e| e.to_string())?;
    fs::write(path, content).map_err(|e| e.to_string())?;
    if let Ok(mut guard) = state.config.lock() {
        *guard = config;
    }
    Ok(())
}

#[tauri::command]
fn resolve_path(path: String) -> Result<String, String> {
    let p = PathBuf::from(path);
    let absolute = if p.is_absolute() {
        p
    } else {
        std::env::current_dir().map_err(|e| e.to_string())?.join(p)
    };
    Ok(absolute.to_string_lossy().to_string())
}

#[tauri::command]
fn get_algorithm_cache_path_cmd() -> Result<String, String> {
    let path = get_algorithm_cache_path();
    Ok(path.to_string_lossy().to_string())
}

#[tauri::command]
async fn open_in_file_manager(state: State<'_, AppState>, path: String) -> Result<(), String> {
    let input = PathBuf::from(path);
    let mut target = if input.is_absolute() {
        input
    } else {
        std::env::current_dir().map_err(|e| e.to_string())?.join(input)
    };

    if !target.exists() {
        if let Some(parent) = target.parent() {
            if parent.exists() {
                target = parent.to_path_buf();
            }
        }
    }
    push_backend_log(
        &state,
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
                match OpenDirectoryRequest::default().send(&dir_file.as_fd()).await {
                    Ok(_) => {
                    push_backend_log(
                        &state,
                        "INFO",
                        format!("open_in_file_manager success via xdg-desktop-portal {}", target_s),
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
                        &state,
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
        push_backend_log(&state, "ERROR", error.clone());
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
        let status = cmd
            .status()
            .map_err(|e| e.to_string())?;
        if status.success() {
            push_backend_log(&state, "INFO", "open_in_file_manager success via explorer");
            eprintln!("[backend:INFO] open_in_file_manager success via explorer");
            return Ok(());
        }
        push_backend_log(&state, "ERROR", "open_in_file_manager failed via explorer");
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
            push_backend_log(&state, "INFO", "open_in_file_manager success via open");
            eprintln!("[backend:INFO] open_in_file_manager success via open");
            return Ok(());
        }
        push_backend_log(&state, "ERROR", "open_in_file_manager failed via open");
        eprintln!("[backend:ERROR] open_in_file_manager failed via open");
        return Err("failed to open with open".to_string());
    }

    #[allow(unreachable_code)]
    Err("unsupported platform".to_string())
}

// ============== API 相关 ==============

#[tauri::command]
async fn test_connection(
    state: State<'_, AppState>,
    token: String,
    api_url: String,
) -> Result<bool, String> {
    push_backend_log(&state, "INFO", format!("test_connection start: {}", api_url));
    let client = build_http_client(&state)?;
    let result = get_json_with_fallback(
        &client,
        &api_url,
        &token,
        &["/app/algorithms", "/algorithm_groups"],
        vec![("scopes".to_string(), "single_upload".to_string())],
    )
    .await;
    let ok = result.is_ok();
    push_backend_log(&state, if ok { "INFO" } else { "ERROR" }, format!("test_connection result: {}", ok));
    Ok(ok)
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
    push_backend_log(&state, "INFO", "fetch_latest_algorithm_info start");
    eprintln!("[backend:INFO] fetch_latest_algorithm_info start");
    let api_url_for_cli = if api_url.trim_end_matches('/').ends_with("/api") {
        api_url.clone()
    } else {
        format!("{}/api", api_url.trim_end_matches('/'))
    };
    let token_for_cli = token.clone();
    let proxy_mode_for_cli = proxy_mode.unwrap_or_else(|| "system".to_string());
    let proxy_host_for_cli = proxy_host.unwrap_or_else(|| "127.0.0.1".to_string());
    let proxy_port_for_cli = proxy_port.unwrap_or_else(|| "7897".to_string());

    let cli_output = tokio::task::spawn_blocking(move || {
        let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        let manifest_path = manifest_dir.join("../Cargo.toml");
        if !manifest_path.exists() {
            eprintln!("[backend:WARN] CLI manifest not packaged, skipping");
            return Ok(Output {
                status: ExitStatus::from_raw(0),
                stdout: Vec::new(),
                stderr: Vec::new(),
            });
        }
        let mut cmd = Command::new("cargo");
        cmd.current_dir(&manifest_dir);
        cmd.arg("run")
            .arg("--manifest-path")
            .arg("../Cargo.toml")
            .arg("--")
            .arg("list")
            .arg("-r")
            .env("MVSEP_API_URL", api_url_for_cli)
            .env("MVSEP_API_TOKEN", token_for_cli);

        if proxy_mode_for_cli == "manual" {
            cmd.env("PROXY_HOST", proxy_host_for_cli.clone());
            cmd.env("HTTP_PROXY_PORT", proxy_port_for_cli.clone());
        }
        if proxy_mode_for_cli == "none" {
            cmd.env("HTTP_PROXY", "");
            cmd.env("HTTPS_PROXY", "");
            cmd.env("http_proxy", "");
            cmd.env("https_proxy", "");
        }

        cmd.output()
    })
    .await
    .map_err(|e| e.to_string())?
    .map_err(|e| e.to_string())?;

    let cli_exit_code = cli_output.status.code().unwrap_or(-1);
    if !cli_output.status.success() {
        let stderr = String::from_utf8_lossy(&cli_output.stderr).to_string();
        eprintln!(
            "[backend:ERROR] fetch_latest_algorithm_info cli failed: code={}, stderr={}",
            cli_exit_code, stderr
        );
        push_backend_log(
            &state,
            "ERROR",
            format!("fetch_latest_algorithm_info cli failed: code={}, stderr={}", cli_exit_code, stderr),
        );
        return Err(format!("CLI command failed: exit code {}, {}", cli_exit_code, stderr));
    }

    let raw_cli_stdout = String::from_utf8_lossy(&cli_output.stdout).to_string();
    eprintln!("[backend:INFO] fetch_latest_algorithm_info remote fetch start: {}", api_url);
    let algorithms = fetch_remote_algorithms_raw(&state, &api_url, &token)
        .await
        .map_err(|e| {
            push_backend_log(&state, "ERROR", format!("fetch_latest_algorithm_info remote fetch failed: {}", e));
            eprintln!("[backend:ERROR] fetch_latest_algorithm_info remote fetch failed: {}", e);
            e
        })?;
    let (groups, details_by_id) = normalize_algorithm_groups_and_details(algorithms);
    let updated_at = now_timestamp();
    let cache = AlgorithmCacheFile {
        updated_at: updated_at.clone(),
        raw_cli_stdout,
        groups: groups.clone(),
        details_by_id,
    };
    save_algorithm_cache_file(&cache).map_err(|e| {
        push_backend_log(&state, "ERROR", format!("fetch_latest_algorithm_info save cache failed: {}", e));
        e
    })?;

    let total_algorithms = get_total_algorithms(&groups);
    push_backend_log(
        &state,
        "INFO",
        format!(
            "fetch_latest_algorithm_info success: groups={}, algorithms={}",
            groups.len(),
            total_algorithms
        ),
    );
    eprintln!(
        "[backend:INFO] fetch_latest_algorithm_info success: groups={}, algorithms={}",
        groups.len(),
        total_algorithms
    );
    Ok(FetchLatestAlgorithmInfoResult {
        updated_at,
        total_groups: groups.len(),
        total_algorithms,
        cli_exit_code,
    })
}

#[tauri::command]
fn refresh_algorithm_list_from_local(state: State<'_, AppState>) -> Result<LocalAlgorithmListResponse, String> {
    let cache_path = get_algorithm_cache_path();
    let cache_missing_before = !cache_path.exists();
    let cache = load_algorithm_cache_file().map_err(|e| {
        push_backend_log(&state, "ERROR", format!("refresh_algorithm_list_from_local failed: {}", e));
        e
    })?;
    if cache_missing_before {
        push_backend_log(
            &state,
            "INFO",
            format!(
                "refresh_algorithm_list_from_local initialized missing cache file: {}",
                cache_path.to_string_lossy()
            ),
        );
    }
    let total_algorithms = get_total_algorithms(&cache.groups);
    push_backend_log(
        &state,
        "INFO",
        format!("refresh_algorithm_list_from_local success: algorithms={}", total_algorithms),
    );
    Ok(LocalAlgorithmListResponse {
        updated_at: cache.updated_at,
        groups: cache.groups,
        total_algorithms,
    })
}

#[tauri::command]
fn get_algorithm_details_from_local(
    state: State<'_, AppState>,
    algorithm_id: i32,
) -> Result<AlgorithmDetails, String> {
    let cache_path = get_algorithm_cache_path();
    let cache_missing_before = !cache_path.exists();
    let cache = load_algorithm_cache_file().map_err(|e| {
        push_backend_log(&state, "ERROR", format!("get_algorithm_details_from_local cache read failed: {}", e));
        e
    })?;
    if cache_missing_before {
        push_backend_log(
            &state,
            "INFO",
            format!(
                "get_algorithm_details_from_local initialized missing cache file: {}",
                cache_path.to_string_lossy()
            ),
        );
    }
    if let Some(details) = cache.details_by_id.get(&algorithm_id) {
        return Ok(details.clone());
    }

    for group in &cache.groups {
        if let Some(item) = group.algorithms.iter().find(|algo| algo.id == algorithm_id) {
            let fallback = AlgorithmDetails {
                id: algorithm_id,
                name: item.name.clone(),
                fields: Vec::new(),
            };
            push_backend_log(
                &state,
                "WARN",
                format!(
                    "get_algorithm_details_from_local fallback without fields: {}",
                    algorithm_id
                ),
            );
            return Ok(fallback);
        }
    }

    push_backend_log(
        &state,
        "ERROR",
        format!("get_algorithm_details_from_local not found: {}", algorithm_id),
    );
    Err("Algorithm not found in local cache".to_string())
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
    push_backend_log(
        &state,
        "INFO",
        format!("list_algorithms start: keyword={}", keywords.clone().unwrap_or_default()),
    );
    let mut algorithms = fetch_remote_algorithms_raw(&state, &api_url, &token).await.map_err(|e| {
        push_backend_log(&state, "ERROR", format!("list_algorithms failed: {}", e));
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
        push_backend_log(&state, "INFO", format!("list_algorithms search result: {}", results.len()));
        return Ok(serde_json::json!({ "algorithms": results }));
    }

    if let Some(alg_id) = algorithm_id {
        if let Some(found) = algorithms.into_iter().find(|algo| read_i32(algo.get("render_id")) == Some(alg_id)) {
            push_backend_log(&state, "INFO", format!("list_algorithms single algorithm: {}", alg_id));
            return Ok(serde_json::json!({
                "id": read_i32(found.get("render_id")).unwrap_or(0),
                "name": found.get("name").and_then(|v| v.as_str()).unwrap_or("Unknown"),
                "group_id": read_i32(found.get("algorithm_group").and_then(|g| g.get("id"))).unwrap_or(0)
            }));
        }
        push_backend_log(&state, "ERROR", format!("list_algorithms algorithm not found: {}", alg_id));
        return Err("Algorithm not found".to_string());
    }

    let (groups, _) = normalize_algorithm_groups_and_details(algorithms);
    push_backend_log(&state, "INFO", format!("list_algorithms grouped result: {} groups", groups.len()));
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

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
struct AlgorithmCacheFile {
    #[serde(default)]
    pub updated_at: String,
    #[serde(default)]
    pub raw_cli_stdout: String,
    #[serde(default)]
    pub groups: Vec<AlgorithmGroupData>,
    #[serde(default)]
    pub details_by_id: BTreeMap<i32, AlgorithmDetails>,
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

#[tauri::command]
async fn get_algorithm_details(
    state: State<'_, AppState>,
    algorithm_id: i32,
    api_url: String,
    token: String,
) -> Result<AlgorithmDetails, String> {
    push_backend_log(&state, "INFO", format!("get_algorithm_details start: {}", algorithm_id));
    let algorithms = fetch_remote_algorithms_raw(&state, &api_url, &token).await.map_err(|e| {
        push_backend_log(&state, "ERROR", format!("get_algorithm_details failed: {}", e));
        e
    })?;

    for algo in algorithms {
        if read_i32(algo.get("render_id")) == Some(algorithm_id) {
            let details = parse_algorithm_details_from_value(&algo);
            push_backend_log(
                &state,
                "INFO",
                format!("get_algorithm_details ok: {}, {} fields", algorithm_id, details.fields.len()),
            );
            return Ok(details);
        }
    }

    push_backend_log(&state, "ERROR", format!("get_algorithm_details not found: {}", algorithm_id));
    Err("Algorithm not found".to_string())
}

#[tauri::command]
async fn list_formats(
    state: State<'_, AppState>,
    api_url: String,
    token: String,
) -> Result<Vec<OutputFormat>, String> {
    push_backend_log(&state, "INFO", "list_formats start");
    let client = build_http_client(&state)?;
    let body = get_json_with_fallback(
        &client,
        &api_url,
        &token,
        &["/output_formats", "/app/output_formats"],
        vec![],
    )
    .await;
    if let Ok(value) = body {
        let arr = if let Some(items) = value.as_array() {
            items.to_vec()
        } else if let Some(items) = value.get("data").and_then(|v| v.as_array()) {
            items.to_vec()
        } else {
            Vec::new()
        };
        let parsed: Vec<OutputFormat> = arr
            .iter()
            .filter_map(|item| {
                let id = read_i32(item.get("id"))?;
                let name = item.get("name").and_then(|v| v.as_str()).unwrap_or("Unknown").to_string();
                Some(OutputFormat { id, name })
            })
            .collect();
        if !parsed.is_empty() {
            push_backend_log(&state, "INFO", format!("list_formats remote count: {}", parsed.len()));
            return Ok(parsed);
        }
    }
    push_backend_log(&state, "WARN", "list_formats fallback to defaults");
    Ok(default_formats())
}

#[tauri::command]
async fn query_name(
    state: State<'_, AppState>,
    algorithm_id: i32,
    model_id: Option<String>,
    api_url: String,
    token: String,
) -> Result<String, String> {
    let client = build_http_client(&state)?;
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
    state: &State<'_, AppState>,
    hash: &str,
    token: Arc<AtomicBool>,
) -> Result<(), String> {
    let mut guard = state
        .download_cancellations
        .lock()
        .map_err(|e| e.to_string())?;
    guard.insert(hash.to_string(), token);
    Ok(())
}

fn unregister_download_token(state: &State<'_, AppState>, hash: &str) {
    if let Ok(mut map) = state.download_cancellations.lock() {
        map.remove(hash);
    }
}

#[tauri::command]
fn cancel_download(state: State<'_, AppState>, hash: String) -> Result<(), String> {
    let map = state
        .download_cancellations
        .lock()
        .map_err(|e| e.to_string())?;
    if let Some(token) = map.get(&hash) {
        token.store(true, Ordering::SeqCst);
    }
    Ok(())
}

#[tauri::command]
async fn get_queue_info(
    state: State<'_, AppState>,
    api_url: String,
    token: String,
) -> Result<QueueStatus, String> {
    let client = build_http_client(&state)?;
    let info = get_json_with_fallback(
        &client,
        &api_url,
        &token,
        &["/app/queue", "/queue"],
        vec![],
    )
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
            push_backend_log(&state, "INFO", format!("get_queue_info active={}, queued={}", active, queued));
        }
    }
    Ok(QueueStatus { active, queued })
}

#[tauri::command]
async fn get_remote_history(
    state: State<'_, AppState>,
    limit: Option<i32>,
    api_url: String,
    token: String,
) -> Result<serde_json::Value, String> {
    push_backend_log(&state, "INFO", format!("get_remote_history start: limit={}", limit.unwrap_or(20)));
    let client = build_http_client(&state)?;
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
            push_backend_log(&state, "ERROR", format!("get_remote_history request failed: {}", msg));
            msg
        })?;

    response.json().await.map_err(|e| {
        let msg = e.to_string();
        push_backend_log(&state, "ERROR", format!("get_remote_history decode failed: {}", msg));
        msg
    })
}

#[tauri::command]
async fn create_task(
    state: State<'_, AppState>,
    window: tauri::Window,
    file_path: String,
    sep_type: i32,
    opt1: Option<i32>,
    opt2: Option<i32>,
    opt3: Option<i32>,
    output_format: Option<i32>,
    demo: bool,
    api_url: String,
    token: String,
) -> Result<String, String> {
    push_backend_log(&state, "INFO", format!("create_task start: {}", file_path));
    eprintln!("[backend:INFO] create_task start: {}", file_path);
    let client = build_http_client(&state)?;
    let url = build_api_url(&api_url, "/separation/create");

    let file_name = std::path::Path::new(&file_path)
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("audio.wav");

    let file = tokio::fs::File::open(&file_path).await.map_err(|e| {
        let msg = e.to_string();
        push_backend_log(&state, "ERROR", format!("create_task open file failed: {}", msg));
        eprintln!("[backend:ERROR] create_task open file failed: {}", msg);
        msg
    })?;
    let file_size = file.metadata().await.map_err(|e| {
        let msg = e.to_string();
        push_backend_log(&state, "ERROR", format!("create_task read metadata failed: {}", msg));
        eprintln!("[backend:ERROR] create_task read metadata failed: {}", msg);
        msg
    })?.len();
    let file_size_for_percent = file_size.max(1);

    let uploaded = Arc::new(AtomicU64::new(0));
    let started = Instant::now();
    let last_emit = Arc::new(Mutex::new(Instant::now()));
    let file_name_for_emit = file_name.to_string();
    let window_for_stream = window.clone();

    let uploaded_clone = uploaded.clone();
    let last_emit_clone = last_emit.clone();
    let stream = ReaderStream::new(file).map_ok(move |chunk| {
        let total_uploaded = uploaded_clone.fetch_add(chunk.len() as u64, Ordering::SeqCst) + chunk.len() as u64;
        let mut should_emit = false;
        if let Ok(mut last) = last_emit_clone.lock() {
            if last.elapsed().as_millis() >= 120 {
                *last = Instant::now();
                should_emit = true;
            }
        }
        if should_emit {
            let elapsed = started.elapsed().as_secs_f64().max(0.001);
            let speed_bps = total_uploaded as f64 / elapsed;
            let payload = UploadProgressPayload {
                file_name: file_name_for_emit.clone(),
                uploaded_bytes: total_uploaded,
                total_bytes: file_size,
                speed_bps,
                percent: (total_uploaded as f32 / file_size_for_percent as f32) * 100.0,
                done: false,
                failed: false,
            };
            let _ = window_for_stream.emit("upload-progress", payload);
        }
        chunk
    });

    let part = reqwest::multipart::Part::stream_with_length(reqwest::Body::wrap_stream(stream), file_size)
        .file_name(file_name.to_string())
        .mime_str("audio/*")
        .map_err(|e| e.to_string())?;

    let demo_str = if demo { "1" } else { "0" };

    let mut form = reqwest::multipart::Form::new()
        .text("sep_type", sep_type.to_string())
        .text("output_format", output_format.unwrap_or(1).to_string())
        .text("is_demo", demo_str)
        .text("api_token", token.clone())
        .part("audiofile", part);

    if let Some(o1) = opt1 {
        form = form.text("add_opt1", o1.to_string());
    }
    if let Some(o2) = opt2 {
        form = form.text("add_opt2", o2.to_string());
    }
    if let Some(o3) = opt3 {
        form = form.text("add_opt3", o3.to_string());
    }

    let response = client
        .post(&url)
        .multipart(form)
        .send()
        .await
        .map_err(|e| {
            let msg = e.to_string();
            push_backend_log(&state, "ERROR", format!("create_task request failed: {}", msg));
            eprintln!("[backend:ERROR] create_task request failed: {}", msg);
            msg
        })?;

    let body = match parse_json_value(response).await {
        Ok(v) => v,
        Err(e) => {
            let total_uploaded = uploaded.load(Ordering::SeqCst);
            let elapsed = started.elapsed().as_secs_f64().max(0.001);
            let speed_bps = total_uploaded as f64 / elapsed;
            let payload = UploadProgressPayload {
                file_name: file_name.to_string(),
                uploaded_bytes: total_uploaded,
                total_bytes: file_size,
                speed_bps,
                percent: (total_uploaded as f32 / file_size_for_percent as f32) * 100.0,
                done: true,
                failed: true,
            };
            let _ = window.emit("upload-progress", payload);
            push_backend_log(&state, "ERROR", format!("create_task parse response failed: {}", e));
            eprintln!("[backend:ERROR] create_task parse response failed: {}", e);
            return Err(e);
        }
    };

    let total_uploaded = uploaded.load(Ordering::SeqCst).max(file_size);
    let elapsed = started.elapsed().as_secs_f64().max(0.001);
    let speed_bps = total_uploaded as f64 / elapsed;
    let payload = UploadProgressPayload {
        file_name: file_name.to_string(),
        uploaded_bytes: total_uploaded,
        total_bytes: file_size,
        speed_bps,
        percent: 100.0,
        done: true,
        failed: false,
    };
    let _ = window.emit("upload-progress", payload);
    let hash = body
        .get("data")
        .and_then(|d| d.get("hash"))
        .or_else(|| body.get("hash"))
        .or_else(|| body.get("task_hash"))
        .and_then(|v| v.as_str())
        .ok_or("Failed to get task hash")?
        .to_string();

    push_backend_log(&state, "INFO", format!("create_task success: hash={}", hash));
    eprintln!("[backend:INFO] create_task success: hash={}", hash);
    Ok(hash)
}

#[tauri::command]
async fn get_task_status(
    state: State<'_, AppState>,
    hash: String,
    api_url: String,
    token: String,
) -> Result<TaskStatus, String> {
    let client = build_http_client(&state)?;
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
            push_backend_log(&state, "ERROR", format!("get_task_status request failed: hash={}, {}", hash, msg));
            msg
        })?;

    let body = parse_json_value(response).await?;

    let status = body.get("status")
        .and_then(|v| v.as_str())
        .unwrap_or("unknown")
        .to_string();

    let message = body
        .get("data")
        .and_then(|d| d.get("message"))
        .and_then(|v| v.as_str())
        .map(|s| s.to_string())
        .or_else(|| body.get("message").and_then(|v| v.as_str()).map(|s| s.to_string()));

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
                                .and_then(|u| u.split('/').last().map(|s| s.to_string()))
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
        &state,
        "INFO",
        format!(
            "get_task_status hash={} status={} progress={:.2}",
            hash, result.status, result.progress
        ),
    );
    Ok(result)
}

#[tauri::command]
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
    push_backend_log(&state, "INFO", format!("download_result start: hash={}", hash));
    eprintln!("[backend:INFO] download_result start: hash={}", hash);
    let cancel_token = Arc::new(AtomicBool::new(false));
    register_download_token(&state, &hash, cancel_token.clone())?;
    let download_job = async {
        let client = build_http_client(&state)?;
        let url = build_api_url(&api_url, "/separation/get");
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
                push_backend_log(&state, "ERROR", format!("download_result query failed: {}", msg));
                eprintln!("[backend:ERROR] download_result query failed: {}", msg);
                msg
            })?;
        let body = parse_json_value(response).await?;
        let files = body
            .get("data")
            .and_then(|d| d.get("files"))
            .and_then(|v| v.as_array())
            .ok_or("No files to download")?;

        let mut downloaded: Vec<String> = Vec::new();
        let output_dir_path = PathBuf::from(&output_dir);
        let normalized_output_dir = if output_dir_path.is_absolute() {
            output_dir_path
        } else {
            std::env::current_dir()
                .map_err(|e| e.to_string())?
                .join(output_dir_path)
        };

        fs::create_dir_all(&normalized_output_dir).map_err(|e| e.to_string())?;
        let mut used_names: HashMap<String, usize> = HashMap::new();
        let original_ref = original_file_name.as_deref();

        for (i, file_info) in files.iter().enumerate() {
            if let Some(idx) = file_index {
                if i != idx as usize {
                    continue;
                }
            }

            if cancel_token.load(Ordering::SeqCst) {
                push_backend_log(
                    &state,
                    "WARN",
                    format!("download_result cancelled for hash={}", hash),
                );
                eprintln!("[backend:WARN] download_result cancelled for hash={}", hash);
                return Err("Download cancelled".to_string());
            }

            let file_url = file_info
                .get("url")
                .and_then(|v| v.as_str())
                .ok_or("File URL missing")?;
            let file_name = file_info
                .get("name")
                .and_then(|v| v.as_str())
                .or_else(|| file_url.split('/').last())
                .unwrap_or("output.bin");
            let local_file_name = build_download_output_name(
                original_ref,
                file_name,
                &mut used_names,
            );
            let output_path = normalized_output_dir.join(&local_file_name);
            let part_path = get_partial_file_path(&output_path);
            let meta_path = get_partial_meta_path(&part_path);
            ensure_parent_dir(&part_path).map_err(|e| e.to_string())?;
            ensure_parent_dir(&meta_path).map_err(|e| e.to_string())?;

            let mut resume_from: u64 = 0;
            if let Ok(metadata) = fs::metadata(&part_path) {
                let part_size = metadata.len();
                if part_size > 0 {
                    let can_resume = load_partial_download_meta(&meta_path)
                        .map(|meta| meta.file_url == file_url)
                        .unwrap_or(true);
                    if can_resume {
                        resume_from = part_size;
                    } else {
                        let _ = fs::remove_file(&part_path);
                        let _ = fs::remove_file(&meta_path);
                    }
                }
            }

            let meta = PartialDownloadMeta {
                file_url: file_url.to_string(),
                remote_file_name: file_name.to_string(),
                updated_at: now_timestamp(),
            };
            save_partial_download_meta(&meta_path, &meta)?;

            let mut request = client.get(file_url);
            if resume_from > 0 {
                request = request.header(reqwest::header::RANGE, format!("bytes={}-", resume_from));
                push_backend_log(
                    &state,
                    "INFO",
                    format!("download_result resume attempt: hash={}, file={}, offset={}", hash, local_file_name, resume_from),
                );
            }

            let response = request.send().await.map_err(|e| {
                let msg = e.to_string();
                push_backend_log(
                    &state,
                    "ERROR",
                    format!("download_result stream request failed: hash={}, file={}, {}", hash, local_file_name, msg),
                );
                msg
            })?;

            let status = response.status();
            let (append_mode, total_bytes) = if resume_from > 0 && status == reqwest::StatusCode::PARTIAL_CONTENT {
                let total_bytes = parse_total_bytes_from_content_range(&response)
                    .or_else(|| response.content_length().map(|len| resume_from + len));
                (true, total_bytes)
            } else if status.is_success() {
                if resume_from > 0 {
                    push_backend_log(
                        &state,
                        "WARN",
                        format!(
                            "download_result resume rejected, restart full: hash={}, file={}, status={}",
                            hash, local_file_name, status
                        ),
                    );
                }
                resume_from = 0;
                let total_bytes = response.content_length();
                let _ = fs::remove_file(&part_path);
                (false, total_bytes)
            } else {
                let msg = format!("http status {} while downloading file", status);
                push_backend_log(
                    &state,
                    "ERROR",
                    format!("download_result stream bad status: hash={}, file={}, {}", hash, local_file_name, msg),
                );
                return Err(msg);
            };

            let mut options = tokio::fs::OpenOptions::new();
            options.create(true).write(true);
            if append_mode {
                options.append(true);
            } else {
                options.truncate(true);
            }
            let mut file = options.open(&part_path).await.map_err(|e| e.to_string())?;

            let mut stream = response.bytes_stream();
            let mut downloaded_bytes: u64 = resume_from;
            let mut session_downloaded_bytes: u64 = 0;
            let started = Instant::now();
            let mut last_emit = Instant::now();

            while let Some(chunk) = stream.next().await {
                let chunk = chunk.map_err(|e| e.to_string())?;
                file.write_all(&chunk).await.map_err(|e| e.to_string())?;
                let chunk_len = chunk.len() as u64;
                downloaded_bytes += chunk_len;
                session_downloaded_bytes += chunk_len;

                if cancel_token.load(Ordering::SeqCst) {
                    push_backend_log(
                        &state,
                        "WARN",
                        format!("download_result cancelled mid-stream for hash={}", hash),
                    );
                    eprintln!("[backend:WARN] download_result cancelled mid-stream for hash={}", hash);
                    return Err("Download cancelled".to_string());
                }

                if last_emit.elapsed().as_millis() >= 150 {
                    let elapsed = started.elapsed().as_secs_f64().max(0.001);
                    let speed_bps = session_downloaded_bytes as f64 / elapsed;
                    let percent = total_bytes
                        .map(|t| {
                            let total = t.max(1);
                            (downloaded_bytes as f32 / total as f32) * 100.0
                        })
                        .unwrap_or(0.0);
                    let payload = DownloadProgressPayload {
                        hash: hash.clone(),
                        file_name: local_file_name.clone(),
                        downloaded_bytes,
                        total_bytes,
                        speed_bps,
                        percent,
                        done: false,
                    };
                    let _ = window.emit("download-progress", payload);
                    last_emit = Instant::now();
                }
            }

            file.flush().await.map_err(|e| e.to_string())?;

            if output_path.exists() {
                let _ = fs::remove_file(&output_path);
            }
            fs::rename(&part_path, &output_path).map_err(|e| e.to_string())?;
            let _ = fs::remove_file(&meta_path);

            let elapsed = started.elapsed().as_secs_f64().max(0.001);
            let speed_bps = session_downloaded_bytes as f64 / elapsed;
            let percent = total_bytes.map(|_| 100.0).unwrap_or(0.0);
            let payload = DownloadProgressPayload {
                hash: hash.clone(),
                file_name: local_file_name.clone(),
                downloaded_bytes,
                total_bytes,
                speed_bps,
                percent,
                done: true,
            };
            let _ = window.emit("download-progress", payload);

            downloaded.push(output_path.to_string_lossy().to_string());
        }

        push_backend_log(
            &state,
            "INFO",
            format!("download_result success: {} files", downloaded.len()),
        );
        eprintln!("[backend:INFO] download_result success: {} files", downloaded.len());
        Ok(downloaded)
    };

    let result = download_job.await;
    unregister_download_token(&state, &hash);
    result
}
// ============== 任务管理 ==============

#[tauri::command]
fn get_tasks(state: State<'_, AppState>) -> Result<Vec<TaskInfo>, String> {
    let tasks = state.tasks.lock().map_err(|e| e.to_string())?;
    Ok(tasks.values().cloned().collect())
}

#[tauri::command]
fn add_task(state: State<'_, AppState>, task: TaskInfo) -> Result<(), String> {
    let mut tasks = state.tasks.lock().map_err(|e| e.to_string())?;
    tasks.insert(task.hash.clone(), task);
    Ok(())
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
    let mut tasks = state.tasks.lock().map_err(|e| e.to_string())?;
    if let Some(task) = tasks.get_mut(&hash) {
        task.status = status;
        task.progress = progress;
        if let Some(f) = files {
            task.output_files = f;
        }
        if let Some(e) = error {
            task.error = Some(e);
        }
    }
    Ok(())
}

#[tauri::command]
fn remove_task(state: State<'_, AppState>, hash: String) -> Result<(), String> {
    let mut tasks = state.tasks.lock().map_err(|e| e.to_string())?;
    tasks.remove(&hash);
    Ok(())
}

#[tauri::command]
fn get_backend_logs(state: State<'_, AppState>) -> Result<Vec<LogEntry>, String> {
    let logs = state.backend_logs.lock().map_err(|e| e.to_string())?;
    Ok(logs.clone())
}

#[tauri::command]
fn frontend_debug_log(state: State<'_, AppState>, level: String, message: String) {
    let normalized = match level.to_uppercase().as_str() {
        "ERROR" => "ERROR",
        "WARN" => "WARN",
        "DEBUG" => "DEBUG",
        _ => "INFO",
    };
    let line = format!("[frontend] {}", message);
    push_backend_log(&state, normalized, line.clone());
    eprintln!("[frontend:{}] {}", normalized, message);
}

// ============== 主程序 ==============

fn main() {
    let app_state = AppState {
        config: Mutex::new(Config::default()),
        tasks: Mutex::new(HashMap::new()),
        backend_logs: Mutex::new(Vec::new()),
        last_queue_info: Mutex::new(None),
        download_cancellations: Mutex::new(HashMap::new()),
        http_client_cache: Mutex::new(None),
    };

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
                    tauri_plugin_log::TargetKind::LogDir { file_name: Some("mvsep".into()) },
                ))
                .build()
        )
        .manage(app_state)
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
            update_task_status,
            remove_task,
            get_backend_logs,
            frontend_debug_log,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
