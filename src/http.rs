// HTTP parsing and dynamic routing

use anyhow::{Context, Result};
use chrono::Local;
use std::collections::HashMap;
use tokio::io::{AsyncBufReadExt, AsyncReadExt, AsyncWriteExt, BufReader};
use tokio::net::TcpStream;
use rsb::prelude::*;

#[derive(Debug, Clone)]
pub struct HttpRequest {
    pub method: String,
    pub path: String,
    pub query: String,
    pub headers: HashMap<String, String>,
    pub body: Vec<u8>,
}

#[derive(Debug)]
pub struct DynamicRoute {
    pub target_host: String,
    pub target_port: u16,
}

#[derive(Debug)]
pub struct ResponseInfo {
    pub status: String,
    pub body_size: usize,
}

pub async fn handle_http_connection(
    mut client: TcpStream,
    route_name: String,
    default_target: String,
    route_host: Option<String>,
    log_requests: bool,
    verbose: bool,
    log_level: &str,
    timeout_seconds: u64,
    max_retries: u32,
) -> Result<()> {
    let client_addr = client.peer_addr()?.to_string();

    // Parse HTTP request
    let request = parse_http_request(&mut client).await?;

    // Check for host header matching first (if configured)
    let target_route = if let Some(expected_host) = &route_host {
        if let Some(incoming_host) = request.headers.get("host") {
            if incoming_host == expected_host {
                // Host header matches, use configured route target
                let parts: Vec<&str> = default_target.split(':').collect();
                Some(DynamicRoute {
                    target_host: parts[0].to_string(),
                    target_port: parts.get(1)
                        .and_then(|p| p.parse().ok())
                        .unwrap_or(80),
                })
            } else {
                // Host header doesn't match, check for dynamic routing fallback
                extract_dynamic_route(&request.query)?
            }
        } else {
            // No host header, check for dynamic routing fallback
            extract_dynamic_route(&request.query)?
        }
    } else {
        // No host matching configured, check for dynamic routing
        extract_dynamic_route(&request.query)?
    };

    // If no route determined, use default target
    let dynamic_route = target_route.or_else(|| {
        // Parse default target as fallback
        let parts: Vec<&str> = default_target.split(':').collect();
        if parts.len() == 2 {
            Some(DynamicRoute {
                target_host: parts[0].to_string(),
                target_port: parts[1].parse().unwrap_or(80),
            })
        } else {
            None
        }
    });

    if let Some(route) = dynamic_route {
        if log_requests && log_level != "none" {
            let timestamp = Local::now().format("%Y-%m-%d %H:%M:%S%.3f");
            echo!(
                "🔄 [{}] {} | {} {}?{}",
                route_name, timestamp, request.method, request.path, request.query
            );
            echo!("   ├─ From: {}", client_addr);
            echo!(
                "   ├─ To: {}:{} (dynamic)",
                route.target_host, route.target_port
            );
            if log_level == "verbose" || verbose {
                for (key, value) in &request.headers {
                    echo!("   ├─ {}: {}", key, value);
                }
            }
        }

        let start_time = std::time::Instant::now();

        // Forward the cleaned request with retry logic
        match forward_http_request_with_retry(request, route, client, timeout_seconds, max_retries).await {
            Ok(response_info) => {
                if log_requests && log_level != "none" {
                    let duration = start_time.elapsed();
                    let timestamp = Local::now().format("%Y-%m-%d %H:%M:%S%.3f");
                    echo!(
                        "✅ [{}] {} | {} ({:.0}ms)",
                        route_name,
                        timestamp,
                        response_info.status,
                        duration.as_millis()
                    );
                    if log_level == "verbose" || verbose {
                        echo!("   └─ Body: {} bytes", response_info.body_size);
                    }
                }
            }
            Err(e) => {
                if log_requests {
                    let timestamp = Local::now().format("%Y-%m-%d %H:%M:%S%.3f");
                    stderr!("❌ [{}] {} | Error: {}", route_name, timestamp, e);
                }
                return Err(e);
            }
        }
    } else {
        // No dynamic routing, send 400 Bad Request
        let _ = send_error_response(&mut client, 400, "Missing porty_host and porty_port parameters").await;
    }

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
        return Err(anyhow::anyhow!("Malformed HTTP request: Empty request"));
    }

    // Parse request line: "GET /path?query HTTP/1.1"
    let request_line = &lines[0];
    let parts: Vec<&str> = request_line.split_whitespace().collect();
    if parts.len() < 2 {
        return Err(anyhow::anyhow!("Malformed HTTP request: Invalid request line '{}'", request_line));
    }

    if parts.len() >= 3 && !parts[2].starts_with("HTTP/") {
        return Err(anyhow::anyhow!("Malformed HTTP request: Invalid HTTP version '{}'", parts[2]));
    }

    let method = parts[0].to_string();
    let url_part = parts[1];
    let (path, query) = if let Some(pos) = url_part.find('?') {
        (url_part[..pos].to_string(), url_part[pos + 1..].to_string())
    } else {
        (url_part.to_string(), String::new())
    };

    // Parse headers
    for line in &lines[1..] {
        if let Some(pos) = line.find(':') {
            let key = line[..pos].trim().to_lowercase();
            let value = line[pos + 1..].trim().to_string();
            if key.is_empty() {
                return Err(anyhow::anyhow!("Malformed HTTP request: Empty header name"));
            }
            headers.insert(key, value);
        } else if !line.trim().is_empty() {
            return Err(anyhow::anyhow!("Malformed HTTP request: Invalid header format '{}'", line));
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
            let value = &pair[pos + 1..];
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
    query
        .split('&')
        .filter(|pair| !pair.starts_with("porty_host=") && !pair.starts_with("porty_port="))
        .collect::<Vec<_>>()
        .join("&")
}

async fn forward_http_request_with_retry(
    request: HttpRequest,
    route: DynamicRoute,
    mut client: TcpStream,
    timeout_seconds: u64,
    max_retries: u32,
) -> Result<ResponseInfo> {
    let mut last_error = None;

    for attempt in 0..=max_retries {
        if attempt > 0 {
            // Wait before retry (exponential backoff)
            let delay = std::time::Duration::from_millis(100 * (1u64 << (attempt - 1)));
            tokio::time::sleep(delay).await;
        }

        match forward_http_request_with_client(request.clone(), route.clone(), &mut client, timeout_seconds).await {
            Ok(response) => return Ok(response),
            Err(e) => {
                last_error = Some(e);
                if attempt < max_retries {
                    stderr!("⚠️  HTTP request failed, retrying... (attempt {}/{})", attempt + 1, max_retries + 1);
                }
            }
        }
    }

    // Send error response to client after all retries failed
    let _ = send_error_response(&mut client, 502, "Backend connection failed after retries").await;

    Err(last_error.unwrap())
}

impl Clone for DynamicRoute {
    fn clone(&self) -> Self {
        Self {
            target_host: self.target_host.clone(),
            target_port: self.target_port,
        }
    }
}


async fn forward_http_request_with_client(
    request: HttpRequest,
    route: DynamicRoute,
    client: &mut TcpStream,
    timeout_seconds: u64,
) -> Result<ResponseInfo> {
    let timeout = std::time::Duration::from_secs(timeout_seconds);

    tokio::time::timeout(timeout, forward_http_request_internal(request, route, client)).await
        .map_err(|_| anyhow::anyhow!("Request timeout after {} seconds", timeout_seconds))?
}

async fn forward_http_request_internal(
    request: HttpRequest,
    route: DynamicRoute,
    client: &mut TcpStream,
) -> Result<ResponseInfo> {
    // Connect to target
    let target_addr = format!("{}:{}", route.target_host, route.target_port);
    let mut target = TcpStream::connect(&target_addr)
        .await
        .with_context(|| format!("Failed to connect to {}", target_addr))?;

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
                if !headers_read && !response_buf.is_empty() {
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
        status: if status_line.is_empty() {
            "Unknown".to_string()
        } else {
            status_line
        },
        body_size: total_bytes,
    })
}

async fn send_error_response(client: &mut TcpStream, status_code: u16, message: &str) -> Result<()> {
    let status_text = match status_code {
        400 => "Bad Request",
        404 => "Not Found",
        500 => "Internal Server Error",
        502 => "Bad Gateway",
        503 => "Service Unavailable",
        504 => "Gateway Timeout",
        _ => "Error",
    };

    let body = format!("{} {}", status_code, message);
    let response = format!(
        "HTTP/1.1 {} {}\r\nContent-Type: text/plain\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
        status_code, status_text, body.len(), body
    );

    client.write_all(response.as_bytes()).await?;
    Ok(())
}