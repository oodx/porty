# Porty

A lightweight, high-performance TCP/HTTP port forwarder and dynamic proxy built with Rust and Tokio. Features innovative HTTP routing via query parameters, enabling runtime backend selection without configuration changes.

## Key Innovation: Dynamic HTTP Routing

Porty's breakthrough feature is **query parameter routing** - route HTTP requests to any backend at runtime using URL parameters. This enables zero-configuration proxying and unprecedented flexibility for development, testing, and production environments.

```bash
# Route to ANY backend dynamically:
curl "http://localhost:9090/api/users?porty_host=api.example.com&porty_port=80"
curl "http://localhost:9090/health?porty_host=localhost&porty_port=8000"
```

## Features

### Core Capabilities
- **Dynamic HTTP Routing**: Route to any backend via `?porty_host=X&porty_port=Y` query parameters
- **Multi-protocol Support**: TCP forwarding and HTTP dynamic routing in the same binary
- **Zero-Configuration Proxy**: HTTP mode enables fully dynamic routing without config changes
- **Host Header Routing**: Route based on Host headers with config-driven backend mapping
- **Multiple Routes**: Configure unlimited forwarding rules with per-route protocol modes

### Performance & Reliability
- **High Performance**: Tokio async runtime with zero-copy streaming
- **Connection Pooling**: Configurable connection limits with semaphore-based control
- **HTTP Error Handling**: Robust error handling with timeouts, retries, and custom error pages
- **Enhanced HTTP Logging**: Configurable log levels (none/basic/verbose) with response status codes, sizes, and timing

### Developer Experience
- **RSB Framework Integration**: Professional CLI with structured commands and context management
- **Lightweight**: Small binary (~4MB) with minimal dependencies
- **Comprehensive Examples**: 6 configuration examples covering all use cases
- **Flexible Configuration**: TOML-based config with intelligent defaults

## Quick Start

### Installation

```bash
# Clone the repository
git clone <repo-url>
cd porty

# Build in release mode for optimal performance
cargo build --release

# Binary will be at target/release/porty
# File size: ~4MB optimized binary
```

### System Requirements

- **Rust**: 1.70+ (2021 edition)
- **OS**: Linux, macOS, Windows
- **RAM**: Minimal (configurable buffer sizes)
- **Network**: TCP socket permissions

### Basic Usage

```bash
# Generate example config
./porty generate-config

# Run with default config
./porty start

# Override config settings
./porty start --listen-port 9000 --target-port 3000 --verbose

# Run as daemon (Unix only)
./porty start --daemon
```

## Examples

Porty includes 6 comprehensive configuration examples in the `examples/` directory, each demonstrating different features and use cases:

### Quick Examples

```bash
# Basic TCP forwarding
./porty start --config=examples/01-basic-tcp.toml

# HTTP dynamic routing (route to ANY backend!)
./porty start --config=examples/02-http-dynamic.toml
curl "http://localhost:9090/api?porty_host=api.example.com&porty_port=80"

# Host header routing
./porty start --config=examples/03-host-routing.toml
curl -H "Host: api.example.com" "http://localhost:9080/users"

# Production-ready with error handling
./porty start --config=examples/04-production-ready.toml

# Development workflow
./porty start --config=examples/05-development.toml

# Comprehensive feature showcase
./porty start --config=examples/06-comprehensive.toml
```

**📁 See [`examples/README.md`](examples/README.md) for detailed documentation and comprehensive usage guide.**

## Why Porty?

### Unique Value Proposition

**Traditional proxies** require configuration changes for each new backend. **Porty's dynamic routing** enables runtime backend selection via URL parameters - no config changes, no restarts, unlimited flexibility.

**Perfect for:**
- **API Development**: Switch between environments without config changes
- **Testing**: Route to mock services, staging, or production dynamically
- **Integration**: Bridge services without hardcoded endpoints
- **DevOps**: Simplify deployment pipelines with flexible routing

### Technical Innovations

1. **Zero-Configuration Proxy**: HTTP mode with query parameter routing
2. **Dual-Protocol Support**: TCP and HTTP in the same binary with per-route configuration
3. **Intelligent Routing**: Host header matching with dynamic fallback
4. **Performance-First**: Tokio async with zero-copy streaming and connection pooling
5. **Developer-Friendly**: Rich logging, error handling, and comprehensive examples

### Comparison

| Feature | Traditional Proxy | Porty |
|---------|------------------|-------|
| Backend Selection | Config change + restart | URL parameter |
| Multi-Protocol | Separate tools | Single binary |
| Error Handling | Basic | Comprehensive with retries |
| Configuration | Complex | Simple TOML with examples |
| Performance | Varies | Rust + Tokio optimized |

