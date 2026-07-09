//! 文件传输模块。
//!
//! 提供流式上传（async + 进度回调）、流式下载（断点续传 + 进度追踪）、
//! 文件重命名等功能。
//!
//! ## 上传
//!
//! [`upload_file`] 使用 tokio 异步运行时读取文件并流式发送 multipart 请求，
//! 通过回调实时报告上传进度。
//!
//! ## 下载
//!
//! [`download_file`] 流式读取响应体并写入磁盘，支持：
//! - **断点续传**：通过 `Range` 头 + `.part` 文件实现
//! - **进度回调**：每 150ms 报告速度、百分比、已传输字节
//! - **完成重命名**：`.part` → 最终文件
//!
//! ## 重命名
//!
//! [`build_local_name`] 将远端产物名映射为本地文件名：
//! ```text
//! 远端: 20260626..._melroformer_mt_10_vocals.flac
//! 本地: {原文件名主干}_vocals.flac
//! ```

use futures_util::StreamExt;
use std::fmt;
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use tokio::io::AsyncWriteExt;
use tokio_util::io::ReaderStream;

/// 上传/下载进度信息
///
/// 通过进度回调传递给调用方，每 ~120ms（上传）或 ~150ms（下载）更新一次。
///
/// # 示例
///
/// ```rust,no_run
/// use mvsep_api_tester::file_transfer::TransferProgress;
///
/// let progress_cb = |p: TransferProgress| {
///     println!("文件: {}", p.file_name);
///     println!("进度: {:.1}%", p.percent);
///     println!("速度: {:.1} KB/s", p.speed_bps / 1024.0);
///     if p.done {
///         println!("传输完成!");
///     }
/// };
/// ```
#[derive(Debug, Clone)]
pub struct TransferProgress {
    /// 文件名
    pub file_name: String,
    /// 已传输字节数
    pub bytes: u64,
    /// 总字节数（可能未知）
    pub total_bytes: Option<u64>,
    /// 传输速度（字节/秒）
    pub speed_bps: f64,
    /// 进度百分比（0-100）
    pub percent: f32,
    /// 是否已完成
    pub done: bool,
    /// 是否失败
    pub failed: bool,
}

/// 结构化传输错误
///
/// 用于异步调用者保存 HTTP/文件上下文信息，在转换为面向用户的字符串之前保留详细错误信息。
///
/// # 示例
///
/// ```rust,no_run
/// use mvsep_api_tester::file_transfer::TransferError;
///
/// let error = TransferError::new("网络连接失败")
///     .with_url("https://api.example.com/upload")
///     .with_http_status(503);
///
/// if error.is_cancelled() {
///     println!("传输已取消");
/// } else {
///     println!("错误: {}", error);
///     if let Some(status) = error.http_status() {
///         println!("HTTP 状态: {}", status);
///     }
/// }
/// ```
#[derive(Debug, Clone)]
pub struct TransferError {
    message: String,
    http_status: Option<u16>,
    url: Option<String>,
    path: Option<PathBuf>,
    cancelled: bool,
}

impl TransferError {
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
            http_status: None,
            url: None,
            path: None,
            cancelled: false,
        }
    }

    pub fn cancelled(message: impl Into<String>) -> Self {
        Self {
            cancelled: true,
            ..Self::new(message)
        }
    }

    pub fn with_http_status(mut self, status: u16) -> Self {
        self.http_status = Some(status);
        self
    }

    pub fn with_url(mut self, url: impl Into<String>) -> Self {
        self.url = Some(url.into());
        self
    }

    pub fn with_path(mut self, path: impl Into<PathBuf>) -> Self {
        self.path = Some(path.into());
        self
    }

    pub fn http_status(&self) -> Option<u16> {
        self.http_status
    }

    pub fn url(&self) -> Option<&str> {
        self.url.as_deref()
    }

    pub fn path(&self) -> Option<&Path> {
        self.path.as_deref()
    }

    pub fn is_cancelled(&self) -> bool {
        self.cancelled
    }
}

impl fmt::Display for TransferError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl std::error::Error for TransferError {}

/// Metadata stored alongside `.part` files for download resume.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
struct PartialDownloadMeta {
    file_url: String,
    remote_file_name: String,
    updated_at: String,
}

// ── Path helpers ──

/// Path to the `.part` file for partial downloads.
pub fn part_path(final_path: &Path) -> PathBuf {
    let mut p = final_path.to_path_buf();
    let name = format!(
        "{}.part",
        final_path.file_name().unwrap_or_default().to_string_lossy()
    );
    p.set_file_name(name);
    p
}

