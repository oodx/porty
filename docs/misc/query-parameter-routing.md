# Query Parameter Routing Guide

Porty's HTTP dynamic routing feature allows you to route requests to **any backend** at runtime using query parameters. This enables zero-configuration proxying and flexible service routing.

## How It Works

When Porty receives an HTTP request with `porty_host` and `porty_port` query parameters:

1. **Extracts** the target backend from query parameters
2. **Strips** these parameters from the forwarded request
3. **Routes** the clean request to the specified backend
4. **Returns** the response with all headers and body intact

## Basic Usage

### Configuration

Enable HTTP mode for dynamic routing:

```toml
[[routes]]
name = "dynamic-proxy"
listen_port = 9090
target_addr = "unused"      # Will be overridden
target_port = 0
enabled = true
mode = "http"               # Enable HTTP parsing!
```

### Request Format

```
GET /path?param1=value1&porty_host=TARGET_HOST&porty_port=TARGET_PORT&param2=value2
```

**Result:** Clean request sent to target:
```
GET /path?param1=value1&param2=value2
```

## Examples

### API Routing

```bash
# Route to production API
curl "http://localhost:9090/api/users?porty_host=api.company.com&porty_port=443"

# Route to staging environment
curl "http://localhost:9090/api/users?porty_host=staging-api.internal&porty_port=8080"

# Route to local development
curl "http://localhost:9090/api/users?porty_host=localhost&porty_port=3000"
```

### Multi-Environment Testing

```bash
# Test same endpoint across environments
ENDPOINT="/api/v1/health"

# Production
curl "http://localhost:9090${ENDPOINT}?porty_host=prod.api.com&porty_port=443"

# Staging
curl "http://localhost:9090${ENDPOINT}?porty_host=staging.api.com&porty_port=443"

# Development
curl "http://localhost:9090${ENDPOINT}?porty_host=localhost&porty_port=8080"

# Mock services
curl "http://localhost:9090${ENDPOINT}?porty_host=mock.localhost&porty_port=3001"
```

### Load Balancing Simulation

```bash
# Distribute requests across backends
for i in {1..3}; do
  curl "http://localhost:9090/api/data?porty_host=backend${i}.internal&porty_port=8080"
done
```

## Advanced Features

### Error Handling

Configure robust error handling for dynamic routing:

```toml
[[routes]]
name = "dynamic-proxy"
listen_port = 9090
mode = "http"
timeout_seconds = 10        # Backend timeout
max_retries = 3             # Retry failed connections
log_level = "verbose"       # Full routing logs
```

**Error Responses:**

```bash
# Missing parameters → 400 Bad Request
curl "http://localhost:9090/api"
# HTTP/1.1 400 Bad Request
# 400 Missing porty_host and porty_port parameters

# Connection failure → 502 Bad Gateway (after retries)
curl "http://localhost:9090/api?porty_host=down.service&porty_port=9999"
# HTTP/1.1 502 Bad Gateway
# 502 Backend connection failed after retries
```

### Host Header Combinations

Combine with host header routing for fallback behavior:

```toml
[[routes]]
name = "smart-proxy"
listen_port = 8080
target_addr = "primary.backend.com"
target_port = 443
mode = "http"
host = "api.company.com"    # Primary routing by host
# Falls back to dynamic routing if host doesn't match
```

**Usage:**
```bash
# Host header match → routes to primary.backend.com
curl -H "Host: api.company.com" "http://localhost:8080/users"

# Different host → uses dynamic routing
curl -H "Host: other.com" "http://localhost:8080/users?porty_host=alt.backend&porty_port=80"
```

## Query Parameter Details

### Required Parameters

- `porty_host` - Target hostname or IP address
- `porty_port` - Target port number (1-65535)

### Parameter Processing

**Input Request:**
```
GET /api/users?id=123&porty_host=api.example.com&porty_port=80&filter=active HTTP/1.1
Host: localhost:9090
Authorization: Bearer token123
```

