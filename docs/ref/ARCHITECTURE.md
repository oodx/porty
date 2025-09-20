# Porty Architecture

## System Overview

Porty is a high-performance TCP/HTTP port forwarder built with Rust, leveraging the Tokio async runtime for concurrent connection handling and the RSB framework for CLI management.

## Core Components

### 1. Main Entry Point (`main.rs`)
- RSB framework bootstrap and context initialization
- Command dispatch (start, help, inspect, etc.)
- Daemon mode support
- 156 lines with modular design

### 2. Configuration Module (`cfg.rs`)
- TOML-based configuration parsing
- Multi-route support
- Dynamic configuration generation
- Validation and defaults

### 3. Network Module (`net.rs`)
- TCP connection handling
- Route management with semaphore-based concurrency control
- Mode-based routing (TCP vs HTTP)
- Connection pooling and limits

### 4. HTTP Module (`http.rs`)
- HTTP request parsing and routing
- Query parameter extraction for dynamic routing
- Host header-based routing
- Request/response streaming

## Data Flow

```
Client Request → Porty Listener → Route Matcher → Mode Handler → Target Backend
                                         ↓
                                   TCP Handler
                                   HTTP Handler
```

## Key Design Patterns

### Async/Await Pattern
All I/O operations use Tokio's async runtime for non-blocking execution.

### Zero-Copy Streaming
Data is streamed between client and backend without unnecessary copying.

### Semaphore-Based Connection Limiting
Prevents resource exhaustion through configurable connection limits.

### Mode-Based Routing
Routes can operate in either TCP (raw forwarding) or HTTP (application-aware) mode.

## Configuration Structure

```rust
struct Config {
    listen_addr: String,
    listen_port: u16,
    target_addr: String,
    target_port: u16,
    routes: Vec<Route>,
    // ... other fields
}

struct Route {
    name: String,
    listen_port: u16,
    target_addr: String,
    target_port: u16,
    mode: String,  // "tcp" or "http"
    host: Option<String>,  // For HTTP host routing
    enabled: bool,
}
```

## Performance Characteristics

- **Concurrency**: Tokio runtime with work-stealing scheduler
- **Memory**: Minimal allocations with buffer reuse
- **Latency**: Sub-millisecond routing decisions
- **Throughput**: Limited primarily by network I/O

## RSB Framework Integration

The RSB framework provides:
- Global context management (`opt_*` variables)
- Structured output (`echo!()`, `stderr!()` macros)
- Built-in commands (help, inspect, stack)
- Configuration file discovery
- Error handling patterns