/// Path to the `.meta.json` file for partial download metadata.
pub fn meta_path(part_path: &Path) -> PathBuf {
    let mut p = part_path.to_path_buf();
    p.set_file_name(format!(
        "{}.meta.json",
        part_path.file_name().unwrap_or_default().to_string_lossy()
    ));
    p
}

fn now_timestamp() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    match SystemTime::now().duration_since(UNIX_EPOCH) {
        Ok(d) => format!("{}.{:03}", d.as_secs(), d.subsec_millis()),
        Err(_) => "0.000".to_string(),
    }
}

async fn save_partial_download_meta_async(
    meta_path: &Path,
    meta: &PartialDownloadMeta,
) -> Result<(), TransferError> {
    if let Some(parent) = meta_path.parent() {
        tokio::fs::create_dir_all(parent).await.map_err(|e| {
            TransferError::new(format!("create partial metadata directory failed: {}", e))
                .with_path(parent)
        })?;
    }
    let content = serde_json::to_string_pretty(meta).map_err(|e| {
        TransferError::new(format!("serialize partial metadata failed: {}", e)).with_path(meta_path)
    })?;
    tokio::fs::write(meta_path, content).await.map_err(|e| {
        TransferError::new(format!("write partial metadata failed: {}", e)).with_path(meta_path)
    })?;
    Ok(())
}

async fn get_resume_info_async(final_path: &Path, file_url: &str) -> u64 {
    let part = part_path(final_path);
    let meta = meta_path(&part);

    let part_size = match tokio::fs::metadata(&part).await {
        Ok(metadata) => metadata.len(),
        Err(_) => return 0,
    };
    if part_size == 0 {
        return 0;
    }

    let can_resume = match tokio::fs::read_to_string(&meta).await {
        Ok(content) => match serde_json::from_str::<PartialDownloadMeta>(&content) {
            Ok(pm) => pm.file_url == file_url,
            Err(_) => true,
        },
        Err(_) => true,
    };

    if can_resume {
        part_size
    } else {
        let _ = tokio::fs::remove_file(&part).await;
        let _ = tokio::fs::remove_file(&meta).await;
        0
    }
}

// ── File naming ──

/// Extract the stem (filename without extension) from a path.
fn file_stem(name: &str) -> String {
    Path::new(name)
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("output")
        .to_string()
}

/// Build a local filename from the original uploaded filename and the remote output filename.
///
/// Remote filenames follow the pattern: `{hash}_{...}_{suffix}.{ext}`
/// We extract everything after the last `_` (the suffix + extension),
/// and prefix it with the original file's stem.
///
/// Example:
///   original: "螢塚-Calvaria.mp3"
///   remote:   "20260626..._melroformer_mt_10_vocals.flac"
///   result:   "螢塚-Calvaria_vocals.flac"
pub fn build_local_name(original_name: &str, remote_name: &str) -> String {
    let original_stem = sanitize_name(&file_stem(original_name));

    // Find the last `_` in the remote name to isolate the suffix + ext
    if let Some(underscore_pos) = remote_name.rfind('_') {
        let suffix_and_ext = &remote_name[underscore_pos + 1..];
        format!("{}_{}", original_stem, suffix_and_ext)
    } else {
        // No underscore found — just use remote name as-is
        remote_name.to_string()
    }
}

/// Sanitize a filename component by replacing problematic characters.
fn sanitize_name(value: &str) -> String {
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
    if trimmed.is_empty() {
        "track".to_string()
    } else {
        trimmed
    }
}

// ── Upload ──

fn upload_error_message(status_code: u16, body_text: &str) -> String {
    let msg = serde_json::from_str::<serde_json::Value>(body_text)
        .ok()
        .and_then(|v| {
            v.get("errors")
                .and_then(|e| e.as_array())
                .map(|a| {
                    a.iter()
                        .filter_map(|v| v.as_str())
                        .collect::<Vec<_>>()
                        .join(", ")
                })
                .or_else(|| {
                    v.get("message")
                        .and_then(|v| v.as_str())
                        .map(|s| s.to_string())
                })
        })
        .unwrap_or_else(|| body_text.to_string());
    format!("HTTP {} - {}", status_code, msg)
}

fn extract_upload_hash(body_text: &str) -> Option<String> {
    let body = serde_json::from_str::<serde_json::Value>(body_text).ok()?;
    body.get("hash")
        .or_else(|| body.get("data").and_then(|d| d.get("hash")))
        .or_else(|| body.get("task_hash"))
        .and_then(|v| v.as_str())
        .map(|s| s.to_string())
}

