//! Proxy coverage integration tests.
//!
//! These tests require a running proxy server and optionally an API token.
//! Run them with:
//!
//! ```bash
//! cargo test --test proxy_integration -- --ignored --nocapture
//! ```
//!
//! Set environment variables to configure:
//! - `MVSEP_PROXY_HOST` (default: 127.0.0.1)
//! - `MVSEP_PROXY_PORT` (default: 7897)
//! - `MVSEP_API_URL` (default: https://mvsep.com)
//! - `MVSEP_API_TOKEN` (optional: needed for authenticated endpoints)

use std::net::ToSocketAddrs;
use std::time::Instant;

const DEFAULT_PROXY_HOST: &str = "127.0.0.1";
const DEFAULT_PROXY_PORT: u16 = 7897;
const DEFAULT_API_URL: &str = "https://mvsep.com";

// ── Helpers ──

fn proxy_host() -> String {
    std::env::var("MVSEP_PROXY_HOST").unwrap_or_else(|_| DEFAULT_PROXY_HOST.to_string())
}

fn proxy_port() -> u16 {
    std::env::var("MVSEP_PROXY_PORT")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(DEFAULT_PROXY_PORT)
}

fn api_url() -> String {
    std::env::var("MVSEP_API_URL").unwrap_or_else(|_| DEFAULT_API_URL.to_string())
}

fn api_token() -> Option<String> {
    std::env::var("MVSEP_API_TOKEN").ok().filter(|t| !t.is_empty())
}

fn build_proxy_client() -> reqwest::blocking::Client {
    let proxy_url = format!("http://{}:{}", proxy_host(), proxy_port());
    let proxy = reqwest::Proxy::all(&proxy_url).expect("Invalid proxy URL");
    reqwest::blocking::Client::builder()
        .proxy(proxy)
        .timeout(std::time::Duration::from_secs(15))
        .build()
        .expect("Failed to build HTTP client")
}

// ── Tests ──

/// Verify TCP connectivity to the proxy
#[test]
#[ignore]
fn test_tcp_connection() {
    let host = proxy_host();
    let port = proxy_port();
    let addr = format!("{}:{}", host, port);

    let socket_addrs: Vec<_> = addr
        .to_socket_addrs()
        .expect("DNS resolution failed")
        .collect();
    assert!(!socket_addrs.is_empty(), "No addresses resolved for proxy");

    let resolved = socket_addrs[0];
    let connected = std::net::TcpStream::connect_timeout(&resolved, std::time::Duration::from_secs(5));
    assert!(connected.is_ok(), "TCP connection to proxy {} failed: {:?}", resolved, connected.err());
}

/// Verify DNS resolution works
#[test]
#[ignore]
fn test_dns_resolution() {
    // Test that we can resolve MVSep's domain
    let addrs: Vec<_> = format!("{}:443", "mvsep.com")
        .to_socket_addrs()
        .expect("DNS resolution failed")
        .collect();
    assert!(!addrs.is_empty(), "Failed to resolve mvsep.com");
}

/// Verify generic HTTP connectivity through the proxy
#[test]
#[ignore]
fn test_generic_http_via_proxy() {
    let client = build_proxy_client();
    let test_urls = [
        ("httpbin.org/ip", "https://httpbin.org/ip"),
        ("ip-api.com/json", "http://ip-api.com/json"),
    ];

    let mut any_ok = false;
    for (name, url) in &test_urls {
        match client.get(*url).timeout(std::time::Duration::from_secs(8)).send() {
            Ok(resp) if resp.status().is_success() => {
                println!("  ✅ {} responded", name);
                any_ok = true;
                break;
            }
            Ok(resp) => println!("  ⚠️ {} HTTP {}", name, resp.status()),
            Err(e) => println!("  ❌ {} {}", name, e),
        }
    }
    assert!(any_ok, "All generic HTTP tests failed — proxy may be down");
}

/// Verify we can fetch the algorithm list via proxy
#[test]
#[ignore]
fn test_get_algorithms() {
    let client = build_proxy_client();
    let url = format!("{}/api/app/algorithms?scopes=single_upload", api_url());

    let resp = client.get(&url).send().expect("GET /app/algorithms failed");
    assert!(resp.status().is_success(), "HTTP {}", resp.status());

    let body: serde_json::Value = resp.json().expect("Invalid JSON response");
    let algorithms = body.as_array().expect("Expected array response");
    assert!(!algorithms.is_empty(), "Algorithm list is empty");

    println!("  ✅ {} algorithms loaded", algorithms.len());
    if let Some(name) = algorithms[0].get("name").and_then(|v| v.as_str()) {
        println!("     First: {}", name);
    }
}

