use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::time::Instant;
use futures_util::StreamExt;
use tokio_util::io::ReaderStream;

/// Progress information for upload/download operations.
#[derive(Debug, Clone)]
pub struct TransferProgress {
    pub file_name: String,
    pub bytes: u64,
    pub total_bytes: Option<u64>,
    pub speed_bps: f64,
    pub percent: f32,
    pub done: bool,
    pub failed: bool,
}

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

/// Upload a file with progress reporting.
///
/// Reads the file with a progress-tracking wrapper, then sends it
/// as a multipart request. Progress is reported during the read phase.
/// Async implementation of the upload logic.
async fn upload_async(
    file_path: &Path,
    file_name: &str,
    file_size: u64,
    proxy_host: &str,
    proxy_port: u16,
    proxy_mode: &str,
    url: &str,
    extra_fields: Vec<(String, String)>,
    progress_cb: impl Fn(TransferProgress) + Send + 'static,
) -> Result<(u16, String), String> {
    let mut builder = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(300));
    if proxy_mode == "manual" {
        let p = format!("http://{}:{}", proxy_host, proxy_port);
        if let Ok(proxy) = reqwest::Proxy::all(&p) {
            builder = builder.proxy(proxy);
        }
    } else if proxy_mode == "none" {
        builder = builder.no_proxy();
    }
    let client = builder.build().map_err(|e| format!("{}", e))?;

    let file = tokio::fs::File::open(file_path).await
        .map_err(|e| format!("{}", e))?;

    let fname = file_name.to_string();
    let fsize = file_size;
    let cb = std::sync::Arc::new(std::sync::Mutex::new(Some(progress_cb)));
    let uploaded = std::sync::Arc::new(std::sync::atomic::AtomicU64::new(0));

    use futures_util::StreamExt;
    let stream = ReaderStream::new(file).map(move |result| {
        if let Ok(ref chunk) = result {
            let chunk_len = chunk.len() as u64;
            let total = uploaded.fetch_add(chunk_len, std::sync::atomic::Ordering::SeqCst) + chunk_len;
            let pct = if fsize > 0 { total as f32 / fsize as f32 * 100.0 } else { 0.0 };
            if let Some(cb_fn) = cb.lock().unwrap().as_ref() {
                cb_fn(TransferProgress {
                    file_name: fname.clone(),
                    bytes: total,
                    total_bytes: Some(fsize),
                    speed_bps: 0.0,
                    percent: pct,
                    done: false,
                    failed: false,
                });
            }
        }
        result
    });

    let file_body = reqwest::Body::wrap_stream(stream);
    let part = reqwest::multipart::Part::stream(file_body)
        .file_name(file_name.to_string())
        .mime_str("audio/*")
        .map_err(|e| format!("{}", e))?;

    let mut form = reqwest::multipart::Form::new().part("audiofile", part);
    for (key, val) in extra_fields {
        form = form.text(key, val);
    }

    let response = client.post(url)
        .multipart(form)
        .send()
        .await
        .map_err(|e| format!("Upload failed: {}", e))?;

    let status = response.status();
    let body_text = response.text().await.map_err(|e| format!("{}", e))?;

    Ok((status.as_u16(), body_text))
}

pub fn upload_file(
    proxy_host: &str,
    proxy_port: u16,
    proxy_mode: &str,
    url: &str,
    file_path: &Path,
    extra_fields: Vec<(String, String)>,
    progress_cb: impl Fn(TransferProgress) + Send + 'static,
) -> anyhow::Result<String> {
    let file_name = file_path
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("audio.wav")
        .to_string();
    let file_size = std::fs::metadata(file_path)?.len();

    let file_path = file_path.to_path_buf();
    let url = url.to_string();

    let runtime = match tokio::runtime::Runtime::new() {
        Ok(r) => r,
        Err(e) => anyhow::bail!("Failed to start runtime: {}", e),
    };
    let result = runtime.block_on(upload_async(
        &file_path, &file_name, file_size,
        proxy_host, proxy_port, proxy_mode,
        &url, extra_fields, progress_cb,
    ));
    let (status_code, body_text) = match result {
        Ok(r) => r,
        Err(e) => anyhow::bail!("Upload failed: {}", e),
    };

    if status_code != 200 {
        let msg = serde_json::from_str::<serde_json::Value>(&body_text)
            .ok()
            .and_then(|v| {
                v.get("errors").and_then(|e| e.as_array())
                    .map(|a| a.iter().filter_map(|v| v.as_str()).collect::<Vec<_>>().join(", "))
                    .or_else(|| v.get("message").and_then(|v| v.as_str()).map(|s| s.to_string()))
            })
            .unwrap_or_else(|| body_text.clone());
        anyhow::bail!("HTTP {} - {}", status_code, msg);
    }

    let body: serde_json::Value = serde_json::from_str(&body_text)
        .unwrap_or(serde_json::Value::Null);
    let hash = body.get("hash")
        .or_else(|| body.get("data").and_then(|d| d.get("hash")))
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string();

    Ok(hash)
}

// ── Download ──

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
        request = request.header(
            reqwest::header::RANGE,
            format!("bytes={}-", resume_from),
        );
    }

    let response = request.send()?;
    let status = response.status();

    let (append_mode, total_bytes) = if resume_from > 0 && status == reqwest::StatusCode::PARTIAL_CONTENT {
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

/// Parse total bytes from the `Content-Range` response header.
fn parse_total_bytes_from_content_range(response: &reqwest::blocking::Response) -> Option<u64> {
    let value = response.headers().get(reqwest::header::CONTENT_RANGE)?;
    let raw = value.to_str().ok()?;
    let slash = raw.rfind('/')?;
    raw[(slash + 1)..].trim().parse::<u64>().ok()
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
