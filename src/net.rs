// TCP networking and connection handling

use anyhow::{Context, Result};
use chrono::Local;
use log::error;
use std::sync::Arc;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::Semaphore;
use crate::cfg::Config;
use crate::http::handle_http_connection;
use rsb::prelude::*;

pub async fn run_route(
    name: &str,
    listen_addr: &str,
    listen_port: u16,
    target_addr: &str,
    target_port: u16,
    max_connections: usize,
    buffer_size_kb: usize,
    log_requests: bool,
    verbose: bool,
    mode: &str,  // "tcp" or "http"
) -> Result<()> {
    let semaphore = Arc::new(Semaphore::new(max_connections));
    let listen_addr_full = format!("{}:{}", listen_addr, listen_port);
    let target_addr_full = format!("{}:{}", target_addr, target_port);

    let listener = TcpListener::bind(&listen_addr_full)
        .await
        .context(format!("Failed to bind to {}", listen_addr_full))?;

    log::info!("[{}] Listening on {} -> {}", name, listen_addr_full, target_addr_full);

    loop {
        let (client, client_addr) = listener.accept().await?;
        let target_addr = target_addr_full.clone();
        let permit = semaphore.clone().acquire_owned().await?;
        let buffer_size = buffer_size_kb * 1024;
        let route_name = name.to_string();
        let route_mode = mode.to_string();

        tokio::spawn(async move {
            let timestamp = Local::now().format("%Y-%m-%d %H:%M:%S%.3f");

            // Print forward request message
            if log_requests {
                echo!(
                    "🔄 [{}] {} | New connection from {} -> forwarding to {}",
                    route_name, timestamp, client_addr, target_addr
                );
            }

            let start_time = std::time::Instant::now();
            let mut bytes_transferred = 0u64;

            // Route based on mode: TCP or HTTP
            let connection_result = if route_mode == "http" {
                // Use HTTP handler for dynamic routing
                match handle_http_connection(client, route_name.clone(), log_requests, verbose).await {
                    Ok(_) => Ok(()),
                    Err(e) => Err(e),
                }
            } else {
                // Default TCP forwarding
                handle_tcp_connection(client, target_addr.clone(), buffer_size, &mut bytes_transferred).await
            };

            match connection_result {
                Ok(_) => {
                    let duration = start_time.elapsed();
                    if verbose {
                        echo!(
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
                    stderr!(
                        "❌ [{}] {} | Connection error for {}: {}",
                        route_name, timestamp, client_addr, e
                    );
                }
            }

            drop(permit);
        });
    }
}

async fn handle_tcp_connection(
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

pub fn format_bytes(bytes: u64) -> String {
    if bytes < 1024 {
        format!("{}", bytes)
    } else if bytes < 1024 * 1024 {
        format!("{:.2} KB", bytes as f64 / 1024.0)
    } else {
        format!("{:.2} MB", bytes as f64 / (1024.0 * 1024.0))
    }
}

pub async fn run_porty_server(config: Config) -> Result<()> {
    // Start additional routes if configured
    let enabled_routes: Vec<_> = config.routes.iter()
        .filter(|r| r.enabled)
        .cloned()
        .collect();

    for route in enabled_routes {
        let listen_addr = config.listen_addr.clone();
        let max_conn = config.max_connections;
        let buffer_size = config.buffer_size_kb;
        let log_requests = config.log_requests;
        let verbose = is_true("opt_verbose");

        echo!("🔊 Additional route '{}': {}:{} -> {}:{}",
            route.name, listen_addr, route.listen_port,
            route.target_addr, route.target_port);

        let route_mode = route.mode.clone();
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
                &route_mode,
            ).await {
                error!("Route {} failed: {}", route.name, e);
            }
        });
    }

    // Run main route (default to TCP mode)
    run_route(
        "main",
        &config.listen_addr,
        config.listen_port,
        &config.target_addr,
        config.target_port,
        config.max_connections,
        config.buffer_size_kb,
        config.log_requests,
        is_true("opt_verbose"),
        "tcp",  // Main route defaults to TCP
    ).await?;

    Ok(())
}