struct ProgressEmit {
    file_name: String,
    bytes: u64,
    total_bytes: Option<u64>,
    started: Instant,
    session_bytes: u64,
    done: bool,
    failed: bool,
}

fn emit_progress(cb: &(dyn Fn(TransferProgress) + Send + Sync), event: ProgressEmit) {
    let elapsed = event.started.elapsed().as_secs_f64().max(0.001);
    let speed_bps = if event.done {
        0.0
    } else {
        event.session_bytes as f64 / elapsed
    };
    let percent = event
        .total_bytes
        .map(|total| {
            let total = total.max(1);
            (event.bytes as f32 / total as f32) * 100.0
        })
        .unwrap_or(0.0);
    cb(TransferProgress {
        file_name: event.file_name,
        bytes: event.bytes,
        total_bytes: event.total_bytes,
        speed_bps,
        percent: if event.done && !event.failed && event.total_bytes.is_some() {
            100.0
        } else {
            percent
        },
        done: event.done,
        failed: event.failed,
    });
}

/// Upload a file with async streaming and progress reporting.
///
/// The caller supplies the async `reqwest::Client`, so GUI adapters can reuse
/// their proxy-aware client without creating nested runtimes.
pub async fn upload_file_async(
    client: &reqwest::Client,
    url: &str,
    file_path: &Path,
    extra_fields: Vec<(String, String)>,
    cancel_token: Option<Arc<AtomicBool>>,
    progress_cb: impl Fn(TransferProgress) + Send + Sync + 'static,
) -> Result<String, TransferError> {
    let file_name = file_path
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("audio.wav")
        .to_string();
    let file = tokio::fs::File::open(file_path).await.map_err(|e| {
        TransferError::new(format!("open upload file failed: {}", e)).with_path(file_path)
    })?;
    let file_size = file
        .metadata()
        .await
        .map_err(|e| {
            TransferError::new(format!("read upload file metadata failed: {}", e))
                .with_path(file_path)
        })?
        .len();
    let uploaded = Arc::new(AtomicU64::new(0));
    let started = Instant::now();
    let last_emit = Arc::new(Mutex::new(Instant::now() - Duration::from_millis(120)));
    let progress_cb: Arc<dyn Fn(TransferProgress) + Send + Sync> = Arc::new(progress_cb);

    let stream_file_name = file_name.clone();
    let stream_uploaded = uploaded.clone();
    let stream_last_emit = last_emit.clone();
    let stream_progress_cb = progress_cb.clone();
    let stream_cancel = cancel_token.clone();
    let stream = ReaderStream::new(file).map(move |result| {
        if stream_cancel
            .as_ref()
            .is_some_and(|token| token.load(Ordering::SeqCst))
        {
            return Err(std::io::Error::new(
                std::io::ErrorKind::Interrupted,
                "Upload cancelled",
            ));
        }

        let chunk = result?;
        let chunk_len = chunk.len() as u64;
        let total_uploaded = stream_uploaded.fetch_add(chunk_len, Ordering::SeqCst) + chunk_len;
        let mut should_emit = false;
        if let Ok(mut last) = stream_last_emit.lock() {
            if last.elapsed().as_millis() >= 120 {
                *last = Instant::now();
                should_emit = true;
            }
        }
        if should_emit {
            emit_progress(
                stream_progress_cb.as_ref(),
                ProgressEmit {
                    file_name: stream_file_name.clone(),
                    bytes: total_uploaded,
                    total_bytes: Some(file_size),
                    started,
                    session_bytes: total_uploaded,
                    done: false,
                    failed: false,
                },
            );
        }
        Ok(chunk)
    });

    let part =
        reqwest::multipart::Part::stream_with_length(reqwest::Body::wrap_stream(stream), file_size)
            .file_name(file_name.clone())
            .mime_str("audio/*")
            .map_err(|e| {
                TransferError::new(format!("build upload multipart part failed: {}", e))
                    .with_url(url)
                    .with_path(file_path)
            })?;

    let mut form = reqwest::multipart::Form::new().part("audiofile", part);
    for (key, val) in extra_fields {
        form = form.text(key, val);
    }

    let response = client.post(url).multipart(form).send().await.map_err(|e| {
        if cancel_token
            .as_ref()
            .is_some_and(|token| token.load(Ordering::SeqCst))
        {
            TransferError::cancelled("Upload cancelled")
                .with_url(url)
                .with_path(file_path)
        } else {
            TransferError::new(format!("Upload failed: {}", e))
                .with_url(url)
                .with_path(file_path)
        }
    })?;

    let status = response.status();
    let status_code = status.as_u16();
    let body_text = response.text().await.map_err(|e| {
        TransferError::new(format!("read upload response body failed: {}", e))
            .with_url(url)
            .with_path(file_path)
            .with_http_status(status_code)
    })?;
    if !status.is_success() {
        return Err(
            TransferError::new(upload_error_message(status_code, &body_text))
                .with_url(url)
                .with_path(file_path)
                .with_http_status(status_code),
        );
    }

    let hash = extract_upload_hash(&body_text)
        .filter(|hash| !hash.is_empty())
        .ok_or_else(|| {
            TransferError::new("Failed to get task hash")
                .with_url(url)
                .with_path(file_path)
                .with_http_status(status_code)
        })?;

    let total_uploaded = uploaded.load(Ordering::SeqCst).max(file_size);
    emit_progress(
        progress_cb.as_ref(),
        ProgressEmit {
            file_name,
            bytes: total_uploaded,
            total_bytes: Some(file_size),
            started,
            session_bytes: total_uploaded,
            done: true,
            failed: false,
        },
    );

    Ok(hash)
}