## Configuration

Porty uses TOML configuration files. Generate an example with `generate-config`:

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
mode = "tcp"

[[routes]]
name = "dynamic"
listen_port = 9090
mode = "http"  # Enables dynamic routing!
enabled = true
```

## Usage Examples

### TCP Port Forwarding

Basic port forwarding from local port 8080 to service on port 3000:

```bash
./porty start --listen-port 8080 --target-port 3000
```

### HTTP Dynamic Routing ⭐

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

### HTTP Error Handling

Configure robust error handling for HTTP routes with timeouts, retries, and custom error pages:

```toml
[[routes]]
name = "production-api"
listen_port = 8080
mode = "http"
timeout_seconds = 10    # Backend connection timeout
max_retries = 3         # Retry attempts with exponential backoff
log_level = "verbose"   # Enhanced error logging
```

**Error Handling Features:**
- **Malformed Request Validation** - Graceful handling of invalid HTTP requests with descriptive error messages
- **Connection Timeouts** - Configurable timeouts prevent hanging on slow backends
- **Retry Logic** - Exponential backoff retries (100ms, 200ms, 400ms) for transient failures
- **Custom Error Pages** - Professional HTTP error responses (400, 502, 504) with proper headers

**Error Response Examples:**
```bash
# Missing routing parameters → 400 Bad Request
curl "http://localhost:8080/api"
# HTTP/1.1 400 Bad Request
# Content-Type: text/plain
# 400 Missing porty_host and porty_port parameters

# Backend connection failure → 502 Bad Gateway (after retries)
curl "http://localhost:8080/api?porty_host=down-service&porty_port=8080"
# HTTP/1.1 502 Bad Gateway
# Content-Type: text/plain
# 502 Backend connection failed after retries
```

**Error Handling Configuration:**
- `timeout_seconds` - Backend connection timeout in seconds (default: 30)
- `max_retries` - Maximum retry attempts with exponential backoff (default: 2)
- `log_level` - Error detail logging: "none", "basic", or "verbose"

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

## Command Line Interface

```
Usage: porty [COMMAND] [OPTIONS]

Commands:
  start              Start the proxy server (default)
  generate-config    Generate example configuration
  help               Show help message
  version            Show version information
  inspect            Runtime inspection (RSB framework)
  stack              Debug stack traces (RSB framework)

Options:
  --config <FILE>            Config file path [default: config.toml]
  --listen-addr <ADDR>       Override listen address
  --listen-port <PORT>       Override listen port
  --target-addr <ADDR>       Override target address
  --target-port <PORT>       Override target port
  --daemon                   Run as daemon (Unix only)
  --verbose                  Enable verbose logging
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

Porty is built with a modern, modular architecture leveraging Rust's performance and safety features:

### Core Architecture

**Technology Stack:**
- **Rust + Tokio**: Async runtime for high-performance concurrent connections
- **RSB Framework**: Professional CLI with context management and structured commands
- **TOML Configuration**: Human-readable config with intelligent defaults
- **Zero-Copy Streaming**: Efficient data transfer with minimal memory overhead

**File Structure:**
- **`main.rs`**: Entry point with RSB command dispatch and daemon support
- **`cfg.rs`**: Configuration parsing, validation, and example generation
- **`net.rs`**: TCP connection handling, routing, and concurrency control
- **`http.rs`**: HTTP parsing, dynamic routing, and query parameter extraction
- **`lib.rs`**: Module organization and public API

### Request Flow

```
Client Request → Porty Listener → Route Matcher → Protocol Handler → Backend
                                        ↓
                              TCP Mode: Direct forwarding
                              HTTP Mode: Parse → Route → Forward
                                        ↓
                              Dynamic: Extract porty_host/porty_port
                              Static: Use configured target
                              Host-based: Match Host header
```

### RSB Framework Integration

Porty leverages the RSB framework for professional CLI capabilities:

```bash
./porty help       # Built-in help system
./porty inspect    # Runtime inspection and diagnostics
./porty stack      # Debug stack traces and context
```

**RSB Benefits:**
- Global context management for CLI arguments
- Structured output with `echo!()` and `stderr!()` macros
- Command routing and parameter handling
- Professional command-line interface

### Performance Characteristics

- **Async I/O**: Built on Tokio for high concurrency and non-blocking operations
- **Zero-Copy Streaming**: Efficient data transfer with minimal memory overhead
- **Connection Pooling**: Semaphore-based concurrency control prevents resource exhaustion
- **Memory Efficient**: Small binary size (~4MB) and low memory footprint
- **Configurable Buffers**: Tunable buffer sizes for optimal performance vs memory usage

