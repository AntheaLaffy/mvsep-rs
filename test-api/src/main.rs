use std::io::{self, Write};
use std::path::PathBuf;
use std::time::Instant;
use colored::Colorize;
use serde_json::Value;

mod db;
mod file_transfer;
mod utils;

const VERSION: &str = "0.4";
const DEFAULT_API_URL: &str = "https://mvsep.com";

struct App {
    token: Option<String>,
    token_valid: bool,
    premium_enabled: bool,
    long_filenames: bool,
    api_url: String,
    proxy_host: String,
    proxy_port: u16,
    proxy_mode: String,
    output_dir: PathBuf,
    output_format: i32,
}

fn main() -> anyhow::Result<()> {
    utils::console::init();

    println!("{}", "╔═══════════════════════════════════════════╗".cyan());
    println!("{}", format!("║       MVSep API Tester v{}         ║", VERSION).cyan().bold());
    println!("{}", "╚═══════════════════════════════════════════╝".cyan());
    println!();

    utils::paths::ensure_data_dir()?;
    let db_path = utils::paths::db_path();
    let db = db::Database::new(Some(&db_path.to_string_lossy()))?;
    let mut app = load_app_config(&db);

    // Init default output formats if the table is empty
    {
        let conn = db.conn.lock().map_err(|e| anyhow::anyhow!("DB lock: {}", e))?;
        let count = db::repositories::get_all_output_formats(&conn)
            .map(|f| f.len())
            .unwrap_or(0);
        if count == 0 {
            db::repositories::init_default_output_formats(&conn)?;
        }
    }

    // Verify token at startup
    if app.token.is_some() {
        match verify_token(&app) {
            Ok(true) => {
                app.token_valid = true;
                println!("{} Token verified", "✅".green());
            }
            Ok(false) => {
                println!("{} Token invalid or expired, please login again", "⚠️".yellow());
            }
            Err(e) => {
                println!("{} Token check failed: {}", "⚠️".yellow(), e);
            }
        }
    }

    // Check algorithm cache expiry at startup
    if let Err(e) = check_cache_and_prompt_refresh(&mut app, &db) {
        eprintln!("  ⚠️ Cache check: {}", e);
    }

    show_status(&app);

    loop {
        run_menu(&mut app, &db)?;
        println!();
    }
}

fn load_app_config(db: &db::Database) -> App {
    // Remote/server-side config from mvsep.db
    let remote = db
        .with_conn(db::repositories::get_config)
        .ok()
        .flatten()
        .unwrap_or_default();

    // User preferences from user_config.db
    let ucfg = open_user_config().ok();
    let u_str = |key: &str, default: &str| -> String {
        ucfg.as_ref()
            .and_then(|c| c.get_string(key).ok().flatten())
            .filter(|s| !s.is_empty())
            .unwrap_or_else(|| default.to_string())
    };
    let u_int = |key: &str, default: i32| -> i32 {
        ucfg.as_ref()
            .and_then(|c| c.get_int(key).ok().flatten())
            .map(|v| v as i32)
            .unwrap_or(default)
    };

    let token = u_str("token", "");
    App {
        token: if token.is_empty() { None } else { Some(token) },
        token_valid: false,
        premium_enabled: u_int("premium_enabled", 0) != 0,
        long_filenames: u_int("long_filenames_enabled", 0) != 0,
        api_url: remote.api_url.unwrap_or_else(|| DEFAULT_API_URL.to_string()),
        proxy_host: u_str("proxy_host", "127.0.0.1"),
        proxy_port: u_str("proxy_port", "7897").parse().unwrap_or(7897),
        proxy_mode: u_str("proxy_mode", "system"),
        output_dir: PathBuf::from(u_str("output_dir", "./output")),
        output_format: u_int("output_format", 1),
    }
}

fn build_http_client(app: &App) -> anyhow::Result<reqwest::blocking::Client> {
    let mut builder = reqwest::blocking::Client::builder()
        .timeout(std::time::Duration::from_secs(60));

    match app.proxy_mode.as_str() {
        "manual" => {
            let proxy_url = format!("http://{}:{}", app.proxy_host, app.proxy_port);
            let proxy =
                reqwest::Proxy::all(&proxy_url).map_err(|e| anyhow::anyhow!("Proxy error: {}", e))?;
            builder = builder.proxy(proxy);
        }
        "none" => {
            builder = builder.no_proxy();
        }
        _ => {}
    }

    builder.build().map_err(|e| anyhow::anyhow!("HTTP client error: {}", e))
}

fn show_status(app: &App) {
    println!("{}", "══════════════════════════════════════════════════".cyan());
    println!("  {} v{}", "MVSep API Tester".yellow().bold(), VERSION);
    if let Some(t) = &app.token {
        let status = if app.token_valid { "Authenticated".green() } else { "Not verified".yellow() };
        let premium = if app.premium_enabled { " ⭐Premium" } else { "" };
        println!("  {} Token: {}..{} ({}){}", "🔑".cyan(), &t[..4].cyan(), "...".dimmed(), status, premium);
    } else {
        println!("  {} {} Token", "🔑".red(), "No".red().bold());
    }
    match app.proxy_mode.as_str() {
        "manual" => println!(
            "  🌐 Proxy: {}:{}",
            app.proxy_host.cyan(),
            app.proxy_port.to_string().cyan()
        ),
        "none" => println!("  🌐 Proxy: {} (direct)", "None".yellow()),
        _ => println!("  🌐 Proxy: System (auto)"),
    }
    println!("{}", "══════════════════════════════════════════════════".cyan());
}

fn run_menu(app: &mut App, db: &db::Database) -> anyhow::Result<()> {
    if app.token_valid {
        println!("  {} Logout", "[l]".yellow());
    } else {
        println!("  {} Login (need email+pass)", "[1]".cyan());
        println!("  {} Set Token manually", "[2]".cyan());
    }
    println!("  {} Configure Proxy", "[p]".cyan());
    println!("  {} User Preferences", "[c]".cyan());
    println!("  {} Create Task {} (need token)", "[3]".cyan().bold(), "⭐".yellow());
    println!("  {} List Tasks (from DB) ︎", "[t]".cyan());
    println!("  {} Operate Task (enter hash)", "[o]".cyan());
    println!("  {}", "──────────────".dimmed());
    println!("  {} Browse Algorithms (from DB)", "[b]".cyan());
    println!("  {} API Reference", "[h]".cyan());
    println!("  {} Get User Info (need token)", "[9]".cyan());
    println!("  {} Run All Tests (auto-detect mode)", "[a]".cyan());
    println!("  {} Refresh Algorithm Cache (from API)", "[r]".cyan());
    println!("  {} Quit", "[q]".red());
    print!("> ");
    io::stdout().flush()?;

    let mut input = String::new();
    io::stdin().read_line(&mut input)?;
    let choice = input.trim().to_lowercase();

    match choice.as_str() {
        "1" if !app.token_valid => cmd_login(app, db)?,
        "2" if !app.token_valid => cmd_set_token(app, db)?,
        "l" | "L" if app.token_valid => cmd_logout(app, db)?,
        "p" | "P" => cmd_proxy_config(app, db)?,
        "c" | "C" => cmd_user_prefs(app, db)?,
        "3" => cmd_create_task(app, db)?,
        "t" | "T" => cmd_list_tasks(app, db)?,
        "o" | "O" => cmd_operate_hash(app, db)?,
        // Hidden shortcuts (still work if typed):
        "s" | "S" => cmd_query_task(app)?,
        "4" => cmd_cancel_task(app, db)?,
        "9" => cmd_user_info(app)?,
        "b" | "B" => cmd_browse_algorithms(db)?,
        "h" | "H" | "?" => cmd_api_reference()?,
        "a" => cmd_run_all_tests(app, db)?,
        "r" | "R" => cmd_refresh_algorithms(app, db)?,
        "q" => {
            println!("{}", "Bye!".green());
            std::process::exit(0);
        }
        _ => println!("{} Unknown option: {}", "⚠️".yellow(), choice),
    }
    Ok(())
}

fn require_token(app: &App) -> anyhow::Result<&str> {
    app.token
        .as_deref()
        .ok_or_else(|| anyhow::anyhow!("No token set. Use option [2] to set a token first."))
}

fn find_audio_file() -> Option<PathBuf> {
    let candidates = [
        "螢塚-Calvaria.mp3",
        "tests/螢塚-Calvaria.mp3",
        "../test-api/螢塚-Calvaria.mp3",
        "../test-api/tests/螢塚-Calvaria.mp3",
        "test.mp3",
        "tests/test.mp3",
    ];
    for c in &candidates {
        let p = PathBuf::from(c);
        if p.exists() {
            return Some(p);
        }
    }
    None
}