**Forwarded Request:**
```
GET /api/users?id=123&filter=active HTTP/1.1
Host: api.example.com
Authorization: Bearer token123
```

**Key Points:**
- ✅ `porty_*` parameters are stripped
- ✅ Other query parameters are preserved
- ✅ All headers are forwarded
- ✅ `Host` header is updated to target
- ✅ Request body is forwarded unchanged

## Use Cases

### 1. API Development

```bash
# Test different API versions
curl "http://localhost:9090/v1/users?porty_host=api-v1.dev&porty_port=8080"
curl "http://localhost:9090/v2/users?porty_host=api-v2.dev&porty_port=8080"
```

### 2. A/B Testing

```bash
# Route traffic to different implementations
curl "http://localhost:9090/feature?porty_host=variant-a.test&porty_port=8080"
curl "http://localhost:9090/feature?porty_host=variant-b.test&porty_port=8080"
```

### 3. Service Discovery

```bash
# Dynamic service routing without configuration changes
SERVICE_HOST=$(consul-template "{{ range service \"api\" }}{{ .Address }}{{ end }}")
curl "http://localhost:9090/health?porty_host=${SERVICE_HOST}&porty_port=8080"
```

### 4. Multi-Tenant Applications

```bash
# Route to tenant-specific backends
TENANT="tenant123"
curl "http://localhost:9090/app/data?porty_host=${TENANT}.backend.com&porty_port=8080"
```

### 5. Integration Testing

```bash
# Test against real services dynamically
TEST_CASES=(
  "localhost:3000"          # Local
  "staging.api.com:443"     # Staging
  "prod.api.com:443"        # Production
)

for target in "${TEST_CASES[@]}"; do
  host=${target%:*}
  port=${target#*:}
  curl "http://localhost:9090/health?porty_host=${host}&porty_port=${port}"
done
```

## Security Considerations

### Network Access
- Porty can connect to **any host/port** specified in parameters
- Consider firewall rules to limit accessible targets
- Use internal networks for sensitive services

### Input Validation
- Port numbers are validated (1-65535)
- Invalid hostnames are handled gracefully
- Malformed parameters result in 400 errors

### Best Practices
```toml
# Production configuration
[[routes]]
name = "secure-proxy"
mode = "http"
timeout_seconds = 5         # Short timeout for security
max_retries = 1             # Minimal retries
log_level = "basic"         # Log for monitoring
```

## Debugging

### Verbose Logging

```toml
log_level = "verbose"       # Enable detailed logs
```

**Sample Output:**
```
🔄 [dynamic-proxy] 2025-09-15 01:23:45.123 | GET /api/users?id=123&porty_host=api.example.com&porty_port=80
   ├─ From: 192.168.1.100:54321
   ├─ To: api.example.com:80 (dynamic)
   ├─ authorization: Bearer xxx...
   ├─ content-type: application/json
✅ [dynamic-proxy] 2025-09-15 01:23:45.567 | HTTP/1.1 200 OK (444ms)
   └─ Body: 245 bytes
```

### Testing Connectivity

```bash
# Test if target is reachable
curl "http://localhost:9090/health?porty_host=httpbin.org&porty_port=80"

# Test with invalid target (should get 502 after retries)
curl "http://localhost:9090/test?porty_host=invalid.host&porty_port=9999"
```

## Limitations

1. **HTTP Only:** Query parameter routing requires `mode = "http"`
2. **Parameter Names:** Must use exact names `porty_host` and `porty_port`
3. **Network Access:** Target must be reachable from Porty server
4. **Protocol:** Currently supports HTTP/1.1 (HTTPS via underlying TCP)

## Performance

- **Minimal Overhead:** Query parsing is lightweight
- **Zero-Copy:** Request bodies are streamed without buffering
- **Concurrent:** Each request is handled independently
- **Configurable:** Timeouts and retries prevent hanging connections

Query parameter routing enables powerful, flexible proxying with minimal configuration - perfect for dynamic environments, testing, and service integration!