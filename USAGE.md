# M-Tunnel Usage Guide

This comprehensive guide covers all aspects of using M-Tunnel for secure tunneling, from basic setup to advanced configurations.

## 📚 Table of Contents

- [Quick Start](#-quick-start)
- [Configuration](#-configuration)
- [Tunnel Types](#-tunnel-types)
- [Common Use Cases](#-common-use-cases)
- [Advanced Configuration](#-advanced-configuration)
- [Monitoring & Logging](#-monitoring--logging)
- [Security Best Practices](#-security-best-practices)
- [Troubleshooting](#-troubleshooting)

## 🚀 Quick Start

### Basic Workflow
1. **Install M-Tunnel** (see [INSTALL.md](INSTALL.md))
2. **Create configuration**
3. **Test configuration**
4. **Run M-Tunnel**

### First Configuration
```bash
# Create initial config (auto-generated on first run)
m-tunnel

# Edit the generated config.toml
nano config.toml

# Test configuration
m-tunnel --dry-run

# Start tunneling
m-tunnel
```

## ⚙️ Configuration

### Configuration File Structure
M-Tunnel uses TOML format for configuration. The file consists of three main sections:

```toml
[gate]        # M-Tunnel-Gate connection settings
[limits]      # Connection retry and timeout settings  
[[tunnels]]   # Individual tunnel configurations (array)
```

### Basic Configuration Example
```toml
# M-Tunnel Configuration
[gate]
host = "your-tunnel-gate.com"
user = "your-username"
port = 22
key_path = "./tunnel-key"
timeout = 30
keepalive_interval = 60
server_name = "Production Gateway"

[limits]
max_attempts = 5
retry_window_secs = 300
max_backoff_secs = 60

[[tunnels]]
name = "web-forward"
direction = "send"
local_host = "127.0.0.1"
local_port = 8080
remote_host = "127.0.0.1"
remote_port = 80
enabled = true
```

### Configuration Locations
M-Tunnel searches for configuration in this order:
1. `/etc/m-tunnel/config.toml` (system-wide)
2. `./config.toml` (current directory)
3. Custom path via `--config` option

```bash
# Use custom config file
m-tunnel --config /path/to/custom-config.toml
```

## 🔀 Tunnel Types

### Local Port Forwarding (Send)
Forwards local connections to remote destinations through the M-Tunnel-Gate.

```toml
[[tunnels]]
name = "local-forward"
direction = "send"           # Local → Remote
local_host = "127.0.0.1"    # Local bind address
local_port = 8080           # Local port to listen on
remote_host = "192.168.1.100"  # Target on remote network
remote_port = 80            # Target port
enabled = true
```

**Use case**: Access remote web server locally
- Connect to `localhost:8080` → reaches `192.168.1.100:80` via M-Tunnel-Gate

### Remote Port Forwarding (Receive)
Forwards remote connections to local destinations through the M-Tunnel-Gate.

```toml
[[tunnels]]
name = "remote-forward"
direction = "receive"        # Remote → Local
local_host = "127.0.0.1"    # Local target address
local_port = 3000           # Local target port
remote_host = "0.0.0.0"     # Remote bind address
remote_port = 8080          # Remote port to listen on
enabled = true
```

**Use case**: Expose local service to remote network
- Remote connections to `gate:8080` → reach `localhost:3000`

## 🎯 Common Use Cases

### 1. Web Development
Access local development server from remote location:

```toml
[[tunnels]]
name = "dev-server"
direction = "receive"
local_host = "127.0.0.1"
local_port = 3000           # Your dev server
remote_host = "0.0.0.0"
remote_port = 80            # Public port
enabled = true
```

### 2. Database Access
Secure connection to remote database:

```toml
[[tunnels]]
name = "postgres"
direction = "send"
local_host = "127.0.0.1"
local_port = 5432
remote_host = "db.internal.com"
remote_port = 5432
enabled = true

[[tunnels]]
name = "mysql"
direction = "send"
local_host = "127.0.0.1"
local_port = 3306
remote_host = "mysql.internal.com"
remote_port = 3306
enabled = true
```

### 3. Remote Desktop Access
Access Windows RDP through secure tunnel:

```toml
[[tunnels]]
name = "rdp-server"
direction = "send"
local_host = "127.0.0.1"
local_port = 3389
remote_host = "192.168.1.50"
remote_port = 3389
enabled = true
```

Connect with: `mstsc /v:localhost:3389`

### 4. Web Service Access
Access internal web applications:

```toml
[[tunnels]]
name = "internal-app"
direction = "send"
local_host = "127.0.0.1"
local_port = 8080
remote_host = "app.internal.com"
remote_port = 80
enabled = true

[[tunnels]]
name = "admin-panel"
direction = "send"
local_host = "127.0.0.1"
local_port = 8443
remote_host = "admin.internal.com"
remote_port = 443
enabled = true
```

### 5. Secure Jump Host
Use M-Tunnel-Gate as secure jump host:

```toml
[[tunnels]]
name = "jump-host"
direction = "send"
local_host = "127.0.0.1"
local_port = 2222
remote_host = "target-server.internal.com"
remote_port = 22
enabled = true
```

Connect with: `ssh -p 2222 user@localhost`

### 6. Multiple Service Access
Access multiple internal services:

```toml
# Web services
[[tunnels]]
name = "web-app-1"
direction = "send"
local_host = "127.0.0.1"
local_port = 8001
remote_host = "web1.internal.com"
remote_port = 80
enabled = true

[[tunnels]]
name = "web-app-2"
direction = "send"
local_host = "127.0.0.1"
local_port = 8002
remote_host = "web2.internal.com"
remote_port = 80
enabled = true

# API services
[[tunnels]]
name = "api-service"
direction = "send"
local_host = "127.0.0.1"
local_port = 9001
remote_host = "api.internal.com"
remote_port = 8080
enabled = true
```

## 🔧 Advanced Configuration

### M-Tunnel-Gate Configuration
```toml
[gate]
host = "tunnel-gate.com"
user = "tunnel-user"
port = 2222                 # Custom port
key_path = "~/.ssh/tunnel_key"
timeout = 60                # Connection timeout
keepalive_interval = 30     # Connection keepalive
server_name = "Production Gateway"  # Display name
```

### Connection Limits & Retry Logic
```toml
[limits]
max_attempts = 10           # Maximum retry attempts
retry_window_secs = 600     # Reset retry counter after this time
max_backoff_secs = 120      # Maximum delay between retries
```

### Tunnel-Specific Settings
```toml
[[tunnels]]
name = "critical-service"
direction = "send"
local_host = "0.0.0.0"      # Bind to all interfaces
local_port = 8080
remote_host = "service.internal.com"
remote_port = 80
enabled = true
```

### Environment Variables
```bash
# Log level
export RUST_LOG=debug      # error, warn, info, debug, trace

# Metrics server
export METRICS_PORT=9090   # Enable metrics on port 9090

# Custom config path
export M_TUNNEL_CONFIG=/path/to/config.toml
```

## 📊 Monitoring & Logging

### Log Levels
```bash
# Basic info (default)
m-tunnel

# Detailed debugging
RUST_LOG=debug m-tunnel

# Very verbose
RUST_LOG=trace m-tunnel

# Warnings and errors only
RUST_LOG=warn m-tunnel
```

### Log Format
M-Tunnel outputs structured logs with timestamps:
```
[2025-09-23T10:30:15Z INFO M-Tunnel] Starting M-Tunnel v1.0.0
[2025-09-23T10:30:15Z INFO M-Tunnel] Loaded configuration with 3 tunnels
[2025-09-23T10:30:15Z INFO M-Tunnel] M-Tunnel-Gate : user@Production Gateway
```

### Metrics Server
Enable built-in metrics server for monitoring:
```bash
METRICS_PORT=9090 m-tunnel
```

Access metrics at `http://localhost:9090/metrics`

Available metrics:
- `tunnel_connections_total`: Total connections per tunnel
- `tunnel_active_connections`: Currently active connections
- `tunnel_status`: Tunnel status (0=down, 1=up)
- `connection_duration_seconds`: Connection duration histogram

### Log Security Features
- **IP Masking**: Internal IPs (192.168.x.x, 10.x.x.x, 172.16-31.x.x) show as server names
- **Command Hiding**: Connection commands are never logged
- **Credential Protection**: No sensitive information in logs

## 🛡️ Security Best Practices

### Authentication Key Security
```bash
# Generate secure keys
ssh-keygen -t ed25519 -f ~/.ssh/m-tunnel-key

# Set proper permissions
chmod 600 ~/.ssh/m-tunnel-key
chmod 644 ~/.ssh/m-tunnel-key.pub

# Use key-specific config
Host tunnel-gate
    HostName your-tunnel-gate.com
    User tunnel-user
    IdentityFile ~/.ssh/m-tunnel-key
    IdentitiesOnly yes
```

### Configuration Security
```toml
[gate]
# Use specific user for tunneling
user = "tunnel-user"        # Not root or admin user

# Use non-standard SSH port if possible
port = 2222

# Restrict server access
server_name = "TunnelGW"    # Hide real hostname in logs
```

### Network Security
- Use dedicated user with minimal privileges on M-Tunnel-Gate
- Restrict access by IP when possible
- Use certificate authentication for enhanced security
- Monitor tunnel connections and usage

## 🔍 Troubleshooting

### Common Issues

#### "Connection failed"
```bash
# Test M-Tunnel-Gate connectivity manually
ssh -i /path/to/key user@tunnel-gate.com "echo test"

# Check connectivity with verbose output
ssh -v -i /path/to/key user@tunnel-gate.com
```

#### "Permission denied"
```bash
# Check key permissions
ls -la ~/.ssh/tunnel-key

# Fix permissions
chmod 600 ~/.ssh/tunnel-key

# Verify key is on M-Tunnel-Gate
ssh-copy-id -i ~/.ssh/tunnel-key.pub user@tunnel-gate.com
```

#### "Connection refused"
```bash
# Check if service is running on target
nmap -p 80 target-host

# Test local tunnel binding
netstat -tulpn | grep :8080

# Check firewall rules
sudo ufw status
```

#### "Tunnel keeps disconnecting"
```toml
# Increase timeouts
[gate]
timeout = 120
keepalive_interval = 30

[limits]
max_attempts = 10
max_backoff_secs = 300
```

### Debug Commands
```bash
# Validate configuration
m-tunnel --dry-run

# Enable debug logging
RUST_LOG=debug m-tunnel

# Test M-Tunnel-Gate connectivity manually
ssh -vvv -i key user@tunnel-gate

# Check network connectivity
ping target-host
telnet target-host port

# Monitor connections
ss -tulpn | grep m-tunnel
netstat -an | grep LISTEN
```

### Log Analysis
```bash
# Follow logs in real-time
m-tunnel 2>&1 | tee tunnel.log

# Filter specific tunnel
m-tunnel 2>&1 | grep "tunnel-name"

# Check error patterns
m-tunnel 2>&1 | grep -i error

# Monitor connection status
m-tunnel 2>&1 | grep -E "(Connected|Disconnected|Failed)"
```

## 📖 Configuration Examples

### Multi-Environment Setup
```toml
# Development environment
[gate]
host = "dev-tunnel-gate.com"
user = "dev-user"
key_path = "./dev-key"
server_name = "Dev Gateway"

[[tunnels]]
name = "dev-db"
direction = "send"
local_host = "127.0.0.1"
local_port = 5432
remote_host = "dev-db.internal"
remote_port = 5432
enabled = true

[[tunnels]]
name = "dev-api"
direction = "send"
local_host = "127.0.0.1"
local_port = 8080
remote_host = "dev-api.internal"
remote_port = 8080
enabled = true
```

### Production Setup
```toml
# Production environment with enhanced security
[gate]
host = "prod-tunnel-gate.com"
user = "tunnel-svc"
port = 2222
key_path = "/etc/m-tunnel/prod-key"
timeout = 60
keepalive_interval = 30
server_name = "Production Gateway"

[limits]
max_attempts = 5
retry_window_secs = 300
max_backoff_secs = 60

# Database access
[[tunnels]]
name = "prod-postgres"
direction = "send"
local_host = "127.0.0.1"
local_port = 5432
remote_host = "db-cluster.internal"
remote_port = 5432
enabled = true

# Monitoring access
[[tunnels]]
name = "grafana"
direction = "send"
local_host = "127.0.0.1"
local_port = 3000
remote_host = "monitoring.internal"
remote_port = 3000
enabled = true
```

### High Availability Setup
```toml
# Primary M-Tunnel-Gate
[gate]
host = "primary-tunnel-gate.com"
user = "tunnel-user"
key_path = "./ha-key"
server_name = "Primary Gateway"

# Multiple redundant tunnels
[[tunnels]]
name = "service-primary"
direction = "send"
local_host = "127.0.0.1"
local_port = 8080
remote_host = "service-1.internal"
remote_port = 80
enabled = true

[[tunnels]]
name = "service-backup"
direction = "send"
local_host = "127.0.0.1"
local_port = 8081
remote_host = "service-2.internal"
remote_port = 80
enabled = true
```

---

For more examples and advanced configurations, see the [README](README.md) or check the project's.