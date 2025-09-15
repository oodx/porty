
// src/main.rs
use anyhow::{Context, Result};
use chrono::Local;
use log::{error, info};
use rsb::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::io::{AsyncBufReadExt, AsyncReadExt, AsyncWriteExt, BufReader};
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::Semaphore;

#[derive(Debug)]
struct PortyArgs {
    config: PathBuf,
    generate_config: bool,
    listen_addr: Option<String>,
    listen_port: Option<u16>,
    target_addr: Option<String>,
    target_port: Option<u16>,
    daemon: bool,
    verbose: bool,
}

impl PortyArgs {
    fn from_rsb_args(mut args: Args) -> Self {
        let config = PathBuf::from(args.has_val("--config").unwrap_or_else(|| "config.toml".to_string()));
        let generate_config = args.has("--generate-config");
        let listen_addr = args.has_val("--listen-addr").or_else(|| args.has_val("-l"));
        let listen_port = args.has_val("--listen-port").or_else(|| args.has_val("-p")).and_then(|s| s.parse().ok());
        let target_addr = args.has_val("--target-addr").or_else(|| args.has_val("-t"));
        let target_port = args.has_val("--target-port").or_else(|| args.has_val("-P")).and_then(|s| s.parse().ok());
        let daemon = args.has("--daemon") || args.has("-d");
        let verbose = args.has("--verbose") || args.has("-v");

        PortyArgs {
            config,
            generate_config,
            listen_addr,
            listen_port,
            target_addr,
            target_port,
            daemon,
            verbose,
        }
    }
}

#[derive(Debug, Deserialize, Serialize)]
struct Config {
    #[serde(default = "default_listen_addr")]
    listen_addr: String,
    
    #[serde(default = "default_listen_port")]
    listen_port: u16,
    
    #[serde(default = "default_target_addr")]
    target_addr: String,
    
    #[serde(default = "default_target_port")]
    target_port: u16,
    
    #[serde(default = "default_max_connections")]
    max_connections: usize,
    
    #[serde(default = "default_buffer_size")]
    buffer_size_kb: usize,
    
    #[serde(default = "default_log_requests")]
    log_requests: bool,
    
    #[serde(default = "default_log_format")]
    log_format: String,