fn prompt(prompt_text: &str, default: Option<&str>) -> anyhow::Result<String> {
    let default_str = default.unwrap_or("");
    if !default_str.is_empty() {
        print!("{} [{}]: ", prompt_text, default_str);
    } else {
        print!("{}: ", prompt_text);
    }
    io::stdout().flush()?;
    let mut input = String::new();
    io::stdin().read_line(&mut input)?;
    let trimmed = input.trim().to_string();
    if trimmed.is_empty() && !default_str.is_empty() {
        Ok(default_str.to_string())
    } else {
        Ok(trimmed)
    }
}

fn prompt_int(prompt_text: &str, default: i32) -> anyhow::Result<i32> {
    let s = prompt(prompt_text, Some(&default.to_string()))?;
    s.parse().map_err(|e| anyhow::anyhow!("Invalid number: {}", e))
}

fn prompt_bool(prompt_text: &str, default: bool) -> anyhow::Result<bool> {
    let d = if default { "y" } else { "n" };
    let s = prompt(prompt_text, Some(d))?;
    Ok(s == "y" || s == "Y" || s == "yes")
}

/// Verify a token by calling the user info endpoint.
/// Returns Ok(true) if the token is valid, Ok(false) if invalid/expired.
fn verify_token(app: &App) -> anyhow::Result<bool> {
    let token = match &app.token {
        Some(t) => t,
        None => return Ok(false),
    };
    let client = match build_http_client(app) {
        Ok(c) => c,
        Err(_) => return Ok(false),
    };
    let url = format!("{}/api/app/user?api_token={}", app.api_url, token);
    match client.get(&url).send() {
        Ok(resp) => Ok(resp.status().is_success()),
        Err(_) => Ok(false),
    }
}

/// Present interactive choices for algorithm fields (add_opt1/2/3).
/// Returns (opt1, opt2, opt3) — each is None when the field has no options
/// or the user selects the default.
fn prompt_algorithm_fields(fields: &[db::repositories::AlgorithmFieldRow]) -> (Option<i32>, Option<i32>, Option<i32>) {
    let mut opt1: Option<i32> = None;
    let mut opt2: Option<i32> = None;
    let mut opt3: Option<i32> = None;

    for field in fields {
        let field_name = field.name.as_str(); // "add_opt1", "add_opt2", "add_opt3"
        let label = field.text.as_deref().unwrap_or(field_name);
        let default_str = field.default_key.as_deref().unwrap_or("");

        // Parse options JSON
        let entries: Vec<(String, String)> = field
            .options
            .as_deref()
            .and_then(|opts| serde_json::from_str::<serde_json::Value>(opts).ok())
            .and_then(|v| v.as_object().cloned())
            .map(|map| {
                let mut v: Vec<(String, String)> = map
                    .into_iter()
                    .map(|(k, v)| (k, v.as_str().unwrap_or(&v.to_string()).to_string()))
                    .collect();
                v.sort_by_key(|a| a.0.parse::<i32>().unwrap_or(0));
                v
            })
            .unwrap_or_default();

        if entries.is_empty() {
            let raw = prompt_int(&format!("  {} (raw value, -1 to skip)", label), default_str.parse().unwrap_or(-1));
            if let Ok(v) = raw {
                if v >= 0 {
                    match field_name {
                        "add_opt1" => opt1 = Some(v),
                        "add_opt2" => opt2 = Some(v),
                        "add_opt3" => opt3 = Some(v),
                        _ => {}
                    }
                }
            }
            continue;
        }

        // Show numbered options
        println!("\n  {} — {}:", "🔧".cyan(), label.cyan());
        for (i, (key, desc)) in entries.iter().enumerate() {
            let marker = if *key == default_str { " (默认)" } else { "" };
            println!("    {}. [{}] {}{}", i + 1, key.yellow(), desc, marker.dimmed());
        }

        let chosen = prompt_int(&format!("  Select {} (1-{}, 0=default)", label, entries.len()), 0).unwrap_or(0);
        let idx = chosen as usize;
        let picked: Option<i32> = if chosen == 0 && !default_str.is_empty() {
            default_str.parse().ok()
        } else if idx >= 1 && idx <= entries.len() {
            entries[idx - 1].0.parse().ok()
        } else {
            None
        };

        if let Some(val) = picked {
            println!("    → Selected: {} = {}", val, entries.iter().find(|(k, _)| k == &val.to_string()).map(|(_, d)| d.as_str()).unwrap_or("?"));
            match field_name {
                "add_opt1" => opt1 = Some(val),
                "add_opt2" => opt2 = Some(val),
                "add_opt3" => opt3 = Some(val),
                _ => {}
            }
        }
    }

    (opt1, opt2, opt3)
}

// ── Algorithm Cache ──

/// Open the user config database for cache metadata
fn open_user_config() -> anyhow::Result<db::user_config::UserConfigDB> {
    let path = utils::paths::user_config_path();
    utils::paths::ensure_data_dir()?;
    db::user_config::UserConfigDB::new(&path.to_string_lossy())
}

/// Check whether the algorithm cache is expired.
/// Returns true if: no last_fetched timestamp, or current time > last_fetched + refresh_days.
fn is_cache_expired() -> bool {
    let ucfg = match open_user_config() {
        Ok(c) => c,
        Err(_) => return true,
    };
    let last_fetched_secs: i64 = match ucfg.get_int("algorithm_last_fetched_at") {
        Ok(Some(v)) => v,
        _ => return true,
    };
    let refresh_days: i64 = match ucfg.get_int("algorithm_auto_refresh_days") {
        Ok(Some(d)) if d > 0 => d,
        _ => 15,
    };

    let now_secs = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0);
    let elapsed_secs = now_secs - last_fetched_secs;
    let elapsed_days = elapsed_secs / 86400;
    elapsed_days >= refresh_days
}

/// Update the cache timestamp in user_config to now.
fn update_cache_timestamp() {
    if let Ok(ucfg) = open_user_config() {
        let now_secs = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_secs() as i64)
            .unwrap_or(0);
        let _ = ucfg.set_int("algorithm_last_fetched_at", now_secs);
        let _ = ucfg.set_int("algorithm_auto_refresh_days", 15);
    }
}

/// Check cache at startup and prompt user to refresh if expired.
fn check_cache_and_prompt_refresh(app: &mut App, db: &db::Database) -> anyhow::Result<()> {
    if !is_cache_expired() {
        return Ok(());
    }
    println!("{} Algorithm cache is expired or empty.", "ℹ️".yellow());
    let do_refresh = prompt_bool("Refresh from API now?", true)?;
    if do_refresh {
        cmd_refresh_algorithms(app, db)?;
    }
    Ok(())
}

/// Fetch algorithm list from API and cache it in the database.
/// This stores: algorithm_groups, algorithms, algorithm_fields (with options),
/// and algorithm_output_formats associations.
fn fetch_and_cache_algorithms(app: &App, db: &db::Database) -> anyhow::Result<usize> {
    println!("  🌐 Fetching algorithm list from {}...", app.api_url.cyan());

    let client = build_http_client(app)?;
    let url = format!("{}/api/app/algorithms?scopes=single_upload", app.api_url);

    let resp = client.get(&url).send().map_err(|e| {
        anyhow::anyhow!("API request failed: {}", e)
    })?;

    if !resp.status().is_success() {
        anyhow::bail!("API returned HTTP {}", resp.status());
    }

    let algorithms: Vec<serde_json::Value> = resp.json().map_err(|e| {
        anyhow::anyhow!("Failed to parse API response: {}", e)
    })?;

    let count = algorithms.len();
    println!("  ✅ Received {} algorithms from API", count);

    let conn = db.conn.lock().map_err(|e| anyhow::anyhow!("DB lock: {}", e))?;

    // Process each algorithm
    for algo in &algorithms {
        let algo_id = algo.get("render_id")
            .and_then(|v| v.as_i64())
            .unwrap_or(0) as i32;
        let algo_name = algo.get("name")
            .and_then(|v| v.as_str())
            .unwrap_or("Unknown")
            .to_string();
        let group_id = algo.get("algorithm_group")
            .and_then(|g| g.get("id"))
            .and_then(|v| v.as_i64())
            .unwrap_or(0) as i32;
        let group_name = algo.get("algorithm_group")
            .and_then(|g| g.get("name"))
            .and_then(|v| v.as_str())
            .unwrap_or("Ungrouped")
            .to_string();

        // Upsert algorithm group
        let _ = conn.execute(
            "INSERT OR IGNORE INTO algorithm_groups (id, name) VALUES (?1, ?2)",
            rusqlite::params![group_id, group_name],
        );

        // Read actual orientation from API
        let orientation = algo.get("orientation").and_then(|v| v.as_i64()).unwrap_or(0) as i32;

        // Upsert algorithm
        let _ = conn.execute(
            "INSERT OR REPLACE INTO algorithms (id, name, group_id, price_coefficient, orientation) VALUES (?1, ?2, ?3, ?4, ?5)",
            rusqlite::params![algo_id, algo_name, group_id, 1.0, orientation],
        );

        // Process algorithm_fields
        if let Some(fields) = algo.get("algorithm_fields").and_then(|f| f.as_array()) {
            for field in fields {
                let field_id = field.get("id").and_then(|v| v.as_i64()).unwrap_or(0);
                let field_name = field.get("name").and_then(|v| v.as_str()).unwrap_or("").to_string();
                let field_text = field.get("text").and_then(|v| v.as_str()).unwrap_or("").to_string();
                let field_options = field.get("options").and_then(|v| v.as_str()).unwrap_or("{}").to_string();
                let field_default = field.get("default_key").and_then(|v| v.as_str()).unwrap_or("").to_string();

                let _ = conn.execute(
                    "INSERT OR REPLACE INTO algorithm_fields (id, algorithm_id, name, text, options, default_key) VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
                    rusqlite::params![field_id, algo_id, field_name, field_text, field_options, field_default],
                );
            }
        }
    }

    // Init default output formats if empty, then associate all formats with all algorithms
    let fmt_count = db::repositories::get_all_output_formats(&conn)
        .map(|f| f.len())
        .unwrap_or(0);
    if fmt_count == 0 {
        db::repositories::init_default_output_formats(&conn)?;
    }
    let _ = db::repositories::init_default_algorithm_format_associations(&conn);

    drop(conn);
    update_cache_timestamp();
    println!("  ✅ Cached {} algorithms to database", count);
    Ok(count)
}

