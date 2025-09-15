# Porty

A lightweight, high-performance TCP/HTTP port forwarder and dynamic proxy built with Rust, Tokio, and the RSB framework. Features dynamic HTTP routing via query parameters, enabling runtime backend selection without configuration changes.

## Features

- **Dynamic HTTP Routing**: Route to any backend via `?porty_host=X&porty_port=Y` query parameters
- **Multi-protocol support**: TCP forwarding and HTTP dynamic routing in the same binary
- **Multiple routes**: Configure unlimited forwarding rules with per-route protocol modes
- **Zero-configuration proxy**: HTTP mode enables fully dynamic routing without config changes
- **Connection pooling**: Configurable connection limits with semaphore-based control
- **Enhanced HTTP Logging**: Configurable log levels (none/basic/verbose) with response status codes, sizes, and timing
- **Host Header Routing**: Route based on Host headers with config-driven backend mapping
- **RSB framework**: Professional CLI with built-in commands (help, inspect, stack)
- **Lightweight**: ~4MB binary with minimal dependencies
- **High performance**: Tokio async runtime with zero-copy streaming

## Quick Start

### Installation

```bash
# Clone and build
git clone <repo-url>
cd porty
cargo build --release

# Binary will be at target/release/porty
```

### Basic Usage

```bash
# Generate example config
./porty --generate-config

# Run with default config
./porty

# Override config settings
./porty --listen-port 9000 --target-port 3000 --verbose

# Run as daemon
./porty --daemon
```

## Configuration

Porty uses TOML configuration files. Generate an example with `--generate-config`:

```toml
# Main forwarding configuration
listen_addr = "0.0.0.0"
listen_port = 8080
target_addr = "127.0.0.1"
target_port = 80
max_connections = 100
buffer_size_kb = 8
log_requests = true

# Additional routes (optional)
[[routes]]
name = "api"
listen_port = 3000
target_addr = "127.0.0.1"
target_port = 3001
enabled = true

[[routes]]
name = "dynamic"
listen_port = 8080
mode = "http"
enabled = true
```

## Usage Examples

### TCP Port Forwarding

Basic port forwarding from local port 8080 to service on port 3000:

```bash
./porty --listen-port 8080 --target-port 3000
```

### HTTP Dynamic Routing (✨ NEW!)

Configure a route with `mode = "http"` to enable dynamic routing:

```toml
[[routes]]
name = "dynamic"
listen_port = 9090
mode = "http"  # Enable HTTP dynamic routing
enabled = true
```

Then route to ANY backend via query parameters:

```bash
# Route to api.internal:3000
curl "http://localhost:9090/users?id=123&porty_host=api.internal&porty_port=3000"

# Route to staging.example.com:443
curl "http://localhost:9090/api/v1/data?porty_host=staging.example.com&porty_port=443"

# Route to localhost:8000 for local development
curl "http://localhost:9090/health?porty_host=localhost&porty_port=8000"
```

**How it works:**
- Porty extracts `porty_host` and `porty_port` from query parameters
- Strips these parameters from the forwarded request
- Forwards clean request: `GET /users?id=123` → `api.internal:3000`
- Returns response with all headers and body intact
- No configuration needed - fully dynamic routing!

### Host Header Routing

Configure static host-based routing for domain names:

```toml
[[routes]]
name = "api-service"
listen_port = 9080
target_addr = "internal-api.company.com"
target_port = 80
mode = "http"
host = "api.example.com"  # Route requests with this Host header
log_level = "verbose"     # Detailed logging
```

```bash
# This request will be routed to internal-api.company.com:80
curl -H "Host: api.example.com" "http://localhost:9080/api/users"
```

### Enhanced HTTP Logging

Configure logging detail per route:

```toml
[[routes]]
name = "production-api"
listen_port = 8080
mode = "http"
log_level = "basic"    # Options: "none", "basic", "verbose"
```

**Log Levels:**
- `none` - No HTTP request/response logging
- `basic` - Request summary with response status and timing
- `verbose` - Full headers, body sizes, and performance metrics

**Sample Output:**
```
🔄 [api-route] 2025-09-15 01:23:45.123 | GET /api/users?id=123
   ├─ From: 192.168.1.100:54321
   ├─ To: internal-api.com:80 (dynamic)
✅ [api-route] 2025-09-15 01:23:45.234 | HTTP/1.1 200 OK (111ms)
   └─ Body: 1,234 bytes
```

### Multiple Routes

Configure multiple forwarding rules in `config.toml`:

```toml
# Web traffic
[[routes]]
name = "web"
listen_port = 80
target_addr = "web-server.local"
target_port = 8080
enabled = true

# API traffic
[[routes]]
name = "api"
listen_port = 3000
target_addr = "api-server.local"
target_port = 3001
enabled = true

# Dynamic routing
[[routes]]
name = "dynamic"
listen_port = 9000
mode = "http"
enabled = true
```

