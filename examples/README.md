# Porty Configuration Examples

This directory contains comprehensive examples showcasing all Porty features and use cases.

## Quick Start

```bash
# Run any example:
./porty start --config=examples/01-basic-tcp.toml

# Test with verbose output:
./porty start --config=examples/02-http-dynamic.toml --verbose
```

## Examples Overview

### 01. Basic TCP Forwarding (`01-basic-tcp.toml`)
**Use Case:** Simple port forwarding
**Features:** TCP mode, basic configuration
**Perfect for:** Database proxies, simple service forwarding

```bash
./porty start --config=examples/01-basic-tcp.toml
curl http://localhost:8080  # Forwards to localhost:3000
```

### 02. HTTP Dynamic Routing (`02-http-dynamic.toml`)
**Use Case:** Zero-configuration proxy
**Features:** Dynamic routing via query parameters
**Perfect for:** API testing, multi-environment development

```bash
./porty start --config=examples/02-http-dynamic.toml
curl "http://localhost:9090/api?porty_host=api.example.com&porty_port=80"
```

### 03. Host Header Routing (`03-host-routing.toml`)
**Use Case:** Multi-domain proxy
**Features:** Host header matching, fallback to dynamic routing
**Perfect for:** Reverse proxy, domain-based routing

```bash
./porty start --config=examples/03-host-routing.toml
curl -H "Host: api.example.com" "http://localhost:9080/users"
```

### 04. Production Ready (`04-production-ready.toml`)
**Use Case:** Production deployment
**Features:** Error handling, timeouts, retries, multiple routes
**Perfect for:** Load balancers, API gateways, service mesh

```bash
./porty start --config=examples/04-production-ready.toml
curl -H "Host: api.company.com" "https://localhost:443/v1/users"
```

### 05. Development Workflow (`05-development.toml`)
**Use Case:** Local development
**Features:** Development-friendly settings, verbose logging
**Perfect for:** API development, testing, debugging

```bash
./porty start --config=examples/05-development.toml
curl "http://localhost:9000/api?porty_host=staging.api.com&porty_port=443"
```

### 06. Comprehensive Showcase (`06-comprehensive.toml`)
**Use Case:** Feature demonstration
**Features:** All Porty capabilities in one config
**Perfect for:** Learning, testing, reference

```bash
./porty start --config=examples/06-comprehensive.toml
# Supports host routing, dynamic routing, TCP, HTTP, error handling
```

## Feature Matrix

| Example | TCP | HTTP | Dynamic Routing | Host Headers | Error Handling | Production Ready |
|---------|-----|------|-----------------|--------------|----------------|------------------|
| 01-basic-tcp | ✅ | ❌ | ❌ | ❌ | ❌ | ⚠️ |
| 02-http-dynamic | ❌ | ✅ | ✅ | ❌ | ✅ | ⚠️ |
| 03-host-routing | ❌ | ✅ | ✅ | ✅ | ✅ | ⚠️ |
| 04-production-ready | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ |
| 05-development | ✅ | ✅ | ✅ | ✅ | ⚠️ | ❌ |
| 06-comprehensive | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ |

## Configuration Tips

### HTTP Dynamic Routing
- Use `mode = "http"` to enable HTTP parsing
- Add `porty_host=X&porty_port=Y` to any URL for dynamic routing
- Query parameters are automatically stripped from forwarded requests

### Host Header Routing
- Set `host = "domain.com"` to route based on Host header
- Combine with dynamic routing for flexible fallback behavior
- Perfect for multi-tenant applications

### Error Handling
- `timeout_seconds` - Backend connection timeout (default: 30)
- `max_retries` - Retry attempts with exponential backoff (default: 2)
- `log_level` - "none", "basic", or "verbose" logging

### Performance Tuning
- `max_connections` - Concurrent connection limit
- `buffer_size_kb` - Data transfer buffer size
- Higher values = better performance, more memory usage

## Common Use Cases

### API Gateway
```toml
[[routes]]
mode = "http"
host = "api.company.com"
timeout_seconds = 10
max_retries = 3
log_level = "basic"
```

### Load Balancer
```toml
[[routes]]
mode = "http"
# No host = dynamic routing enabled
timeout_seconds = 5
max_retries = 2
```

### Development Proxy
```toml
[[routes]]
mode = "http"
log_level = "verbose"
timeout_seconds = 60
max_retries = 0
```

### Database Proxy
```toml
[[routes]]
mode = "tcp"
# Raw TCP forwarding
```

## Testing Examples

```bash
# Test dynamic routing
curl "http://localhost:9090/test?porty_host=httpbin.org&porty_port=80"

# Test host routing
curl -H "Host: api.localhost" "http://localhost:8080/users"

# Test error handling (timeout)
curl "http://localhost:9090/slow?porty_host=httpbin.org&porty_port=80&delay=5"

# Test TCP forwarding
redis-cli -p 6380  # If Redis proxy configured
```

Start with the example that matches your use case, then customize as needed!