/// Menu command: force-refresh algorithm cache from API.
fn cmd_refresh_algorithms(app: &App, db: &db::Database) -> anyhow::Result<()> {
    match fetch_and_cache_algorithms(app, db) {
        Ok(count) => println!("{} Algorithm cache refreshed ({} algorithms)", "✅".green(), count),
        Err(e) => println!("{} Refresh failed: {}", "❌".red(), e),
    }
    Ok(())
}

/// Menu command: browse all algorithms stored in the database, grouped by group.
fn cmd_browse_algorithms(db: &db::Database) -> anyhow::Result<()> {
    let conn = match db.conn.lock() {
        Ok(c) => c,
        Err(_) => {
            println!("{} Failed to access database", "❌".red());
            return Ok(());
        }
    };

    let groups = match db::repositories::get_all_algorithm_groups(&conn) {
        Ok(g) => g,
        Err(e) => {
            println!("{} Failed to load groups: {}", "❌".red(), e);
            return Ok(());
        }
    };

    let algorithms = match db::repositories::get_all_algorithms(&conn) {
        Ok(a) => a,
        Err(e) => {
            println!("{} Failed to load algorithms: {}", "❌".red(), e);
            return Ok(());
        }
    };

    if algorithms.is_empty() {
        println!("  ℹ️ No algorithms in database. Use [r] to refresh from API.");
        return Ok(());
    }

    println!("\n{}", "═══ Algorithm List ═══".cyan().bold());
    println!("  Total: {} algorithms in {} groups\n", algorithms.len().to_string().cyan(), groups.len().to_string().cyan());

    for group in &groups {
        let algos: Vec<_> = algorithms.iter().filter(|a| a.group_id == group.id).collect();
        if algos.is_empty() {
            continue;
        }
        println!("{}", format!("  ▸ {} (ID: {})", group.name, group.id).yellow());
        for algo in &algos {
            let badge = match algo.orientation {
                0 => "  ".to_string(),
                1 => " 🔸".yellow().to_string(),
                _ => " 🔒 Premium".red().to_string(),
            };
            println!("      ID {:>4}  {}{}", algo.id.to_string().cyan(), algo.name, badge);
        }
        println!();
    }

    println!("{}", "═══ End ═══".cyan().bold());
    Ok(())
}

/// Menu command: display all available API endpoints categorized by type.
fn cmd_api_reference() -> anyhow::Result<()> {
    println!("\n{}", "╔══════════════════════════════════════════════════════╗".cyan());
    println!("{}", "║              API Reference                          ║".cyan().bold());
    println!("{}", "╚══════════════════════════════════════════════════════╝".cyan());

    // ── Remote APIs ──
    println!("\n{}", "🌐 Remote APIs (MVSep Server)".green().bold());

    println!("\n  {} {}", "── Account & Auth ──".yellow(), "(操作)".dimmed());
    println!("    POST  /api/app/login");
    println!("    GET   /api/app/user");

    println!("\n  {} {}", "── Separation Tasks ──".yellow(), "(操作)".dimmed());
    println!("    POST  /api/separation/create      Create separation task");
    println!("    GET   /api/separation/get           Query task status");
    println!("    POST  /api/separation/cancel        Cancel task");
    println!("    GET   /api/app/separation_history   Task history");

    println!("\n  {} {}", "── Queue ──".yellow(), "(查询)".dimmed());
    println!("    GET   /api/app/queue                Queue status");

    println!("\n  {} {}", "── Algorithms ──".yellow(), "(查询)".dimmed());
    println!("    GET   /api/app/algorithms           List + fields");
    println!("    GET   /api/output_formats           Output format list");

    println!("\n  {} {}", "── Settings Toggles ──".yellow(), "(操作)".dimmed());
    println!("    POST  /api/app/enable_premium          Enable premium mode");
    println!("    POST  /api/app/disable_premium         Disable premium mode");
    println!("    POST  /api/app/enable_long_filenames   Enable long filenames");
    println!("    POST  /api/app/disable_long_filenames  Disable long filenames");

    // ── Database APIs ──
    println!("\n{}", "────────────────────────────────────────────────────".dimmed());
    println!("{}", "🗄️  Database APIs (Local SQLite)".green().bold());

    println!("\n  {} {}", "── Config ──".yellow(), "(操作)".dimmed());
    println!("    save_config / get_config            User config (token, proxy, etc.)");
    println!("    UserConfigDB (kv store)              Cache metadata (last_fetched, etc.)");

    println!("\n  {} {}", "── Algorithm Cache ──".yellow(), "(查询 + 操作)".dimmed());
    println!("    algorithm_groups                    Groups (id, name)");
    println!("    algorithms                          Algorithms (id, name, group)");
    println!("    algorithm_fields                    Fields with options per algorithm");
    println!("    algorithm_output_formats            Algorithm ↔ Format associations");
    println!("    output_formats                      Format list with bit depth & premium");

    println!("\n  {} {}", "── Tasks ──".yellow(), "(操作)".dimmed());
    println!("    tasks                               Active tasks");
    println!("    task_history                        Completed tasks");

    println!("\n  {} {}", "── Presets ──".yellow(), "(操作)".dimmed());
    println!("    presets                             Saved presets (algo + options)");

    println!("\n{}", "╔══════════════════════════════════════════════════════╗".cyan());
    println!("{}", "║  [b] Browse Algo DB    [r] Refresh from API          ║".cyan());
    println!("{}", "║  [a] Run All Tests      [3] Create Task              ║".cyan());
    println!("{}", "╚══════════════════════════════════════════════════════╝".cyan());
    Ok(())
}

/// Menu command: list tracked tasks from DB with interactive selection.
fn cmd_list_tasks(app: &App, db: &db::Database) -> anyhow::Result<()> {
    let mut tasks = load_tasks_from_db(db)?;
    if tasks.is_empty() {
        println!("  ℹ️ No tasks in database.");
        return Ok(());
    }

    // Auto-poll any non-terminal tasks to refresh status
    let pending_hashes: Vec<String> = tasks.iter()
        .filter(|t| !matches!(t.status.as_str(), "done" | "expired" | "failed" | "cancelled"))
        .map(|t| t.hash.clone())
        .collect();
    if !pending_hashes.is_empty() {
        println!("  🔄 Auto-polling {} task(s)...", pending_hashes.len());
        for h in &pending_hashes {
            print!("    {} ... ", &h[..12.min(h.len())]);
            let _ = io::stdout().flush();
            if let Err(e) = cmd_poll_task(app, db, h) {
                println!("{}", e);
            } else {
                println!("✅");
            }
        }
        // Reload tasks after polling
        tasks = load_tasks_from_db(db)?;
        println!();
    }

    loop {
        print_tasks(&tasks);
        print!("  Select task number (0 to go back): ");
        io::stdout().flush()?;
        let mut input = String::new();
        io::stdin().read_line(&mut input)?;
        let idx: usize = match input.trim().parse() {
            Ok(n) => n,
            Err(_) => continue,
        };
        if idx == 0 {
            break;
        }
        if idx > tasks.len() {
            println!("{} Invalid task number", "⚠️".yellow());
            continue;
        }

        let task = &tasks[idx - 1];
        task_detail_menu(app, db, task)?;
    }
    Ok(())
}

fn load_tasks_from_db(db: &db::Database) -> anyhow::Result<Vec<db::repositories::TaskRow>> {
    let conn = match db.conn.lock() {
        Ok(c) => c,
        Err(_) => return Ok(vec![]),
    };
    db::repositories::get_all_tasks(&conn).map_err(|e| anyhow::anyhow!("{}", e))
}