/// 同步上传文件（内部创建 tokio runtime）
///
/// 提供代理支持的阻塞版本上传函数，适合非异步上下文使用。
///
/// # 参数
///
/// - `proxy_host`: 代理主机地址（仅在 `proxy_mode` 为 `"manual"` 时有效）
/// - `proxy_port`: 代理端口（仅在 `proxy_mode` 为 `"manual"` 时有效）
/// - `proxy_mode`: 代理模式（`"auto"`/`"manual"`/`"none"`）
/// - `url`: 上传目标 URL
/// - `file_path`: 本地文件路径
/// - `extra_fields`: 额外的表单字段（如 API token）
/// - `progress_cb`: 进度回调函数
///
/// # 返回
///
/// `anyhow::Result<String>` - 任务 Hash 或错误
///
/// # 示例
///
/// ```rust,no_run
/// use mvsep_api_tester::file_transfer;
///
/// let hash = file_transfer::upload_file(
///     "",
///     0,
///     "none",
///     "https://api.mvsep.com/upload",
///     std::path::Path::new("./song.mp3"),
///     vec![("api_token", "your-token".to_string())],
///     |p| println!("上传: {:.1}%", p.percent),
/// ).unwrap();
/// ```
pub fn upload_file(
    proxy_host: &str,
    proxy_port: u16,
    proxy_mode: &str,
    url: &str,
    file_path: &Path,
    extra_fields: Vec<(String, String)>,
    progress_cb: impl Fn(TransferProgress) + Send + Sync + 'static,
) -> anyhow::Result<String> {
    let runtime = match tokio::runtime::Runtime::new() {
        Ok(r) => r,
        Err(e) => anyhow::bail!("Failed to start runtime: {}", e),
    };

    let mut builder = reqwest::Client::builder().timeout(Duration::from_secs(300));
    if proxy_mode == "manual" {
        let p = format!("http://{}:{}", proxy_host, proxy_port);
        if let Ok(proxy) = reqwest::Proxy::all(&p) {
            builder = builder.proxy(proxy);
        }
    } else if proxy_mode == "none" {
        builder = builder.no_proxy();
    }
    let client = builder.build()?;

    runtime
        .block_on(upload_file_async(
            &client,
            url,
            file_path,
            extra_fields,
            None,
            progress_cb,
        ))
        .map_err(anyhow::Error::new)
}

// ── Download ──