## Configuration Reference

### Main Configuration

| Field | Type | Default | Description |
|-------|------|---------|-------------|
| `listen_addr` | string | "0.0.0.0" | Address to bind to |
| `listen_port` | integer | 8080 | Port to listen on |
| `target_addr` | string | "127.0.0.1" | Target server address |
| `target_port` | integer | 80 | Target server port |
| `max_connections` | integer | 100 | Maximum concurrent connections |
| `buffer_size_kb` | integer | 8 | Buffer size for data transfer (KB) |
| `log_requests` | boolean | true | Enable request logging |
| `log_format` | string | "default" | Log format style |

### Route Configuration

| Field | Type | Default | Description |
|-------|------|---------|-------------|
| `name` | string | required | Route identifier (must be unique) |
| `listen_port` | integer | required | Port to listen on |
| `target_addr` | string | required* | Target server address |
| `target_port` | integer | required* | Target server port |
| `enabled` | boolean | false | Enable/disable route |
| `mode` | string | "tcp" | Protocol mode: "tcp" or "http" |
| `host` | string | optional | Host header matching (HTTP mode only) |
| `log_level` | string | "basic" | Log detail: "none", "basic", "verbose" |
| `timeout_seconds` | integer | 30 | Backend connection timeout (HTTP mode) |
| `max_retries` | integer | 2 | Maximum retry attempts (HTTP mode) |

*For HTTP dynamic routing routes, `target_addr` and `target_port` are overridden by query parameters.

## Logging & Monitoring

Porty provides structured logging with configurable detail levels:

### Startup Logging
```
🚀 Porty v0.1.0 starting up
📁 Config loaded from: config.toml
🔊 Main route: 0.0.0.0:8080 -> 127.0.0.1:3000
```

### Connection Logging

**Basic Level (`log_level = "basic"`):**
```
🔄 [main] 2025-09-20 20:45:23.123 | New connection from 192.168.1.100:54321
✅ [main] 2025-09-20 20:45:25.456 | Connection closed | Duration: 2.33s | Transferred: 1.2 KB
```

**Verbose Level (`log_level = "verbose"` or `--verbose`):**
```
🔄 [dynamic] 2025-09-20 20:45:23.123 | GET /api/users?role=admin
   ├─ From: 192.168.1.100:54321
   ├─ To: api.internal:3000 (dynamic)
   ├─ Authorization: Bearer xxx...
   └─ Content-Type: application/json
✅ [dynamic] 2025-09-20 20:45:23.567 | HTTP/1.1 200 OK (444ms)
   └─ Body: 245 bytes
```

**Log Levels:**
- `none`: No request/response logging
- `basic`: Connection summaries with timing and data transfer
- `verbose`: Full request/response details with headers and performance metrics

## Dependencies

**Core Dependencies:**
- **Tokio**: Async runtime and networking (`tokio = { version = "1", features = ["full"] }`)
- **RSB**: CLI framework and context management (`rsb = { path = "../rsb" }`)
- **Anyhow**: Error handling and context (`anyhow = "1"`)
- **Serde**: Configuration serialization (`serde = { version = "1", features = ["derive"] }`)
- **TOML**: Configuration file parsing (`toml = "0.9"`)

**Supporting Libraries:**
- **Log + Env Logger**: Structured logging (`log = "0.4"`, `env_logger = "0.11"`)
- **Chrono**: Timestamp formatting (`chrono = "0.4"`)

All dependencies are mature, well-maintained crates with minimal security surface area.

## Project Status

**Current Version:** v0.1.0 (Active Development)

Porty is in active development with a focus on performance, reliability, and developer experience. The core functionality is stable and production-ready for many use cases.

**Current Phase:** Performance & Reliability (Phase 4)
- ✅ Core TCP/HTTP routing complete
- ✅ Dynamic routing via query parameters
- ✅ Host header routing
- ✅ Comprehensive error handling
- 🔄 Performance benchmarking and optimization
- 🔄 Enhanced test coverage and validation

**Getting Started:** Read [`START.txt`](START.txt) for the complete development workflow and project context.

## Contributing

1. Fork the repository
2. Read [`docs/procs/PROCESS.txt`](docs/procs/PROCESS.txt) for development workflow
3. Create a feature branch
4. Implement changes with comprehensive tests
5. Run `./bin/validate-docs.sh` to ensure documentation integrity
6. Submit a pull request

**Development Documentation:** See [`docs/`](docs/) directory for architecture, processes, and reference materials.

## License

See [`LICENSE`](LICENSE) file for license details.