/// Verify task queue status endpoint
#[test]
#[ignore]
fn test_queue_status() {
    let token = api_token().expect("MVSEP_API_TOKEN not set (required for this test)");
    let client = build_proxy_client();
    let url = format!("{}/api/app/queue", api_url());

    let resp = client
        .get(&url)
        .query(&[("api_token", &token)])
        .send()
        .expect("GET /queue failed");
    assert!(resp.status().is_success(), "HTTP {}", resp.status());

    let body: serde_json::Value = resp.json().expect("Invalid JSON");
    println!("  ✅ Queue info: {}", body);
}

/// Verify user info endpoint
#[test]
#[ignore]
fn test_user_info() {
    let token = api_token().expect("MVSEP_API_TOKEN not set (required for this test)");
    let client = build_proxy_client();
    let url = format!("{}/api/app/user?api_token={}", api_url(), token);

    let resp = client.get(&url).send().expect("GET /user failed");
    assert!(resp.status().is_success(), "HTTP {}", resp.status());

    let body: serde_json::Value = resp.json().expect("Invalid JSON");
    let data = body.get("data").or(Some(&body));
    if let Some(d) = data {
        let name = d.get("name").and_then(|v| v.as_str()).unwrap_or("?");
        let plan = d.get("plan").and_then(|v| v.as_str()).unwrap_or("free");
        println!("  ✅ User: {} (Plan: {})", name, plan);
    }
}

/// Verify premium toggle endpoints
#[test]
#[ignore]
fn test_toggle_premium() {
    let token = api_token().expect("MVSEP_API_TOKEN not set (required for this test)");
    let client = build_proxy_client();

    for (endpoint, label) in &[("/app/enable_premium", "Enable Premium"), ("/app/disable_premium", "Disable Premium")] {
        let url = format!("{}/api{}", api_url(), endpoint);
        let form = reqwest::blocking::multipart::Form::new().text("api_token", token.clone());

        let resp = client.post(&url).multipart(form).send().unwrap_or_else(|_| panic!("POST {} failed", endpoint));
        assert!(resp.status().is_success(), "{} HTTP {}", label, resp.status());
        println!("  ✅ {}", label);
    }
}

/// Verify filename toggle endpoints
#[test]
#[ignore]
fn test_toggle_long_filenames() {
    let token = api_token().expect("MVSEP_API_TOKEN not set (required for this test)");
    let client = build_proxy_client();

    for (endpoint, label) in &[("/app/enable_long_filenames", "Enable Long Filenames"), ("/app/disable_long_filenames", "Disable Long Filenames")] {
        let url = format!("{}/api{}", api_url(), endpoint);
        let form = reqwest::blocking::multipart::Form::new().text("api_token", token.clone());

        let resp = client.post(&url).multipart(form).send().unwrap_or_else(|_| panic!("POST {} failed", endpoint));
        assert!(resp.status().is_success(), "{} HTTP {}", label, resp.status());
        println!("  ✅ {}", label);
    }
}

/// Full connectivity check: DNS → TCP → HTTP → API
#[test]
#[ignore]
fn test_full_proxy_coverage() {
    println!("\n═══ Full Proxy Coverage Test ═══");
    let host = proxy_host();
    let port = proxy_port();

    // 1. DNS
    let t = Instant::now();
    let addrs: Vec<_> = format!("{}:{}", host, port)
        .to_socket_addrs()
        .expect("DNS resolution failed")
        .collect();
    assert!(!addrs.is_empty(), "No addresses resolved");
    println!("  ✅ DNS ({}ms)", t.elapsed().as_millis());

    // 2. TCP
    let t = Instant::now();
    let connected = std::net::TcpStream::connect_timeout(&addrs[0], std::time::Duration::from_secs(5));
    assert!(connected.is_ok(), "TCP connection failed");
    println!("  ✅ TCP ({}ms)", t.elapsed().as_millis());

    // 3. HTTP via proxy
    let t = Instant::now();
    let client = build_proxy_client();
    let resp = client
        .get("https://httpbin.org/ip")
        .timeout(std::time::Duration::from_secs(8))
        .send();
    assert!(resp.is_ok(), "HTTP via proxy failed: {:?}", resp.err());
    println!("  ✅ HTTP via proxy ({}ms)", t.elapsed().as_millis());

    // 4. API algorithms (unauthenticated)
    let t = Instant::now();
    let url = format!("{}/api/app/algorithms?scopes=single_upload", api_url());
    let resp = client.get(&url).send().expect("GET algorithms failed");
    assert!(resp.status().is_success(), "HTTP {}", resp.status());
    println!("  ✅ API algorithms ({}ms)", t.elapsed().as_millis());

    // 5. API user info (if token available)
    if let Some(token) = api_token() {
        let t = Instant::now();
        let url = format!("{}/api/app/user?api_token={}", api_url(), token);
        let resp = client.get(&url).send().expect("GET user failed");
        assert!(resp.status().is_success(), "HTTP {}", resp.status());
        println!("  ✅ API user info ({}ms)", t.elapsed().as_millis());
    } else {
        println!("  ⏭️ API user info (no token)");
    }

    println!("═══ All proxy tests passed ═══");
}