/// Download a file with async streaming, automatic resume support, cancellation,
/// and progress reporting.
///
/// Cancellation returns an error containing `Download cancelled` and leaves the
/// `.part` file plus metadata in place so a later call can resume.
pub async fn download_file_async(
    client: &reqwest::Client,
    file_url: &str,
    dest_path: &Path,
    remote_file_name: &str,
    cancel_token: Option<Arc<AtomicBool>>,
    progress_cb: impl Fn(TransferProgress) + Send + Sync + 'static,
) -> Result<(), TransferError> {
    let part = part_path(dest_path);
    let meta = meta_path(&part);

    if let Some(parent) = part.parent() {
        tokio::fs::create_dir_all(parent).await.map_err(|e| {
            TransferError::new(format!("create partial download directory failed: {}", e))
                .with_url(file_url)
                .with_path(parent)
        })?;
    }
    if let Some(parent) = meta.parent() {
        tokio::fs::create_dir_all(parent).await.map_err(|e| {
            TransferError::new(format!("create partial metadata directory failed: {}", e))
                .with_url(file_url)
                .with_path(parent)
        })?;
    }

    let mut resume_from = get_resume_info_async(dest_path, file_url).await;
    let pm = PartialDownloadMeta {
        file_url: file_url.to_string(),
        remote_file_name: remote_file_name.to_string(),
        updated_at: now_timestamp(),
    };
    save_partial_download_meta_async(&meta, &pm).await?;

    if cancel_token
        .as_ref()
        .is_some_and(|token| token.load(Ordering::SeqCst))
    {
        return Err(TransferError::cancelled("Download cancelled")
            .with_url(file_url)
            .with_path(dest_path));
    }

    let mut request = client.get(file_url);
    if resume_from > 0 {
        request = request.header(reqwest::header::RANGE, format!("bytes={}-", resume_from));
    }

    let response = request.send().await.map_err(|e| {
        TransferError::new(format!("Download request failed: {}", e))
            .with_url(file_url)
            .with_path(dest_path)
    })?;
    let status = response.status();
    let (append_mode, total_bytes) =
        if resume_from > 0 && status == reqwest::StatusCode::PARTIAL_CONTENT {
            let total = parse_total_bytes_from_headers(response.headers())
                .or_else(|| response.content_length().map(|len| resume_from + len));
            (true, total)
        } else if status.is_success() {
            if resume_from > 0 {
                resume_from = 0;
                let _ = tokio::fs::remove_file(&part).await;
            }
            (false, response.content_length())
        } else {
            return Err(
                TransferError::new(format!("HTTP {} while downloading file", status))
                    .with_url(file_url)
                    .with_path(dest_path)
                    .with_http_status(status.as_u16()),
            );
        };

    let mut options = tokio::fs::OpenOptions::new();
    options.create(true).write(true);
    if append_mode {
        options.append(true);
    } else {
        options.truncate(true);
    }
    let mut file = options.open(&part).await.map_err(|e| {
        TransferError::new(format!("open partial download file failed: {}", e))
            .with_url(file_url)
            .with_path(&part)
    })?;

    let progress_cb: Arc<dyn Fn(TransferProgress) + Send + Sync> = Arc::new(progress_cb);
    let file_name = dest_path
        .file_name()
        .and_then(|s| s.to_str())
        .unwrap_or("output")
        .to_string();
    let mut stream = response.bytes_stream();
    let mut downloaded: u64 = resume_from;
    let mut session_downloaded: u64 = 0;
    let started = Instant::now();
    let mut last_emit = Instant::now() - Duration::from_millis(150);

    while let Some(chunk) = stream.next().await {
        let chunk = chunk.map_err(|e| {
            TransferError::new(format!("read download response chunk failed: {}", e))
                .with_url(file_url)
                .with_path(dest_path)
        })?;
        file.write_all(&chunk).await.map_err(|e| {
            TransferError::new(format!("write partial download file failed: {}", e))
                .with_url(file_url)
                .with_path(&part)
        })?;
        let chunk_len = chunk.len() as u64;
        downloaded += chunk_len;
        session_downloaded += chunk_len;

        if last_emit.elapsed().as_millis() >= 150 {
            emit_progress(
                progress_cb.as_ref(),
                ProgressEmit {
                    file_name: file_name.clone(),
                    bytes: downloaded,
                    total_bytes,
                    started,
                    session_bytes: session_downloaded,
                    done: false,
                    failed: false,
                },
            );
            last_emit = Instant::now();
        }

        if cancel_token
            .as_ref()
            .is_some_and(|token| token.load(Ordering::SeqCst))
        {
            file.flush().await.map_err(|e| {
                TransferError::new(format!("flush partial download file failed: {}", e))
                    .with_url(file_url)
                    .with_path(&part)
            })?;
            return Err(TransferError::cancelled("Download cancelled")
                .with_url(file_url)
                .with_path(dest_path));
        }
    }

    file.flush().await.map_err(|e| {
        TransferError::new(format!("flush partial download file failed: {}", e))
            .with_url(file_url)
            .with_path(&part)
    })?;

    if dest_path.exists() {
        tokio::fs::remove_file(dest_path).await.map_err(|e| {
            TransferError::new(format!("remove existing output file failed: {}", e))
                .with_url(file_url)
                .with_path(dest_path)
        })?;
    }
    tokio::fs::rename(&part, dest_path).await.map_err(|e| {
        TransferError::new(format!("finalize downloaded file failed: {}", e))
            .with_url(file_url)
            .with_path(dest_path)
    })?;
    let _ = tokio::fs::remove_file(&meta).await;

    emit_progress(
        progress_cb.as_ref(),
        ProgressEmit {
            file_name,
            bytes: downloaded,
            total_bytes,
            started,
            session_bytes: session_downloaded,
            done: true,
            failed: false,
        },
    );

    Ok(())
}

