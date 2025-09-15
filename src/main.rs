
// src/main.rs
use anyhow::{Context, Result};
use chrono::Local;
use clap::Parser;
use log::{error, info};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::Semaphore;

#[derive(Parser, Debug)]
#[command(name = "porty")]
#[command(author = "Your Name")]
#[command(version = "1.0")]
#[command(about = "A simple port forwarder with config file support", long_about = None)]
struct Args {
    /// Config file path
    #[arg(short, long, default_value = "porty.toml")]
    config: PathBuf,

    /// Generate example config file
    #[arg(long)]
    generate_config: bool,

    /// Override listen address from config
    #[arg(short, long)]
    listen_addr: Option<String>,

    /// Override listen port from config
    #[arg(short = 'p', long)]
    listen_port: Option<u16>,

    /// Override target address from config
    #[arg(short, long)]
    target_addr: Option<String>,

    /// Override target port from config
    #[arg(short = 'P', long)]
    target_port: Option<u16>,

    /// Run as daemon (detach from terminal)
    #[arg(short, long)]
    daemon: bool,

    /// Verbose output (show all connection details)
    #[arg(short, long)]
    verbose: bool,
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

#[derive(Debug, Deserialize, Serialize)]
struct Route {
    name: String,
    listen_port: u16,
    target_addr: String,
    target_port: u16,
    #[serde(default)]
    enabled: bool,
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
    
    let args = Args::parse();

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
    for route in config.routes.iter().filter(|r| r.enabled) {
        let route = route.clone();
        let listen_addr = config.listen_addr.clone();
        let max_conn = config.max_connections;
        let buffer_size = config.buffer_size_kb;
        let log_requests = config.log_requests;
        let verbose = args.verbose;
        
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
        
        println!("🔊 Additional route '{}': {}:{} -> {}:{}", 
            route.name, config.listen_addr, route.listen_port, 
            route.target_addr, route.target_port);
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
            },
            Route {
                name: "ssh".to_string(),
                listen_port: 2222,
                target_addr: "127.0.0.1".to_string(),
                target_port: 22,
                enabled: false,
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
