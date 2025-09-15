# Porty Documentation

Comprehensive documentation for Porty - the lightweight, high-performance TCP/HTTP port forwarder and dynamic proxy.

## 📚 Documentation Index

### Core Features
- **[Query Parameter Routing Guide](query-parameter-routing.md)** - Complete guide to HTTP dynamic routing with `?porty_host=X&porty_port=Y`
- **[Performance Benchmarking](performance-benchmarking.md)** - Benchmarking methodologies, optimization strategies, and performance characteristics

### Configuration Examples
- **[Examples Directory](../examples/README.md)** - Comprehensive configuration examples for all use cases
- **[Main README](../README.md)** - Core documentation with RSB framework integration details

## 🚀 Quick Navigation

### For New Users
1. Start with the **[Main README](../README.md)** for overview and basic usage
2. Try **[Basic Examples](../examples/README.md)** to get hands-on experience
3. Read **[Query Parameter Routing](query-parameter-routing.md)** for advanced HTTP features

### For Developers
1. Review **[RSB Architecture](../README.md#architecture)** in the main README
2. Study **[Performance Guide](performance-benchmarking.md)** for optimization techniques
3. Explore **[Production Examples](../examples/04-production-ready.toml)** for deployment patterns

### For Operations
1. Use **[Production Configuration](../examples/04-production-ready.toml)** as a starting point
2. Review **[Performance Benchmarking](performance-benchmarking.md)** for monitoring strategies
3. Reference **[Configuration Guide](../README.md#configuration-reference)** for tuning parameters

## 📖 Feature Documentation

### HTTP Dynamic Routing
Route to **any backend** at runtime using query parameters:
```bash
curl "http://localhost:9090/api?porty_host=api.example.com&porty_port=443"
```
**→ [Complete Guide](query-parameter-routing.md)**

### Host Header Routing
Route requests based on Host headers with fallback to dynamic routing:
```bash
curl -H "Host: api.company.com" "http://localhost:9080/users"
```
**→ [Example Configuration](../examples/03-host-routing.toml)**

### Error Handling
Robust error handling with timeouts, retries, and custom error pages:
```toml
timeout_seconds = 10
max_retries = 3
log_level = "verbose"
```
**→ [Production Example](../examples/04-production-ready.toml)**

### RSB Framework Integration
Professional CLI with built-in commands and global context:
```bash
./porty help       # Built-in help
./porty inspect    # Runtime inspection
./porty stack      # Debug traces
```
**→ [Architecture Details](../README.md#rsb-framework-integration)**

## 🎯 Use Case Guides

### Development Workflow
```bash
./porty start --config=examples/05-development.toml
curl "http://localhost:9000/api?porty_host=staging.api.com&porty_port=443"
```

### Production Deployment
```bash
./porty start --config=examples/04-production-ready.toml
```

### Load Balancing
```bash
curl "http://localhost:9090/health?porty_host=backend1.internal&porty_port=8080"
curl "http://localhost:9090/health?porty_host=backend2.internal&porty_port=8080"
```

### API Testing
```bash
# Test across environments dynamically
for env in dev staging prod; do
  curl "http://localhost:9090/health?porty_host=${env}.api.com&porty_port=443"
done
```

## 🔧 Configuration Reference

### Quick Configuration Templates

**Basic TCP Forwarding:**
```toml
listen_port = 8080
target_addr = "127.0.0.1"
target_port = 3000
```

**HTTP Dynamic Routing:**
```toml
[[routes]]
name = "dynamic"
listen_port = 9090
mode = "http"
enabled = true
```

**Production HTTP with Error Handling:**
```toml
[[routes]]
name = "api"
listen_port = 443
mode = "http"
host = "api.company.com"
timeout_seconds = 10
max_retries = 3
log_level = "basic"
```

## 📊 Performance Quick Reference

### Typical Performance (4-core, 16GB RAM)
- **TCP Mode:** 45,000 req/sec, 1.2ms p50 latency
- **HTTP Mode:** 38,000 req/sec, 1.8ms p50 latency
- **Memory:** 4MB base + 8KB per connection
- **Connections:** 1000+ concurrent (configurable)

### Optimization Tips
```toml
# High throughput
max_connections = 2000
buffer_size_kb = 64
log_level = "none"

# Low latency
buffer_size_kb = 4
timeout_seconds = 1
max_retries = 0
```

**→ [Complete Performance Guide](performance-benchmarking.md)**

## 🤝 Contributing

Found an issue or want to improve the documentation?

1. **Examples:** Add new configuration examples in [`examples/`](../examples/)
2. **Guides:** Enhance existing guides or create new ones
3. **Performance:** Submit benchmark results from your environment
4. **Feedback:** Open issues for documentation improvements

## 📝 Documentation Structure

```
docs/
├── README.md                     # This index (you are here)
├── query-parameter-routing.md    # HTTP dynamic routing guide
└── performance-benchmarking.md   # Performance and optimization guide

examples/
├── README.md                     # Configuration examples index
├── 01-basic-tcp.toml            # Simple TCP forwarding
├── 02-http-dynamic.toml         # HTTP dynamic routing
├── 03-host-routing.toml         # Host header routing
├── 04-production-ready.toml     # Production configuration
├── 05-development.toml          # Development workflow
└── 06-comprehensive.toml        # All features showcase
```

Start exploring with the **[Main README](../README.md)** or jump into **[Examples](../examples/README.md)** for hands-on learning!