fn print_tasks(tasks: &[db::repositories::TaskRow]) {
    println!("\n{}", "═══ Task History ═══".cyan().bold());
    for (i, task) in tasks.iter().enumerate() {
        let status_icon = match task.status.as_str() {
            "done" => "✅",
            "expired" => "⌛",
            "failed" => "❌",
            "cancelled" => "🚫",
            "queued" => "⏳",
            "processing" => "⚙️",
            _ => "⏳",
        };
        let phase_tag = match task.status.as_str() {
            "uploaded" => "📤已上传",
            "uploading" => "⬆️上传中",
            "queued" => "⏳排队中",
            "processing" => "⚙️分离中",
            "cancelled" => "❌已取消",
            "done" => "✅已完成",
            "expired" => "⌛已过期",
            "failed" => "❌失败",
            _ => "⏳处理中",
        };
        let short_hash = if task.hash.len() > 12 {
            format!("{}...", &task.hash[..12])
        } else {
            task.hash.clone()
        };
        println!("  {}. {} {} {} {}",
            (i + 1).to_string().cyan(),
            status_icon,
            phase_tag.dimmed(),
            short_hash.cyan(),
            task.file_name.dimmed(),
        );
    }
    println!("{}", "─────────────────────────".dimmed());
}

fn task_detail_menu(app: &App, db: &db::Database, task: &db::repositories::TaskRow) -> anyhow::Result<()> {
    loop {
        // Reload task from DB every iteration to pick up poll/cancel changes
        let current = db.conn.lock().ok()
            .and_then(|conn| db::repositories::get_task_by_hash(&conn, &task.hash).ok().flatten())
            .unwrap_or_else(|| task.clone());

        let status_label = match current.status.as_str() {
            "done" => "✅ 已完成".green(),
            "expired" => "⌛ 文件已过期".red(),
            "failed" => "❌ 失败".red(),
            "cancelled" => "🚫 已取消".red(),
            "queued" => "⏳ 排队中".yellow(),
            "processing" => "⚙️ 分离中".cyan(),
            "uploaded" => "📤 已上传".dimmed(),
            _ => current.status.dimmed(),
        };
        let is_terminal = matches!(current.status.as_str(), "done" | "expired" | "failed" | "cancelled");

        println!("\n{} {} {}", "── Task".cyan(), current.hash.cyan(), "──".cyan());
        println!("  File:     {}", current.file_name);
        println!("  Algo:     {} (ID: {})", current.algorithm_name, current.algorithm_id);
        println!("  Format:   {}", current.format);
        println!("  Status:   {} ({:.0}%)", status_label, current.progress);
        if current.status == "done" {
            println!("  {} Download results", "[d]".green());
        }
        if !is_terminal {
            println!("  {} Poll status (query API)", "[p]".cyan());
            println!("  {} Cancel task", "[c]".yellow());
        }
        println!("  {} Back to list", "[b]".dimmed());
        print!("> ");
        io::stdout().flush()?;

        let mut input = String::new();
        io::stdin().read_line(&mut input)?;
        match input.trim().to_lowercase().as_str() {
            "d" => cmd_download_task(app, &current.hash)?,
            "p" => cmd_poll_task(app, db, &current.hash)?,
            "c" => cmd_cancel_single(app, db, &current.hash)?,
            "b" => break,
            _ => println!("{} Unknown option", "⚠️".yellow()),
        }
    }
    Ok(())
}

/// Operate on a task by manually entering its hash.
fn cmd_operate_hash(app: &App, db: &db::Database) -> anyhow::Result<()> {
    let hash = prompt("Task hash", None)?;
    if hash.is_empty() {
        println!("⚠️ No hash entered");
        return Ok(());
    }

    // Auto-poll to refresh status
    print!("  🔄 Polling {} ... ", &hash[..12.min(hash.len())]);
    let _ = io::stdout().flush();
    let _ = cmd_poll_task(app, db, &hash);
    println!();

    // Try to find task in local DB first for better display
    let task_from_db = db.conn.lock().ok()
        .and_then(|conn| db::repositories::get_task_by_hash(&conn, &hash).ok().flatten());

    if let Some(task) = task_from_db {
        task_detail_menu(app, db, &task)?;
    } else {
        hash_detail_menu(app, db, &hash)?;
    }
    Ok(())
}

/// Detail sub-menu for a manual hash (not in DB).
fn hash_detail_menu(app: &App, db: &db::Database, hash: &str) -> anyhow::Result<()> {
    println!("\n{} {} {}", "── Task".cyan(), hash.cyan(), "──".cyan());
    println!("  (not in local database)");
    loop {
        println!();
        println!("  {} Poll status (query API)", "[p]".cyan());
        println!("  {} Download results", "[d]".green());
        println!("  {} Cancel task", "[c]".yellow());
        println!("  {} Back", "[b]".dimmed());
        print!("> ");
        io::stdout().flush()?;
        let mut input = String::new();
        io::stdin().read_line(&mut input)?;
        match input.trim().to_lowercase().as_str() {
            "p" => cmd_poll_task(app, db, hash)?,
            "d" => cmd_download_task(app, hash)?,
            "c" => cmd_cancel_single(app, db, hash)?,
            "b" => break,
            _ => println!("{} Unknown option", "⚠️".yellow()),
        }
    }
    Ok(())
}

fn cmd_poll_task(app: &App, db: &db::Database, hash: &str) -> anyhow::Result<()> {
    let token = match &app.token {
        Some(t) => t.clone(),
        None => {
            println!("{} No token", "❌".red());
            return Ok(());
        }
    };

    let client = match build_http_client(app) {
        Ok(c) => c,
        Err(e) => {
            println!("{} {}", "❌".red(), e);
            return Ok(());
        }
    };

    let url = format!("{}/api/separation/get", app.api_url);
    match client.get(&url).query(&[("hash", hash), ("api_token", &token)]).send() {
        Ok(resp) => {
            if !resp.status().is_success() {
                println!("❌ Poll failed (HTTP {})", resp.status());
                return Ok(());
            }
            let body: Value = resp.json().unwrap_or(Value::Null);
            let raw_status = body.get("status").and_then(|v| v.as_str()).unwrap_or("unknown").to_string();
            let queue_order = body.get("data").and_then(|d| d.get("current_order")).and_then(|v| v.as_i64()).unwrap_or(0);
            let queue_total = body.get("data").and_then(|d| d.get("queue_count")).and_then(|v| v.as_i64());
            let finished = body.get("data").and_then(|d| d.get("finished_chunks")).and_then(|v| v.as_i64()).unwrap_or(0);
            let total = body.get("data").and_then(|d| d.get("all_chunks")).and_then(|v| v.as_i64()).unwrap_or(0);
            let progress = if total > 0 { finished as f64 / total as f64 } else { 0.0 };

            // Refine status: if "processing" but still in queue → "queued"
            // If "done" but no files → files expired
            let has_files = body.get("data").and_then(|d| d.get("files")).and_then(|v| v.as_array()).map(|a| !a.is_empty()).unwrap_or(false);
            let refined_status = if raw_status == "processing" && queue_order > 0 {
                "queued".to_string()
            } else if raw_status == "done" && !has_files {
                "expired".to_string()
            } else {
                raw_status.clone()
            };

            // Sync DB with refined status + phase + progress
            if let Ok(conn) = db.conn.lock() {
                let _ = conn.execute(
                    "UPDATE tasks SET status = ?1, phase = ?1, progress = ?2 WHERE hash = ?3",
                    rusqlite::params![refined_status, progress, hash],
                );
                // Save output_files when task is done with files
                if raw_status == "done" && has_files {
                    if let Some(files_arr) = body.get("data").and_then(|d| d.get("files")).and_then(|v| v.as_array()) {
                        let file_list: Vec<serde_json::Value> = files_arr.iter().map(|f| {
                            let url = f.get("url").and_then(|v| v.as_str()).unwrap_or("");
                            let name = f.get("name").and_then(|v| v.as_str()).or_else(|| url.split('/').next_back()).unwrap_or("");
                            let size = f.get("size").and_then(|v| v.as_str()).and_then(|s| parse_size_mb(s)).unwrap_or(0);
                            serde_json::json!({
                                "remote_name": name,
                                "url": url,
                                "size": size,
                                "downloaded": false,
                            })
                        }).collect();
                        let json_str = serde_json::to_string(&file_list).unwrap_or_else(|_| "[]".to_string());
                        let _ = db::repositories::update_task_output_files(&conn, hash, &json_str);
                    }
                }
            }

            let msg = body.get("data").and_then(|d| d.get("message")).and_then(|v| v.as_str())
                .or_else(|| body.get("message").and_then(|v| v.as_str()))
                .unwrap_or("");

            let status_display = match refined_status.as_str() {
                "queued" => format!("⏳ 排队中 (第 {} 位)", queue_order).yellow(),
                "processing" => "⚙️ 分离中".cyan(),
                "done" => "✅ 已完成".green(),
                "expired" => "⌛ 文件已过期".red(),
                "failed" => "❌ 失败".red(),
                "cancelled" => "🚫 已取消".red(),
                _ => refined_status.dimmed(),
            };
            println!("  📊 {}", status_display);
            if !msg.is_empty() {
                println!("  💬 {}", msg);
            }
            if let Some(qt) = queue_total {
                if queue_order > 0 {
                    println!("  🚶 Queue: {}/{}", queue_order, qt);
                }
            }
            if total > 0 {
                println!("  📈 Progress: {:.1}% ({}/{})", progress * 100.0, finished, total);
            }
            if raw_status == "done" && has_files {
                println!("  {} Separation complete! Use [d] to download.", "✅".green());
            } else if raw_status == "done" && !has_files {
                println!("  {} Task files have expired (no longer available).", "⌛".red());
            } else if raw_status == "failed" {
                println!("  {} Task failed.", "❌".red());
            }
        }
        Err(e) => println!("❌ Poll failed: {}", e),
    }
    Ok(())
}