/// Download a file with streaming, resume support, and progress reporting.
///
/// If `resume_from > 0`, sends a `Range` header and appends to the existing `.part` file.
/// Progress is reported every ~150ms during the download.
/// On completion, renames `.part` to the final path.
pub fn download_file(
    client: &reqwest::blocking::Client,
    file_url: &str,
    dest_path: &Path,
    resume_from: u64,
    progress_cb: impl Fn(TransferProgress) + Send,
) -> anyhow::Result<()> {
    let part = part_path(dest_path);
    let meta = meta_path(&part);

    // Save resume metadata
    if let Some(parent) = part.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let pm = PartialDownloadMeta {
        file_url: file_url.to_string(),
        remote_file_name: dest_path
            .file_name()
            .and_then(|s| s.to_str())
            .unwrap_or("output")
            .to_string(),
        updated_at: now_timestamp(),
    };
    if let Ok(content) = serde_json::to_string_pretty(&pm) {
        let _ = std::fs::write(&meta, &content);
    }

    // Build request with optional Range header
    let mut request = client.get(file_url);
    if resume_from > 0 {
        request = request.header(reqwest::header::RANGE, format!("bytes={}-", resume_from));
    }

    let response = request.send()?;
    let status = response.status();

    let (append_mode, total_bytes) =
        if resume_from > 0 && status == reqwest::StatusCode::PARTIAL_CONTENT {
            let total = parse_total_bytes_from_content_range(&response)
                .or_else(|| response.content_length().map(|len| resume_from + len));
            (true, total)
        } else if status.is_success() {
            if resume_from > 0 {
                // Server doesn't support resume, restart from scratch
            }
            let total = response.content_length();
            let _ = std::fs::remove_file(&part);
            (false, total)
        } else {
            anyhow::bail!("HTTP {} while downloading file", status);
        };

    // Open file for writing
    let mut file = std::fs::OpenOptions::new()
        .create(true)
        .write(true)
        .append(append_mode)
        .open(&part)?;

    // Stream the response body to disk with progress
    let mut bytes_stream = response.take(usize::MAX as u64);
    let mut downloaded: u64 = resume_from;
    let mut last_emit = Instant::now();
    let start = Instant::now();
    let mut session_downloaded: u64 = 0;
    let mut buf = [0u8; 65536]; // 64KB buffer

    let file_name = dest_path
        .file_name()
        .and_then(|s| s.to_str())
        .unwrap_or("output")
        .to_string();

    loop {
        let n = bytes_stream.read(&mut buf)?;
        if n == 0 {
            break;
        }
        file.write_all(&buf[..n])?;
        downloaded += n as u64;
        session_downloaded += n as u64;

        if last_emit.elapsed().as_millis() >= 150 {
            let elapsed = start.elapsed().as_secs_f64().max(0.001);
            let speed = session_downloaded as f64 / elapsed;
            let pct = total_bytes
                .map(|t| (downloaded as f32 / t.max(1) as f32) * 100.0)
                .unwrap_or(0.0);

            progress_cb(TransferProgress {
                file_name: file_name.clone(),
                bytes: downloaded,
                total_bytes,
                speed_bps: speed,
                percent: pct,
                done: false,
                failed: false,
            });
            last_emit = Instant::now();
        }
    }

    file.flush()?;

    // Rename .part → final file
    if dest_path.exists() {
        std::fs::remove_file(dest_path)?;
    }
    std::fs::rename(&part, dest_path)?;
    let _ = std::fs::remove_file(&meta);

    // Final progress report
    progress_cb(TransferProgress {
        file_name,
        bytes: downloaded,
        total_bytes,
        speed_bps: 0.0,
        percent: 100.0,
        done: true,
        failed: false,
    });

    Ok(())
}

fn parse_total_bytes_from_headers(headers: &reqwest::header::HeaderMap) -> Option<u64> {
    let value = headers.get(reqwest::header::CONTENT_RANGE)?;
    let raw = value.to_str().ok()?;
    let slash = raw.rfind('/')?;
    raw[(slash + 1)..].trim().parse::<u64>().ok()
}