## Command Line Options

```
Usage: porty [OPTIONS]

Options:
  -c, --config <FILE>           Config file path [default: config.toml]
      --generate-config         Generate example config file
  -l, --listen-addr <ADDR>      Override listen address
  -p, --listen-port <PORT>      Override listen port
  -t, --target-addr <ADDR>      Override target address
  -P, --target-port <PORT>      Override target port
  -d, --daemon                  Run as daemon (detach from terminal)
  -v, --verbose                 Verbose output (show connection details)
```

## Use Cases

### Development & Testing
- **API development**: Route requests to different service versions
- **Mock testing**: Dynamically switch between real and mock services
- **Load testing**: Forward traffic to multiple backend instances

### Production & Deployment
- **Service proxy**: Simple reverse proxy for internal services
- **Legacy integration**: Route to services with non-configurable endpoints
- **Container networking**: Bridge between container and host networks

### Network Administration
- **Port mapping**: Map external ports to internal services
- **Protocol bridging**: Convert between different network protocols
- **Traffic monitoring**: Log and analyze connection patterns

## Architecture

Porty demonstrates the power of the RSB framework with an incredibly lean architecture:

- **`main.rs`**: Just 27 lines! RSB dispatch pattern and command routing
- **`args.rs`**: RSB CLI integration (can be removed after full migration)
- **`cfg.rs`**: Configuration file handling and generation
- **`net.rs`**: TCP/HTTP routing logic and connection management
- **`http.rs`**: HTTP parsing and dynamic routing implementation

The RSB transformation achieved **78% code reduction** in main.rs (122 → 27 lines) while gaining professional CLI features like built-in help, inspect, and stack commands.

### Performance Characteristics

- **Async I/O**: Built on Tokio for high concurrency
- **Zero-copy forwarding**: Efficient data transfer with minimal overhead
- **Connection pooling**: Configurable limits prevent resource exhaustion
- **Memory efficient**: Small binary size (~4MB) and low memory footprint

## Configuration Reference

### Main Configuration

| Field | Type | Default | Description |
|-------|------|---------|-------------|
| `listen_addr` | string | "0.0.0.0" | Address to bind to |
| `listen_port` | integer | 8080 | Port to listen on |
| `target_addr` | string | "127.0.0.1" | Target server address |
| `target_port` | integer | 80 | Target server port |
| `max_connections` | integer | 100 | Maximum concurrent connections |
| `buffer_size_kb` | integer | 8 | Buffer size for data transfer |
| `log_requests` | boolean | true | Enable request logging |

### Route Configuration

| Field | Type | Default | Description |
|-------|------|---------|-------------|
| `name` | string | required | Route identifier |
| `listen_port` | integer | required | Port to listen on |
| `target_addr` | string | required | Target server address |
| `target_port` | integer | required | Target server port |
| `enabled` | boolean | false | Enable/disable route |
| `mode` | string | "tcp" | Protocol mode: "tcp" or "http" |
| `host` | string | optional | Host header matching (future) |

## Logging Output

Porty provides detailed connection logging:

```
🚀 Porty v0.1.0 starting up
📁 Config loaded from: config.toml
🔊 Main route: 0.0.0.0:8080 -> 127.0.0.1:3000
🔄 [main] 2024-09-14 20:45:23.123 | New connection from 192.168.1.100:54321 -> forwarding to 127.0.0.1:3000
✅ [main] 2024-09-14 20:45:25.456 | Connection closed: 192.168.1.100:54321 | Duration: 2.33s | Transferred: 1.2 KB
```

### Verbose Mode

With `--verbose`, see detailed transfer information:

```
🔄 [dynamic] 2024-09-14 20:45:23.123 | GET /api/users?role=admin
   ├─ From: 192.168.1.100:54321
   ├─ To: api.internal:3000 (dynamic)
   ├─ Authorization: Bearer xxx...
   └─ Content-Type: application/json
✅ [dynamic] 2024-09-14 20:45:23.567 | 200 OK (444ms)
   └─ Body: 245 bytes
```

## Dependencies

- **Tokio**: Async runtime and networking
- **RSB**: CLI framework and configuration management
- **Anyhow**: Error handling
- **Serde**: Configuration serialization
- **TOML**: Configuration file format
- **Chrono**: Timestamp formatting

## Contributing

1. Fork the repository
2. Create a feature branch
3. Make your changes
4. Add tests for new functionality
5. Submit a pull request

## License

[Add your license here]

## Branches

- **`main`**: Stable version with Clap CLI (3.7MB binary)
- **`feature/rsb-integration`**: Enhanced version with RSB framework (4.1MB binary)

Choose the branch that best fits your needs - `main` for minimal size, `feature/rsb-integration` for advanced features and ecosystem integration.