    #[serde(default)]
    routes: Vec<Route>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
struct Route {
    name: String,
    listen_port: u16,
    target_addr: String,
    target_port: u16,
    #[serde(default)]
    enabled: bool,
    #[serde(default)]
    mode: String, // "tcp" or "http"
    #[serde(default)]
    host: Option<String>, // Host header matching
}

#[derive(Debug, Clone)]
struct HttpRequest {
    method: String,
    path: String,
    query: String,
    headers: HashMap<String, String>,
    body: Vec<u8>,
}

#[derive(Debug)]
struct DynamicRoute {
    target_host: String,
    target_port: u16,
}

// Default values for config
fn default_listen_addr() -> String { "0.0.0.0".to_string() }
fn default_listen_port() -> u16 { 8080 }
fn default_target_addr() -> String { "127.0.0.1".to_string() }
fn default_target_port() -> u16 { 80 }
fn default_max_connections() -> usize { 100 }
fn default_buffer_size() -> usize { 8 }
fn default_log_requests() -> bool { true }
fn default_log_format() -> String { "default".to_string() }

impl Default for Config {
    fn default() -> Self {
        Config {
            listen_addr: default_listen_addr(),
            listen_port: default_listen_port(),
            target_addr: default_target_addr(),
            target_port: default_target_port(),
            max_connections: default_max_connections(),
            buffer_size_kb: default_buffer_size(),
            log_requests: default_log_requests(),
            log_format: default_log_format(),
            routes: vec![],
        }
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();

    let rsb_args = bootstrap!();
    let args = PortyArgs::from_rsb_args(rsb_args);

    // Generate example config if requested
    if args.generate_config {
        generate_example_config(&args.config)?;
        return Ok(());
    }

    // Load config from file
    let mut config = load_config(&args.config)?;

    // Override config with command line arguments
    if let Some(addr) = args.listen_addr {
        config.listen_addr = addr;
    }
    if let Some(port) = args.listen_port {
        config.listen_port = port;
    }
    if let Some(addr) = args.target_addr {
        config.target_addr = addr;
    }
    if let Some(port) = args.target_port {
        config.target_port = port;
    }

    // Daemonize if requested (Unix only)
    #[cfg(unix)]
    if args.daemon {
        daemonize()?;
    }

    // Print startup message
    println!("🚀 Porty v{} starting up", env!("CARGO_PKG_VERSION"));
    println!("📁 Config loaded from: {}", args.config.display());
    println!("🔊 Main route: {}:{} -> {}:{}", 
        config.listen_addr, config.listen_port, 
        config.target_addr, config.target_port);

    // Start additional routes if configured
    let enabled_routes: Vec<Route> = config.routes.iter()
        .filter(|r| r.enabled)
        .cloned()
        .collect();

    for route in enabled_routes {
        let listen_addr = config.listen_addr.clone();
        let max_conn = config.max_connections;
        let buffer_size = config.buffer_size_kb;
        let log_requests = config.log_requests;
        let verbose = args.verbose;

        println!("🔊 Additional route '{}': {}:{} -> {}:{}",
            route.name, listen_addr, route.listen_port,
            route.target_addr, route.target_port);

        tokio::spawn(async move {
            if let Err(e) = run_route(
                &route.name,
                &listen_addr,
                route.listen_port,
                &route.target_addr,
                route.target_port,
                max_conn,
                buffer_size,
                log_requests,
                verbose,
            ).await {
                error!("Route {} failed: {}", route.name, e);
            }
        });
    }

    // Run main route
    run_route(
        "main",
        &config.listen_addr,
        config.listen_port,
        &config.target_addr,
        config.target_port,
        config.max_connections,
        config.buffer_size_kb,
        config.log_requests,
        args.verbose,
    ).await?;

    Ok(())
}

async fn run_route(
    name: &str,
    listen_addr: &str,
    listen_port: u16,
    target_addr: &str,
    target_port: u16,
    max_connections: usize,
    buffer_size_kb: usize,
    log_requests: bool,
    verbose: bool,
) -> Result<()> {
    let semaphore = Arc::new(Semaphore::new(max_connections));
    let listen_addr_full = format!("{}:{}", listen_addr, listen_port);
    let target_addr_full = format!("{}:{}", target_addr, target_port);
    
    let listener = TcpListener::bind(&listen_addr_full)
        .await
        .context(format!("Failed to bind to {}", listen_addr_full))?;
    
    info!("[{}] Listening on {} -> {}", name, listen_addr_full, target_addr_full);

    loop {
        let (client, client_addr) = listener.accept().await?;
        let target_addr = target_addr_full.clone();
        let permit = semaphore.clone().acquire_owned().await?;
        let buffer_size = buffer_size_kb * 1024;
        let route_name = name.to_string();

        tokio::spawn(async move {
            let timestamp = Local::now().format("%Y-%m-%d %H:%M:%S%.3f");
            
            // Print forward request message
            if log_requests {
                println!(
                    "🔄 [{}] {} | New connection from {} -> forwarding to {}",
                    route_name, timestamp, client_addr, target_addr
                );
            }

            let start_time = std::time::Instant::now();
            let mut bytes_transferred = 0u64;

            match handle_connection(client, target_addr.clone(), buffer_size, &mut bytes_transferred).await {
                Ok(_) => {
                    let duration = start_time.elapsed();
                    if verbose {
                        println!(
                            "✅ [{}] {} | Connection closed: {} | Duration: {:.2}s | Transferred: {} bytes",
                            route_name,
                            timestamp,
                            client_addr,
                            duration.as_secs_f64(),
                            format_bytes(bytes_transferred)
                        );
                    }
                }
                Err(e) => {
                    println!(
                        "❌ [{}] {} | Connection error for {}: {}",
                        route_name, timestamp, client_addr, e
                    );
                }
            }
            
            drop(permit);
        });
    }
}

async fn handle_connection(
    client: TcpStream,
    target_addr: String,
    buffer_size: usize,
    bytes_transferred: &mut u64,
) -> Result<()> {
    let target = TcpStream::connect(&target_addr)
        .await
        .context(format!("Failed to connect to target {}", target_addr))?;
    
    let (client_read, client_write) = client.into_split();
    let (target_read, target_write) = target.into_split();
    
    let client_to_target = forward_data(client_read, target_write, buffer_size);
    let target_to_client = forward_data(target_read, client_write, buffer_size);
    
    tokio::select! {
        result = client_to_target => {
            if let Ok(bytes) = result {
                *bytes_transferred += bytes;
            }
        }
        result = target_to_client => {
            if let Ok(bytes) = result {
                *bytes_transferred += bytes;
            }
        }
    }

    Ok(())
}

async fn handle_http_connection(
    mut client: TcpStream,
    route_name: String,
    log_requests: bool,
    verbose: bool,
) -> Result<()> {
    let client_addr = client.peer_addr()?
        .to_string();

    // Parse HTTP request
    let request = parse_http_request(&mut client).await?;

    // Check for dynamic routing parameters
    let dynamic_route = extract_dynamic_route(&request.query)?;

    if let Some(route) = dynamic_route {
        if log_requests {
            let timestamp = Local::now().format("%Y-%m-%d %H:%M:%S%.3f");
            println!(
                "🔄 [{}] {} | {} {}?{}",
                route_name, timestamp, request.method, request.path, request.query
            );
            println!(
                "   ├─ From: {}",
                client_addr
            );
            println!(
                "   ├─ To: {}:{} (dynamic)",
                route.target_host, route.target_port
            );
            if verbose {
                for (key, value) in &request.headers {
                    println!("   ├─ {}: {}", key, value);
                }
            }
        }

        let start_time = std::time::Instant::now();

        // Forward the cleaned request
        match forward_http_request(request, route, client).await {
            Ok(response_info) => {
                if log_requests {
                    let duration = start_time.elapsed();
                    let timestamp = Local::now().format("%Y-%m-%d %H:%M:%S%.3f");
                    println!(
                        "✅ [{}] {} | {} ({:.0}ms)",
                        route_name, timestamp, response_info.status, duration.as_millis()
                    );
                    if verbose {
                        println!("   └─ Body: {} bytes", response_info.body_size);
                    }
                }
            }
            Err(e) => {
                if log_requests {
                    let timestamp = Local::now().format("%Y-%m-%d %H:%M:%S%.3f");
                    println!(
                        "❌ [{}] {} | Error: {}",
                        route_name, timestamp, e
                    );
                }
                return Err(e);
            }
        }
    } else {
        // No dynamic routing, send 400 Bad Request
        let response = "HTTP/1.1 400 Bad Request\r\n\r\nMissing porty_host and porty_port parameters";
        client.write_all(response.as_bytes()).await?;
    }

    Ok(())
}

async fn forward_data(
    mut reader: tokio::net::tcp::OwnedReadHalf,
    mut writer: tokio::net::tcp::OwnedWriteHalf,
    buffer_size: usize,
) -> Result<u64> {
    let mut buffer = vec![0u8; buffer_size];
    let mut total_bytes = 0u64;
    
    loop {
        let n = reader.read(&mut buffer).await?;
        if n == 0 {
            break;
        }
        
        writer.write_all(&buffer[..n]).await?;
        writer.flush().await?;
        total_bytes += n as u64;
    }
    
    Ok(total_bytes)
}

fn load_config(path: &PathBuf) -> Result<Config> {
    if !path.exists() {
        println!("⚠️  Config file not found at: {}", path.display());
        println!("   Using default configuration...");
        println!("   Run 'porty --generate-config' to create an example config file");
        return Ok(Config::default());
    }

    let content = fs::read_to_string(path)
        .context(format!("Failed to read config file: {}", path.display()))?;
    
    let config: Config = toml::from_str(&content)
        .context(format!("Failed to parse config file: {}", path.display()))?;
    
    Ok(config)
}

fn generate_example_config(path: &PathBuf) -> Result<()> {
    let example_config = Config {
        listen_addr: "0.0.0.0".to_string(),
        listen_port: 1455,
        target_addr: "127.0.0.1".to_string(),
        target_port: 1455,
        max_connections: 100,
        buffer_size_kb: 8,
        log_requests: true,
        log_format: "default".to_string(),
        routes: vec![
            Route {
                name: "web".to_string(),
                listen_port: 8080,
                target_addr: "127.0.0.1".to_string(),
                target_port: 80,
                enabled: false,
                mode: "tcp".to_string(),
                host: None,
            },
            Route {
                name: "ssh".to_string(),
                listen_port: 2222,
                target_addr: "127.0.0.1".to_string(),
                target_port: 22,
                enabled: false,
                mode: "tcp".to_string(),
                host: None,
            },
        ],
    };

    let toml_string = toml::to_string_pretty(&example_config)?;
    
    // Add comments to the generated TOML
    let commented_toml = format!(
        "# Porty Configuration File\n\
         # Generated with 'porty --generate-config'\n\n\
         # Main forwarding configuration\n\
         {}\n\n\
         # Additional routes (optional)\n\
         # Enable routes by setting 'enabled = true'\n",
        toml_string
    );
    
    fs::write(path, commented_toml)?;
    println!("✅ Example config file created at: {}", path.display());
    println!("   Edit this file to configure your port forwarding rules");
    
    Ok(())
}

async fn parse_http_request(stream: &mut TcpStream) -> Result<HttpRequest> {
    let mut reader = BufReader::new(stream);
    let mut headers = HashMap::new();
    let mut lines = Vec::new();

    // Read request line and headers
    loop {
        let mut line = String::new();
        reader.read_line(&mut line).await?;
        if line == "\r\n" || line == "\n" {
            break;
        }
        lines.push(line.trim_end().to_string());
    }

    if lines.is_empty() {
        return Err(anyhow::anyhow!("Empty HTTP request"));
    }

    // Parse request line: "GET /path?query HTTP/1.1"
    let request_line = &lines[0];
    let parts: Vec<&str> = request_line.split_whitespace().collect();
    if parts.len() < 2 {
        return Err(anyhow::anyhow!("Invalid HTTP request line"));
    }

    let method = parts[0].to_string();
    let url_part = parts[1];
    let (path, query) = if let Some(pos) = url_part.find('?') {
        (url_part[..pos].to_string(), url_part[pos+1..].to_string())
    } else {
        (url_part.to_string(), String::new())
    };

    // Parse headers
    for line in &lines[1..] {
        if let Some(pos) = line.find(':') {
            let key = line[..pos].trim().to_lowercase();
            let value = line[pos+1..].trim().to_string();
            headers.insert(key, value);
        }
    }

    // Read body if Content-Length is specified
    let mut body = Vec::new();
    if let Some(content_length_str) = headers.get("content-length") {
        if let Ok(content_length) = content_length_str.parse::<usize>() {
            body.resize(content_length, 0);
            reader.read_exact(&mut body).await?;
        }
    }

    Ok(HttpRequest {
        method,
        path,
        query,
        headers,
        body,
    })
}

fn extract_dynamic_route(query: &str) -> Result<Option<DynamicRoute>> {
    let mut params: HashMap<String, String> = HashMap::new();

    for pair in query.split('&') {
        if let Some(pos) = pair.find('=') {
            let key = &pair[..pos];
            let value = &pair[pos+1..];
            params.insert(key.to_string(), value.to_string());
        }
    }

    if let (Some(host), Some(port_str)) = (params.get("porty_host"), params.get("porty_port")) {
        if let Ok(port) = port_str.parse::<u16>() {
            return Ok(Some(DynamicRoute {
                target_host: host.clone(),
                target_port: port,
            }));
        }
    }

    Ok(None)
}

fn clean_query_string(query: &str) -> String {
    query.split('&')
        .filter(|pair| !pair.starts_with("porty_host=") && !pair.starts_with("porty_port="))
        .collect::<Vec<_>>()
        .join("&")
}

#[derive(Debug)]
struct ResponseInfo {
    status: String,
    body_size: usize,
}

async fn forward_http_request(
    request: HttpRequest,
    route: DynamicRoute,
    mut client: TcpStream,
) -> Result<ResponseInfo> {
    // Connect to target
    let target_addr = format!("{}:{}", route.target_host, route.target_port);
    let mut target = TcpStream::connect(&target_addr).await
        .context(format!("Failed to connect to {}", target_addr))?;

    // Clean the query string (remove porty_* params)
    let clean_query = clean_query_string(&request.query);
    let url_path = if clean_query.is_empty() {
        request.path
    } else {
        format!("{}?{}", request.path, clean_query)
    };

    // Build HTTP request
    let mut http_request = format!("{} {} HTTP/1.1\r\n", request.method, url_path);

    // Add headers (update Host header to target)
    let mut headers = request.headers;
    headers.insert("host".to_string(), route.target_host);

    for (key, value) in &headers {
        http_request.push_str(&format!("{}: {}\r\n", key, value));
    }

    http_request.push_str("\r\n");

    // Send request
    target.write_all(http_request.as_bytes()).await?;
    if !request.body.is_empty() {
        target.write_all(&request.body).await?;
    }

    // Read response and forward to client
    let mut response_buf = Vec::new();
    let mut temp_buf = [0u8; 8192];
    let mut total_bytes = 0;
    let mut status_line = String::new();
    let mut headers_read = false;

    loop {
        match target.read(&mut temp_buf).await {
            Ok(0) => break, // Connection closed
            Ok(n) => {
                response_buf.extend_from_slice(&temp_buf[..n]);
                client.write_all(&temp_buf[..n]).await?;
                total_bytes += n;

                // Extract status line from first response
                if !headers_read && response_buf.len() > 0 {
                    let response_str = String::from_utf8_lossy(&response_buf);
                    if let Some(first_line_end) = response_str.find('\n') {
                        status_line = response_str[..first_line_end].trim().to_string();
                        if response_str.contains("\r\n\r\n") {
                            headers_read = true;
                        }
                    }
                }
            }
            Err(e) => return Err(e.into()),
        }
    }

    Ok(ResponseInfo {
        status: if status_line.is_empty() { "Unknown".to_string() } else { status_line },
        body_size: total_bytes,
    })
}

fn format_bytes(bytes: u64) -> String {
    if bytes < 1024 {
        format!("{}", bytes)
    } else if bytes < 1024 * 1024 {
        format!("{:.2} KB", bytes as f64 / 1024.0)
    } else {
        format!("{:.2} MB", bytes as f64 / (1024.0 * 1024.0))
    }
}

#[cfg(unix)]
fn daemonize() -> Result<()> {
    use std::env;
    use std::os::unix::process::CommandExt;
    use std::process::Command;
    
    if env::var("DAEMONIZED").is_ok() {
        return Ok(());
    }
    
    let args: Vec<String> = env::args().collect();
    Command::new(&args[0])
        .args(&args[1..])
        .env("DAEMONIZED", "1")
        .stdin(std::process::Stdio::null())
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .exec();
    
    Ok(())
}