/// Parse total bytes from the `Content-Range` response header.
fn parse_total_bytes_from_content_range(response: &reqwest::blocking::Response) -> Option<u64> {
    parse_total_bytes_from_headers(response.headers())
}

/// Check if a `.part` file exists and can be resumed.
/// Returns the number of bytes already downloaded.
pub fn get_resume_info(final_path: &Path, file_url: &str) -> u64 {
    let part = part_path(final_path);
    let meta = meta_path(&part);

    if !part.exists() {
        return 0;
    }
    let part_size = match std::fs::metadata(&part) {
        Ok(m) => m.len(),
        Err(_) => return 0,
    };
    if part_size == 0 {
        return 0;
    }

    // Check meta file matches the URL
    let can_resume = match std::fs::read_to_string(&meta) {
        Ok(content) => match serde_json::from_str::<PartialDownloadMeta>(&content) {
            Ok(pm) => pm.file_url == file_url,
            Err(_) => true, // If we can't parse meta, try resuming anyway
        },
        Err(_) => true,
    };

    if can_resume {
        part_size
    } else {
        let _ = std::fs::remove_file(&part);
        let _ = std::fs::remove_file(&meta);
        0
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::net::{TcpListener, TcpStream};
    use std::sync::mpsc::{self, Receiver};
    use std::thread;
    use std::time::Duration;

    fn test_client() -> reqwest::Client {
        reqwest::Client::builder().no_proxy().build().unwrap()
    }

    fn read_http_request(stream: &mut TcpStream) -> String {
        let mut raw = Vec::new();
        let mut byte = [0_u8; 1];
        while !raw.ends_with(b"\r\n\r\n") {
            stream.read_exact(&mut byte).unwrap();
            raw.push(byte[0]);
        }
        let header = String::from_utf8_lossy(&raw).to_string();
        let content_length = header
            .lines()
            .find_map(|line| {
                let (name, value) = line.split_once(':')?;
                if name.eq_ignore_ascii_case("content-length") {
                    value.trim().parse::<usize>().ok()
                } else {
                    None
                }
            })
            .unwrap_or(0);
        let mut body = vec![0_u8; content_length];
        if content_length > 0 {
            stream.read_exact(&mut body).unwrap();
        }
        format!("{}{}", header, String::from_utf8_lossy(&body))
    }

    fn write_response(
        stream: &mut TcpStream,
        status: &str,
        headers: &[(&str, String)],
        body: &[u8],
    ) {
        let mut response = format!("HTTP/1.1 {}\r\nContent-Length: {}\r\n", status, body.len());
        for (name, value) in headers {
            response.push_str(name);
            response.push_str(": ");
            response.push_str(value);
            response.push_str("\r\n");
        }
        response.push_str("\r\n");
        stream.write_all(response.as_bytes()).unwrap();
        stream.write_all(body).unwrap();
        stream.flush().unwrap();
    }

    fn spawn_server(
        handler: impl FnOnce(String, TcpStream) + Send + 'static,
    ) -> (String, Receiver<String>, thread::JoinHandle<()>) {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let addr = listener.local_addr().unwrap();
        let (tx, rx) = mpsc::channel();
        let handle = thread::spawn(move || {
            let (mut stream, _) = listener.accept().unwrap();
            let request = read_http_request(&mut stream);
            tx.send(request.clone()).unwrap();
            handler(request, stream);
        });
        (format!("http://{}", addr), rx, handle)
    }

    #[test]
    fn async_upload_extracts_hash_from_mock_http_response() {
        let (base_url, rx, handle) = spawn_server(|request, mut stream| {
            assert!(request.starts_with("POST "));
            assert!(request.contains("audiofile"));
            assert!(request.contains("api_token"));
            write_response(
                &mut stream,
                "200 OK",
                &[("Content-Type", "application/json".to_string())],
                br#"{"data":{"hash":"hash-from-upload"}}"#,
            );
        });
        let temp = tempfile::tempdir().unwrap();
        let file_path = temp.path().join("song.wav");
        std::fs::write(&file_path, b"audio-bytes").unwrap();
        let progress = Arc::new(Mutex::new(Vec::<TransferProgress>::new()));
        let progress_for_cb = progress.clone();

        let hash = tokio::runtime::Runtime::new()
            .unwrap()
            .block_on(upload_file_async(
                &test_client(),
                &format!("{}/upload", base_url),
                &file_path,
                vec![("api_token".to_string(), "token".to_string())],
                None,
                move |p| progress_for_cb.lock().unwrap().push(p),
            ));

        assert_eq!(hash.unwrap(), "hash-from-upload");
        assert!(rx.recv().unwrap().contains("audio-bytes"));
        handle.join().unwrap();
        assert!(progress.lock().unwrap().iter().any(|p| p.done && !p.failed));
    }

    #[test]
    fn async_download_resumes_with_range_header() {
        let (base_url, rx, handle) = spawn_server(|_request, mut stream| {
            write_response(
                &mut stream,
                "206 Partial Content",
                &[("Content-Range", "bytes 6-10/11".to_string())],
                b"world",
            );
        });
        let file_url = format!("{}/file.wav", base_url);
        let temp = tempfile::tempdir().unwrap();
        let dest_path = temp.path().join("song_vocals.wav");
        std::fs::write(part_path(&dest_path), b"hello ").unwrap();
        let meta = PartialDownloadMeta {
            file_url: file_url.clone(),
            remote_file_name: "remote.wav".to_string(),
            updated_at: now_timestamp(),
        };
        std::fs::write(
            meta_path(&part_path(&dest_path)),
            serde_json::to_string(&meta).unwrap(),
        )
        .unwrap();

        tokio::runtime::Runtime::new()
            .unwrap()
            .block_on(download_file_async(
                &test_client(),
                &file_url,
                &dest_path,
                "remote.wav",
                None,
                |_| {},
            ))
            .unwrap();

        let request = rx.recv().unwrap();
        handle.join().unwrap();
        assert!(request.to_ascii_lowercase().contains("range: bytes=6-"));
        assert_eq!(std::fs::read(&dest_path).unwrap(), b"hello world");
        assert!(!part_path(&dest_path).exists());
        assert!(!meta_path(&part_path(&dest_path)).exists());
    }

    #[test]
    fn async_download_restarts_when_server_rejects_range() {
        let (base_url, rx, handle) = spawn_server(|_request, mut stream| {
            write_response(&mut stream, "200 OK", &[], b"complete-file");
        });
        let file_url = format!("{}/file.wav", base_url);
        let temp = tempfile::tempdir().unwrap();
        let dest_path = temp.path().join("song_vocals.wav");
        std::fs::write(part_path(&dest_path), b"partial").unwrap();
        let meta = PartialDownloadMeta {
            file_url: file_url.clone(),
            remote_file_name: "remote.wav".to_string(),
            updated_at: now_timestamp(),
        };
        std::fs::write(
            meta_path(&part_path(&dest_path)),
            serde_json::to_string(&meta).unwrap(),
        )
        .unwrap();

        tokio::runtime::Runtime::new()
            .unwrap()
            .block_on(download_file_async(
                &test_client(),
                &file_url,
                &dest_path,
                "remote.wav",
                None,
                |_| {},
            ))
            .unwrap();

        let request = rx.recv().unwrap();
        handle.join().unwrap();
        assert!(request.to_ascii_lowercase().contains("range: bytes=7-"));
        assert_eq!(std::fs::read(&dest_path).unwrap(), b"complete-file");
    }

    #[test]
    fn async_download_cancel_keeps_resumable_partial_files() {
        let (base_url, _rx, handle) = spawn_server(|_request, mut stream| {
            let headers = "HTTP/1.1 200 OK\r\nContent-Length: 12\r\n\r\n";
            stream.write_all(headers.as_bytes()).unwrap();
            stream.write_all(b"hello ").unwrap();
            stream.flush().unwrap();
            thread::sleep(Duration::from_millis(250));
            let _ = stream.write_all(b"world!");
            let _ = stream.flush();
        });
        let file_url = format!("{}/file.wav", base_url);
        let temp = tempfile::tempdir().unwrap();
        let dest_path = temp.path().join("song_vocals.wav");
        let cancel = Arc::new(AtomicBool::new(false));
        let cancel_for_cb = cancel.clone();

        let result = tokio::runtime::Runtime::new()
            .unwrap()
            .block_on(download_file_async(
                &test_client(),
                &file_url,
                &dest_path,
                "remote.wav",
                Some(cancel),
                move |p| {
                    if !p.done {
                        cancel_for_cb.store(true, Ordering::SeqCst);
                    }
                },
            ));

        assert!(result
            .unwrap_err()
            .to_string()
            .to_lowercase()
            .contains("download cancelled"));
        handle.join().unwrap();
        assert!(part_path(&dest_path).exists());
        assert!(meta_path(&part_path(&dest_path)).exists());
        assert_eq!(std::fs::read(part_path(&dest_path)).unwrap(), b"hello ");
        assert!(!dest_path.exists());
    }
}