fn cmd_cancel_single(app: &App, db: &db::Database, hash: &str) -> anyhow::Result<()> {
    let token = match &app.token {
        Some(t) => t.clone(),
        None => {
            println!("{} No token", "❌".red());
            return Ok(());
        }
    };

    let client = match build_http_client(app) {
        Ok(c) => c,
        Err(e) => {
            println!("{} {}", "❌".red(), e);
            return Ok(());
        }
    };

    let url = format!("{}/api/separation/cancel", app.api_url);
    let form = reqwest::blocking::multipart::Form::new()
        .text("api_token", token)
        .text("hash", hash.to_string());

    match client.post(&url).multipart(form).send() {
        Ok(resp) => {
            if resp.status().is_success() {
                println!("✅ Task cancelled");
                if let Ok(conn) = db.conn.lock() {
                    let _ = conn.execute(
                        "UPDATE tasks SET status = 'cancelled', phase = 'cancelled' WHERE hash = ?1",
                        rusqlite::params![hash],
                    );
                }
            } else {
                println!("❌ Cancel failed (HTTP {})", resp.status());
            }
        }
        Err(e) => println!("❌ Cancel failed: {}", e),
    }
    Ok(())
}

fn cmd_download_task(app: &App, hash: &str) -> anyhow::Result<()> {
    let token = match &app.token {
        Some(t) => t.clone(),
        None => {
            println!("{} No token", "❌".red());
            return Ok(());
        }
    };

    let client = match build_http_client(app) {
        Ok(c) => c,
        Err(e) => {
            println!("{} {}", "❌".red(), e);
            return Ok(());
        }
    };

    // Get output directory
    let mut output_dir = PathBuf::from(
        prompt("Output directory (Enter to keep)", Some(&app.output_dir.to_string_lossy()))?,
    );
    if output_dir.as_os_str().is_empty() {
        output_dir = app.output_dir.clone();
    }
    if std::fs::create_dir_all(&output_dir).is_err() {
        output_dir = PathBuf::from("./output");
        eprintln!("  ⚠️ Configured output dir unavailable, using ./output");
        if std::fs::create_dir_all(&output_dir).is_err() {
            eprintln!("  ❌ Cannot create output directory");
            return Ok(());
        }
    }

    // Load saved output_files from DB if available, or query API
    let db_path = utils::paths::db_path();
    let db_conn = db::Database::new(Some(&db_path.to_string_lossy())).ok();
    let saved_files = db_conn.as_ref().and_then(|d| {
        d.conn.lock().ok().and_then(|conn| {
            db::repositories::get_task_output_files(&conn, hash).ok().flatten()
        })
    });

    let file_items: Vec<serde_json::Value> = if let Some(json_str) = saved_files {
        // Use cached file list
        serde_json::from_str(&json_str).unwrap_or_default()
    } else {
        // Query API for file list
        let url = format!("{}/api/separation/get", app.api_url);
        let resp = match client.get(&url).query(&[("hash", hash), ("api_token", &token)]).send() {
            Ok(r) => r,
            Err(e) => {
                println!("❌ Query failed: {}", e);
                return Ok(());
            }
        };
        let body: Value = match resp.json() {
            Ok(v) => v,
            Err(e) => {
                println!("❌ Parse failed: {}", e);
                return Ok(());
            }
        };
        body.get("data").and_then(|d| d.get("files"))
            .and_then(|v| v.as_array())
            .map(|arr| {
                arr.iter().map(|f| {
                    let url = f.get("url").and_then(|v| v.as_str()).unwrap_or("");
                    let name = f.get("name").and_then(|v| v.as_str())
                        .or_else(|| url.split('/').next_back())
                        .unwrap_or("output.wav");
                    let size = f.get("size").and_then(|v| v.as_str()).and_then(|s| parse_size_mb(s)).unwrap_or(0);
                    serde_json::json!({
                        "remote_name": name,
                        "url": url,
                        "size": size,
                        "downloaded": false,
                    })
                }).collect()
            })
            .unwrap_or_default()
    };

    let pending: Vec<&serde_json::Value> = file_items.iter()
        .filter(|f| {
            let marked_done = f.get("downloaded").and_then(|v| v.as_bool()).unwrap_or(false);
            if !marked_done {
                return true; // never downloaded
            }
            // Check if the file actually exists on disk
            let local = f.get("local_path").and_then(|v| v.as_str()).unwrap_or("");
            if local.is_empty() || !std::path::Path::new(local).exists() {
                return true; // marked done but file is missing
            }
            false // exists and marked done
        })
        .collect();

    let completed_count = file_items.len() - pending.len();

    if pending.is_empty() {
        if completed_count > 0 {
            println!("  ✅ All {} file(s) already downloaded", completed_count);
        } else {
            println!("  ℹ️ No files to download");
        }
        return Ok(());
    }

    println!("  📥 {} file(s) to download ({} already done)", pending.len(), completed_count);

    // Find original filename from task DB for proper naming
    let original_name = db_conn.as_ref().and_then(|d| {
        d.conn.lock().ok().and_then(|conn| {
            db::repositories::get_task_by_hash(&conn, hash).ok().flatten()
        })
    }).map(|t| t.file_name).unwrap_or_else(|| "output".to_string());

    // Download each pending file with streaming + resume
    for (i, file_info) in pending.iter().enumerate() {
        let file_url = file_info.get("url").and_then(|v| v.as_str()).unwrap_or("");
        let remote_name = file_info.get("remote_name").and_then(|v| v.as_str()).unwrap_or("output.wav");
        let local_name = file_transfer::build_local_name(&original_name, remote_name);
        let dest_path = output_dir.join(&local_name);

        // Check for partial download resume
        let resume_from = file_transfer::get_resume_info(&dest_path, file_url);

        if resume_from > 0 {
            println!("  🔄 Resuming {} ({:.1} MB already downloaded)", local_name.cyan(), resume_from as f64 / 1024.0 / 1024.0);
        } else if dest_path.exists() {
            println!("  ✅ {} already exists, skipping", local_name.cyan());
            continue;
        }

        io::stdout().flush()?;
        let dl_start = Instant::now();
        let dl_name = local_name.clone();

        match file_transfer::download_file(&client, file_url, &dest_path, resume_from, move |p| {
            let elapsed = dl_start.elapsed().as_secs_f64().max(0.001);
            let avg_speed = p.bytes as f64 / elapsed;
            let speed_mbps = avg_speed / 1024.0 / 1024.0;
            let total_mb = p.total_bytes.unwrap_or(0) as f64 / 1024.0 / 1024.0;
            let done_mb = p.bytes as f64 / 1024.0 / 1024.0;
            print!("\r  ⬇️ {} ▸ {:.1}% ({:.1}/{:.1} MB @ {:.1} MB/s)   ",
                dl_name.cyan(), p.percent, done_mb, total_mb, speed_mbps);
            let _ = io::stdout().flush();
        }) {
            Ok(()) => {
                println!("\r  ✅ {} downloaded successfully    ", local_name.cyan());
                // Update output_files JSON
                if let Some(ref d) = db_conn {
                    if let Ok(conn) = d.conn.lock() {
                        if let Ok(Some(json_str)) = db::repositories::get_task_output_files(&conn, hash) {
                            if let Ok(mut list) = serde_json::from_str::<Vec<serde_json::Value>>(&json_str) {
                                for item in list.iter_mut() {
                                    let u = item.get("url").and_then(|v| v.as_str()).unwrap_or("");
                                    if u == file_url {
                                        item["downloaded"] = serde_json::Value::Bool(true);
                                        item["local_path"] = serde_json::Value::String(dest_path.to_string_lossy().to_string());
                                        break;
                                    }
                                }
                                if let Ok(updated) = serde_json::to_string(&list) {
                                    let _ = db::repositories::update_task_output_files(&conn, hash, &updated);
                                }
                            }
                        }
                    }
                }
            }
            Err(e) => {
                println!("\r  ❌ {} download failed: {}", local_name.red(), e);
            }
        }
    }
    println!("  {} Download complete", "✅".green());
    Ok(())
}

// ── Menu command implementations ──

fn cmd_login(app: &mut App, db: &db::Database) -> anyhow::Result<()> {
    let email = prompt("Email", Some("test@example.com"))?;
    let password = prompt("Password", None)?;

    let client = build_http_client(app)?;
    let url = format!("{}/api/app/login", app.api_url);

    let form = reqwest::blocking::multipart::Form::new()
        .text("email", email)
        .text("password", password);

    let t = Instant::now();
    match client.post(&url).multipart(form).send() {
        Ok(resp) => {
            let status = resp.status();
            let body: Value = resp.json().unwrap_or(Value::Null);
            let elapsed = t.elapsed();

            if status.is_success() {
                let token = body
                    .get("data")
                    .and_then(|d| d.get("api_token"))
                    .and_then(|v| v.as_str())
                    .map(|s| s.to_string());
                if let Some(tok) = token {
                    app.token = Some(tok.clone());
                    save_token_to_db(db, &tok);
                    app.token_valid = verify_token(app).unwrap_or(false);
                    println!("✅ Login success! Token saved ({}..{})", &tok[..4], &tok[tok.len() - 4..]);
                } else {
                    println!("⚠️ Login success but no token in response.");
                }
            } else {
                let msg = body.get("message").and_then(|v| v.as_str()).unwrap_or("?");
                println!("❌ Login failed (HTTP {}): {}", status, msg);
            }
            println!("   Elapsed: {:.0}ms", elapsed.as_millis());
        }
        Err(e) => println!("❌ Request failed: {}", e),
    }
    Ok(())
}

fn cmd_set_token(app: &mut App, db: &db::Database) -> anyhow::Result<()> {
    let current = app.token.as_deref().unwrap_or("");
    let display = if current.len() > 8 {
        format!("{}..{}", &current[..4], &current[current.len() - 4..])
    } else {
        current.to_string()
    };
    let token = prompt("API Token", Some(&display))?;

    if token.is_empty() {
        app.token = None;
        save_token_to_db(db, "");
        println!("⚠️ Token cleared");
    } else {
        app.token = Some(token.clone());
        save_token_to_db(db, &token);
        app.token_valid = verify_token(app).unwrap_or(false);
        println!("✅ Token set and verified");
    }
    show_status(app);
    Ok(())
}

fn cmd_logout(app: &mut App, db: &db::Database) -> anyhow::Result<()> {
    app.token = None;
    app.token_valid = false;
    save_token_to_db(db, "");
    println!("{} Logged out", "🔓".yellow());
    show_status(app);
    Ok(())
}

fn cmd_proxy_config(app: &mut App, _db: &db::Database) -> anyhow::Result<()> {
    println!("\n{}", "--- Proxy Configuration ---".yellow());

    let current_mode = app.proxy_mode.clone();
    let mode = prompt(
        "Proxy mode (system / manual / none)",
        Some(&current_mode),
    )?;
    let mode = mode.to_lowercase();
    if mode != "system" && mode != "manual" && mode != "none" {
        println!("{} Invalid mode. Use system, manual, or none.", "❌".red());
        return Ok(());
    }
    app.proxy_mode = mode;

    if app.proxy_mode == "manual" {
        let host = prompt("Proxy host", Some(&app.proxy_host))?;
        if !host.is_empty() {
            app.proxy_host = host;
        }
        let port_str = prompt("Proxy port", Some(&app.proxy_port.to_string()))?;
        if let Ok(p) = port_str.parse::<u16>() {
            app.proxy_port = p;
        } else {
            println!("{} Invalid port, keeping {}", "⚠️".yellow(), app.proxy_port);
        }
    }

    // Save to user config
    if let Ok(ucfg) = open_user_config() {
        let _ = ucfg.set_string("proxy_mode", &app.proxy_mode);
        let _ = ucfg.set_string("proxy_host", &app.proxy_host);
        let _ = ucfg.set_string("proxy_port", &app.proxy_port.to_string());
        println!("{} Proxy settings saved", "✅".green());
    } else {
        println!("{} Failed to save proxy config", "❌".red());
    }
    show_status(app);
    Ok(())
}

fn save_token_to_db(_db: &db::Database, token: &str) {
    if let Ok(ucfg) = open_user_config() {
        let _ = ucfg.set_string("token", token);
    }
}

fn save_pref_str(key: &str, value: &str) {
    if let Ok(ucfg) = open_user_config() {
        let _ = ucfg.set_string(key, value);
    }
}

fn save_pref_int(key: &str, value: i64) {
    if let Ok(ucfg) = open_user_config() {
        let _ = ucfg.set_int(key, value);
    }
}

fn cmd_user_prefs(app: &mut App, _db: &db::Database) -> anyhow::Result<()> {
    loop {
        println!("\n{}", "── User Preferences ──".yellow());
        println!("  {} Output Directory:  {}", "[1]".cyan(), app.output_dir.to_string_lossy().cyan());
        println!("  {} Default Format:    {} (ID: {})", "[2]".cyan(), format_name(app.output_format), app.output_format);
        println!("  {} API URL:           {}", "[3]".cyan(), app.api_url.cyan());
        let premium_status = if app.premium_enabled { "✅ On" } else { "❌ Off" };
        let fn_status = if app.long_filenames { "✅ On" } else { "❌ Off" };
        println!("  {} Auto-Refresh Days: {} days", "[4]".cyan(), "15");
        println!("  {} Premium Mode:      {}", "[5]".cyan(), premium_status);
        println!("  {} Long Filenames:    {}", "[6]".cyan(), fn_status);
        println!("  {} Back to main menu", "[b]".dimmed());
        print!("> ");
        io::stdout().flush()?;

        let mut input = String::new();
        io::stdin().read_line(&mut input)?;
        match input.trim() {
            "1" => {
                let val = prompt("Output directory (Enter to keep)", Some(&app.output_dir.to_string_lossy()))?;
                app.output_dir = PathBuf::from(val);
                save_pref_str("output_dir", &app.output_dir.to_string_lossy());
                println!("✅ Output directory saved");
            }
            "2" => {
                println!("\n  Available formats:");
                println!("    0 = MP3 (320 kbps)");
                println!("    1 = WAV (16 bit)");
                println!("    2 = FLAC (16 bit)");
                println!("    3 = M4A (lossy)");
                println!("    4 = WAV (32 bit)");
                println!("    5 = FLAC (24 bit)");
                let val = prompt_int("  Default format ID", app.output_format)?;
                if (0..=5).contains(&val) {
                    app.output_format = val;
                    save_pref_int("output_format", val as i64);
                    println!("✅ Default format set to {} ({})", val, format_name(val));
                } else {
                    println!("❌ Invalid format ID, use 0-5");
                }
            }
            "3" => {
                let val = prompt("API URL", Some(&app.api_url))?;
                if !val.is_empty() {
                    app.api_url = val;
                    // Save to mvsep.db config table
                    let db_path = utils::paths::db_path();
                    if let Ok(db) = db::Database::new(Some(&db_path.to_string_lossy())) {
                        if let Ok(conn) = db.conn.lock() {
                            let config = db::repositories::get_config(&conn).ok().flatten().unwrap_or_default();
                            let updated = db::repositories::ConfigRow {
                                api_url: Some(app.api_url.clone()),
                                ..config
                            };
                            let _ = db::repositories::save_config(&conn, &updated);
                        }
                    }
                    println!("✅ API URL saved");
                }
            }
            "4" => {
                let current = open_user_config().ok()
                    .and_then(|c| c.get_int("algorithm_auto_refresh_days").ok().flatten())
                    .unwrap_or(15);
                let val = prompt_int("Auto-refresh days (1-90)", current as i32)?;
                if (1..=90).contains(&val) {
                    save_pref_int("algorithm_auto_refresh_days", val as i64);
                    println!("✅ Auto-refresh set to {} days", val);
                } else {
                    println!("❌ Use 1-90 days");
                }
            }
            "5" => {
                let on = prompt_bool("Enable premium mode?", app.premium_enabled)?;
                let endpoint = if on { "enable_premium" } else { "disable_premium" };
                toggle_endpoint(app, endpoint, "Premium");
                // The toggle_endpoint function already calls the API; update local state
                if on { app.premium_enabled = true; } else { app.premium_enabled = false; }
                save_pref_int("premium_enabled", if app.premium_enabled { 1 } else { 0 });
            }
            "6" => {
                let on = prompt_bool("Enable long filenames?", app.long_filenames)?;
                let endpoint = if on { "enable_long_filenames" } else { "disable_long_filenames" };
                toggle_endpoint(app, endpoint, "Long Filenames");
                if on { app.long_filenames = true; } else { app.long_filenames = false; }
                save_pref_int("long_filenames_enabled", if app.long_filenames { 1 } else { 0 });
            }
            "b" | "B" => break,
            _ => println!("{} Unknown option", "⚠️".yellow()),
        }
    }
    Ok(())
}

/// Parse a size string like "74.29 MB" into bytes.
fn parse_size_mb(s: &str) -> Option<u64> {
    let s = s.trim();
    if let Some(val) = s.strip_suffix("MB").or_else(|| s.strip_suffix("Mb")).map(|v| v.trim()) {
        val.parse::<f64>().ok().map(|mb| (mb * 1024.0 * 1024.0) as u64)
    } else if let Some(val) = s.strip_suffix("KB").or_else(|| s.strip_suffix("Kb")) {
        val.parse::<f64>().ok().map(|kb| (kb * 1024.0) as u64)
    } else {
        s.parse::<u64>().ok()
    }
}

fn format_name(id: i32) -> &'static str {
    match id {
        0 => "MP3 (320 kbps)",
        1 => "WAV (16 bit)",
        2 => "FLAC (16 bit)",
        3 => "M4A (lossy)",
        4 => "WAV (32 bit)",
        5 => "FLAC (24 bit)",
        _ => "Unknown",
    }
}

fn cmd_create_task(app: &App, db: &db::Database) -> anyhow::Result<()> {
    let token = require_token(app)?;

    let audio_file = match find_audio_file() {
        Some(p) => p,
        None => {
            println!("❌ No audio file found (looked for: 螢塚-Calvaria.mp3, test.mp3)");
            println!("  Place an audio file in the current directory and try again.");
            return Ok(());
        }
    };

    let file_size = std::fs::metadata(&audio_file)?.len();
    let file_name = audio_file
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("audio.wav")
        .to_string();

    let sep_type = prompt_int("Sep Type ID", 26)?;
    let output_format = prompt_int("Output Format (0=MP3,1=WAV16,2=FLAC16,3=M4A,4=WAV32,5=FLAC24)", 1)?;
    let is_demo = prompt_bool("Demo Mode? (y=free demo, n=real, costs credits)", true)?;

    // Try to load algorithm fields from DB
    let db_path = utils::paths::db_path();
    let fields = match db::Database::new(Some(&db_path.to_string_lossy())) {
        Ok(db) => db.with_conn(|conn| db::repositories::get_algorithm_fields(conn, sep_type)).ok().unwrap_or_default(),
        Err(_) => Vec::new(),
    };

    let (opt1, opt2, opt3) = if fields.is_empty() {
        // No fields cached — suggest refresh
        println!("  ℹ️ Algorithm details not cached (use [r] to refresh from API).");
        println!("     Enter -1 to skip any option.");
        let o1 = prompt_int("  add_opt1", -1)?;
        let o2 = prompt_int("  add_opt2", -1)?;
        let o3 = prompt_int("  add_opt3", -1)?;
        (
            if o1 >= 0 { Some(o1) } else { None },
            if o2 >= 0 { Some(o2) } else { None },
            if o3 >= 0 { Some(o3) } else { None },
        )
    } else {
        prompt_algorithm_fields(&fields)
    };

    println!("\n{}", "--- Create Separation Task ---".yellow());
    println!("  📤 File: {:?} ({:.2} MB)", file_name, file_size as f64 / 1024.0 / 1024.0);
    println!("  📊 Algorithm ID: {}", sep_type);
    if let Some(v) = opt1 { println!("  🔧 add_opt1: {}", v); }
    if let Some(v) = opt2 { println!("  🔧 add_opt2: {}", v); }
    if let Some(v) = opt3 { println!("  🔧 add_opt3: {}", v); }
    println!("  💾 Output Format ID: {}", output_format);
    println!("  💰 Demo Mode: {}", if is_demo { "Yes (free)".green() } else { "No (costs credits)".yellow() });
    println!("  🌐 Via: {}:{}", app.proxy_host, app.proxy_port);

    let client = build_http_client(app)?;
    let url = format!("{}/api/separation/create", app.api_url);

    let mut fields: Vec<(String, String)> = vec![
        ("api_token".into(), token.to_string()),
        ("sep_type".into(), sep_type.to_string()),
        ("output_format".into(), output_format.to_string()),
        ("is_demo".into(), if is_demo { "1".into() } else { "0".into() }),
    ];
    if let Some(v) = opt1 { fields.push(("add_opt1".into(), v.to_string())); }
    if let Some(v) = opt2 { fields.push(("add_opt2".into(), v.to_string())); }
    if let Some(v) = opt3 { fields.push(("add_opt3".into(), v.to_string())); }

    let upload_start = Instant::now();
    let upload_hash = file_transfer::upload_file(
        &app.proxy_host, app.proxy_port, &app.proxy_mode,
        &url, &audio_file, fields,
        move |p| {
            let total_mb = p.total_bytes.unwrap_or(0) as f64 / 1048576.0;
            let done_mb = p.bytes as f64 / 1048576.0;
            print!("\r  🔄 {:.0}% ({:.1}/{:.1} MB)", p.percent, done_mb, total_mb);
            let _ = io::stdout().flush();
        },
    );

    match upload_hash {
        Ok(hash) => {
            let elapsed = upload_start.elapsed();
            println!("\r✅ ({:.0}ms)", elapsed.as_millis());
            println!("{} Task created!", "✅".green().bold());
            println!("   Hash: {}", hash.cyan());

            // Save to local tasks DB
            let now = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_secs() as i64)
                .unwrap_or(0);
            let task = db::repositories::TaskRow {
                hash: hash.to_string(),
                file_name: file_name.clone(),
                algorithm_id: sep_type,
                algorithm_name: format!("Algo {}", sep_type),
                model_id: None,
                model_name: None,
                model2_id: None,
                model2_name: None,
                model3_id: None,
                model3_name: None,
                format: output_format,
                status: "uploaded".to_string(),
                progress: 0.0,
                created_at: now,
                output_files: "[]".to_string(),
                error: None,
                message: None,
                queue_count: None,
                current_order: None,
                phase: "uploaded".to_string(),
                download_file_name: None,
                download_bytes: 0,
                download_total_bytes: None,
                download_speed_bps: 0.0,
                download_percent: 0.0,
            };
            if let Ok(conn) = db.conn.lock() {
                let _ = db::repositories::insert_task(&conn, &task);
            }
        }
        Err(e) => println!("❌ {}", e),
    }
    Ok(())
}

fn cmd_cancel_task(app: &App, db: &db::Database) -> anyhow::Result<()> {
    let token = require_token(app)?;
    let hash = prompt("Task hash to cancel", None)?;
    if hash.is_empty() {
        println!("⚠️ No hash entered");
        return Ok(());
    }

    let client = build_http_client(app)?;
    let url = format!("{}/api/separation/cancel", app.api_url);

    let form = reqwest::blocking::multipart::Form::new()
        .text("api_token", token.to_string())
        .text("hash", hash.clone());

    match client.post(&url).multipart(form).send() {
        Ok(resp) => {
            let status = resp.status();
            if status.is_success() {
                println!("✅ Task cancelled ({})", hash.dimmed());
                // Update local DB
                if let Ok(conn) = db.conn.lock() {
                    let _ = conn.execute(
                        "UPDATE tasks SET status = 'cancelled', phase = 'cancelled' WHERE hash = ?1",
                        rusqlite::params![hash],
                    );
                }
            } else {
                let body: Value = resp.json().unwrap_or(Value::Null);
                let msg = body.get("message").and_then(|v| v.as_str()).unwrap_or("?");
                println!("❌ Cancel failed (HTTP {}): {}", status, msg);
            }
        }
        Err(e) => println!("❌ Request failed: {}", e),
    }
    Ok(())
}

fn cmd_query_task(app: &App) -> anyhow::Result<()> {
    let token = require_token(app)?;
    // Open DB for optional sync
    let db_path = utils::paths::db_path();
    let db = db::Database::new(Some(&db_path.to_string_lossy())).ok();
    let hash = prompt("Task hash", None)?;
    if hash.is_empty() {
        println!("⚠️ No hash entered");
        return Ok(());
    }

    let client = build_http_client(app)?;
    let url = format!("{}/api/separation/get", app.api_url);

    let t = Instant::now();
    let token_str = token.to_string();
    match client.get(&url).query(&[("hash", hash.as_str()), ("api_token", token_str.as_str())]).send() {
        Ok(resp) => {
            let elapsed = t.elapsed();
            let status = resp.status();
            if !status.is_success() {
                let body: Value = resp.json().unwrap_or(Value::Null);
                let msg = body.get("message").and_then(|v| v.as_str()).unwrap_or("?");
                println!("❌ Query failed (HTTP {}): {}", status, msg);
                return Ok(());
            }

            let body: Value = resp.json().unwrap_or(Value::Null);
            let task_status = body.get("status").and_then(|v| v.as_str()).unwrap_or("unknown");
            let message = body.get("data").and_then(|d| d.get("message")).and_then(|v| v.as_str())
                .or_else(|| body.get("message").and_then(|v| v.as_str()))
                .unwrap_or("");

            let finished = body.get("data").and_then(|d| d.get("finished_chunks")).and_then(|v| v.as_i64()).unwrap_or(0);
            let total = body.get("data").and_then(|d| d.get("all_chunks")).and_then(|v| v.as_i64()).unwrap_or(0);
            let progress = if total > 0 { finished as f64 / total as f64 * 100.0 } else { 0.0 };

            let queue_count = body.get("data").and_then(|d| d.get("queue_count")).and_then(|v| v.as_i64());
            let current_order = body.get("data").and_then(|d| d.get("current_order")).and_then(|v| v.as_i64());

            let files = body.get("data").and_then(|d| d.get("files")).and_then(|v| v.as_array())
                .map(|arr| {
                    arr.iter().filter_map(|f| {
                        f.get("name").and_then(|n| n.as_str()).map(|s| s.to_string())
                            .or_else(|| f.get("url").and_then(|u| u.as_str()).and_then(|u| u.split('/').next_back().map(|s| s.to_string())))
                    }).collect::<Vec<_>>()
                });

            println!("\n{}", "── Task Status ──".yellow());
            println!("  Hash:     {}", hash.cyan());
            println!("  Status:   {}", task_status.green());
            if !message.is_empty() {
                println!("  Message:  {}", message.dimmed());
            }
            if total > 0 {
                println!("  Progress: {:.1}% ({}/{})", progress, finished, total);
            }
            if let Some(q) = queue_count {
                let order = current_order.unwrap_or(0);
                println!("  Queue:    position {}/{}", order, q);
            }
            if let Some(files) = files {
                for (i, f) in files.iter().enumerate() {
                    println!("  File {}:   {}", i + 1, f.cyan());
                }
            }
            println!("  Elapsed:  {:.0}ms", elapsed.as_millis());
            println!("{}", "─────────────────".yellow());

            // Sync local DB with latest status
            if let Some(ref d) = db {
                if let Ok(conn) = d.conn.lock() {
                    let _ = db::repositories::update_task_status(
                        &conn, &hash, task_status, progress / 100.0, None,
                    );
                }
            }
        }
        Err(e) => println!("❌ Request failed: {}", e),
    }
    Ok(())
}

fn toggle_endpoint(app: &App, endpoint: &str, label: &str) -> anyhow::Result<()> {
    let token = require_token(app)?;
    let client = build_http_client(app)?;
    let url = format!("{}/api/app/{}", app.api_url, endpoint);

    let form = reqwest::blocking::multipart::Form::new().text("api_token", token.to_string());

    print!("  {} ... ", label.dimmed());
    match client.post(&url).multipart(form).send() {
        Ok(resp) if resp.status().is_success() => println!("✅"),
        Ok(resp) => println!("⚠️ HTTP {}", resp.status()),
        Err(e) => println!("❌ {}", e.to_string().lines().next().unwrap_or("?")),
    }
    Ok(())
}

fn cmd_user_info(app: &App) -> anyhow::Result<()> {
    let token = require_token(app)?;

    let client = build_http_client(app)?;
    let url = format!("{}/api/app/user?api_token={}", app.api_url, token);

    match client.get(&url).send() {
        Ok(resp) => {
            if resp.status().is_success() {
                let body: Value = resp.json().unwrap_or(Value::Null);
                let data = body.get("data").unwrap_or(&body);
                let name = data.get("name").and_then(|v| v.as_str()).unwrap_or("?");
                let email = data.get("email").and_then(|v| v.as_str()).unwrap_or("?");
                let plan = data.get("plan").and_then(|v| v.as_str()).unwrap_or("free");
                let premium = data.get("premium_minutes").and_then(|v| v.as_i64()).unwrap_or(0);

                // Cache premium/filename state from API
                let premium_on = data.get("premium_enabled").and_then(|v| v.as_i64()).unwrap_or(0) != 0;
                let fn_on = data.get("long_filenames_enabled").and_then(|v| v.as_i64()).unwrap_or(0) != 0;
                save_pref_int("premium_enabled", if premium_on { 1 } else { 0 });
                save_pref_int("long_filenames_enabled", if fn_on { 1 } else { 0 });

                println!("📋 User Info:");
                println!("   Name:    {}", name.green());
                println!("   Email:   {}", email.dimmed());
                println!("   Plan:    {}", plan.yellow());
                println!("   Premium: {} min", premium.to_string().cyan());
                if let Some(max_size) = data.get("max_file_size_mb").and_then(|v| v.as_i64()) {
                    println!("   Max file: {} MB", max_size.to_string().cyan());
                }
            } else {
                println!("❌ HTTP {}", resp.status());
            }
        }
        Err(e) => println!("❌ Request failed: {}", e),
    }
    Ok(())
}

fn cmd_run_all_tests(app: &App, db: &db::Database) -> anyhow::Result<()> {
    println!("\n{}", "Running all API tests...".yellow().bold());

    // Test 1: Queue status
    println!("\n{}", "--- [1/5] Queue Status ---".yellow());
    if let Some(ref token) = app.token {
        let client = build_http_client(app)?;
        let url = format!("{}/api/app/queue", app.api_url);
        match client.get(&url).query(&[("api_token", token)]).send() {
            Ok(resp) if resp.status().is_success() => {
                let body: Value = resp.json().unwrap_or(Value::Null);
                let active = body.pointer("/queue/in_process").and_then(|v| v.as_u64()).unwrap_or(0);
                let queued = body.get("queue").and_then(|v| v.as_u64()).unwrap_or(0);
                println!("  ✅ Queue: {} active, {} queued", active, queued);
            }
            Ok(resp) => println!("  ⚠️ HTTP {}", resp.status()),
            Err(e) => println!("  ❌ {}", e),
        }
    } else {
        println!("  ⏭️ Skipped (no token)");
    }

    // Test 2: Algorithm list
    println!("\n{}", "--- [2/5] Algorithm List ---".yellow());
    {
        let client = build_http_client(app)?;
        let url = format!("{}/api/app/algorithms?scopes=single_upload", app.api_url);
        match client.get(&url).send() {
            Ok(resp) if resp.status().is_success() => {
                let body: Value = resp.json().unwrap_or(Value::Null);
                let count = body.as_array().map(|a| a.len()).unwrap_or(0);
                println!("  ✅ {} algorithms loaded", count);
                if count > 0 {
                    if let Some(first) = body[0].get("name").and_then(|v| v.as_str()) {
                        println!("     First: {}", first.dimmed());
                    }
                }
            }
            Ok(resp) => println!("  ⚠️ HTTP {}", resp.status()),
            Err(e) => println!("  ❌ {}", e),
        }
    }

    // Test 3: Output formats from DB
    println!("\n{}", "--- [3/5] Output Formats (DB) ---".yellow());
    {
        let conn = db.conn.lock().map_err(|e| anyhow::anyhow!("DB lock: {}", e))?;
        match db::repositories::get_all_output_formats(&conn) {
            Ok(formats) => {
                println!("  ✅ {} formats in DB:", formats.len());
                for f in &formats {
                    let bits = f.bits_per_sample.map(|b| format!("{} bit", b)).unwrap_or_else(|| "lossy".to_string());
                    let premium = if f.is_premium { " 🔒 Premium" } else { "" };
                    println!("     ID {}: {} ({}){}", f.id, f.name.cyan(), bits, premium.red());
                }
            }
            Err(e) => println!("  ❌ DB error: {}", e),
        }
    }

    // Test 4: User info
    println!("\n{}", "--- [4/5] User Info ---".yellow());
    if let Some(ref token) = app.token {
        let client = build_http_client(app)?;
        let url = format!("{}/api/app/user?api_token={}", app.api_url, token);
        match client.get(&url).send() {
            Ok(resp) if resp.status().is_success() => {
                let body: Value = resp.json().unwrap_or(Value::Null);
                let data = body.get("data").unwrap_or(&body);
                let name = data.get("name").and_then(|v| v.as_str()).unwrap_or("?");
                let plan = data.get("plan").and_then(|v| v.as_str()).unwrap_or("free");
                println!("  ✅ User: {} (Plan: {})", name.green(), plan.yellow());
            }
            Ok(resp) => println!("  ⚠️ HTTP {}", resp.status()),
            Err(e) => println!("  ❌ {}", e),
        }
    } else {
        println!("  ⏭️ Skipped (no token)");
    }

    // Test 5: Toggle endpoints
    println!("\n{}", "--- [5/5] Toggle Endpoints ---".yellow());
    if app.token.is_some() {
        toggle_endpoint(app, "enable_premium", "  Enable Premium")?;
        toggle_endpoint(app, "disable_premium", "  Disable Premium")?;
    } else {
        println!("  ⏭️ Skipped (no token)");
    }

    println!("\n{}", "All tests completed!".green().bold());
    Ok(())
}
