# M-Tunnel

![Version](https://img.shields.io/badge/version-1.0.0-blue.svg)
![License](https://img.shields.io/badge/license-MIT-green.svg)
![Platform](https://img.shields.io/badge/platform-Windows%20%7C%20Linux%20%7C%20macOS-lightgrey.svg)

A fast, secure, and reliable tunneling utility written in Rust. M-Tunnel provides seamless port forwarding and reverse tunneling capabilities between your local machine and M-Tunnel-Gate servers for optimal performance and compatibility.

## ✨ Features

- **🚀 High Performance**: Native implementation for maximum compatibility and performance
- **🔒 Security First**: IP masking for internal networks, secure key handling
- **⚡ Async Architecture**: Built with Tokio for concurrent tunnel management
- **📊 Built-in Metrics**: Optional metrics server for monitoring tunnel health
- **🎨 Colored Logging**: Clear, professional log output with color-coded levels
- **🔧 TOML Configuration**: Modern, structured configuration format
- **🔄 Auto-Reconnection**: Intelligent retry logic with exponential backoff
- **💻 Cross-Platform**: Works on Windows, Linux, and macOS

## 🚀 Quick Start

### Prerequisites

M-Tunnel requires an SSH client to establish secure connections to M-Tunnel-Gate servers.

**Windows:**
```powershell
# Windows 10/11 includes SSH client by default
# For older versions, install OpenSSH Client:
Add-WindowsCapability -Online -Name OpenSSH.Client~~~~0.0.1.0

# Alternative: Install Git for Windows (includes SSH)
# Download from: https://git-scm.com/download/win
```

**Linux:**
```bash
# Ubuntu/Debian
sudo apt update && sudo apt install openssh-client

# CentOS/RHEL/Fedora
sudo yum install openssh-clients
# or
sudo dnf install openssh-clients
```

**macOS:**
```bash
# SSH client included by default
# If needed, install via Homebrew:
brew install openssh
```

### Installation

1. **Download the latest release** from the [releases page](../../releases)
2. **Extract the binary** to a directory in your PATH
3. **Create configuration file** (will be auto-generated on first run)

### Basic Usage

```bash
# First run - creates sample config.toml
m-tunnel

# Edit config.toml with your M-Tunnel-Gate settings
# Then run again
m-tunnel
```

## 📁 Configuration

M-Tunnel uses a TOML configuration file (`config.toml`) for all settings:

```toml
# M-Tunnel Configuration
[gate]
host = "your-tunnel-gate.com"
user = "your-username"
port = 22
key_path = "./tunnel-key"
timeout = 30
keepalive_interval = 60
server_name = "MyGate"  # Display name for logs

[limits]
max_attempts = 5
retry_window_secs = 300
max_backoff_secs = 60

# Port forwarding examples
[[tunnels]]
name = "web-server"
direction = "send"        # Local → Remote
local_host = "127.0.0.1"
local_port = 8080
remote_host = "127.0.0.1"
remote_port = 80
enabled = true

[[tunnels]]
name = "reverse-tunnel"
direction = "receive"     # Remote → Local
local_host = "127.0.0.1"
local_port = 22
remote_host = "0.0.0.0"
remote_port = 2222
enabled = true
```

## 🔧 Configuration Reference

### [gate] Section
| Field | Type | Description |
|-------|------|-------------|
| `host` | string | M-Tunnel-Gate server hostname or IP |
| `user` | string | Username for M-Tunnel-Gate server |
| `port` | number | M-Tunnel-Gate server port (default: 22) |
| `key_path` | string | Path to authentication key |
| `timeout` | number | Connection timeout in seconds |
| `keepalive_interval` | number | Connection keepalive interval |
| `server_name` | string | Display name for logs (optional) |

### [limits] Section
| Field | Type | Description |
|-------|------|-------------|
| `max_attempts` | number | Maximum retry attempts |
| `retry_window_secs` | number | Retry window in seconds |
| `max_backoff_secs` | number | Maximum backoff delay |

### [[tunnels]] Section
| Field | Type | Description |
|-------|------|-------------|
| `name` | string | Tunnel identifier |
| `direction` | string | "send" (local→remote) or "receive" (remote→local) |
| `local_host` | string | Local bind address |
| `local_port` | number | Local port |
| `remote_host` | string | Remote target address |
| `remote_port` | number | Remote target port |
| `enabled` | boolean | Enable/disable tunnel |

## 🎯 Usage Examples

### Web Development
Forward local development server to remote:
```toml
[[tunnels]]
name = "dev-server"
direction = "send"
local_host = "127.0.0.1"
local_port = 3000
remote_host = "127.0.0.1"
remote_port = 80
enabled = true
```

### Remote Desktop Access
Access internal RDP server:
```toml
[[tunnels]]
name = "remote-desktop"
direction = "receive"
local_host = "127.0.0.1"
local_port = 3389
remote_host = "192.168.1.100"
remote_port = 3389
enabled = true
```

### Database Connection
Secure database access:
```toml
[[tunnels]]
name = "database"
direction = "receive"
local_host = "127.0.0.1"
local_port = 5432
remote_host = "db.internal"
remote_port = 5432
enabled = true
```

## 🖥️ Command Line Options

```bash
m-tunnel [OPTIONS]

OPTIONS:
    --dry-run           Validate configuration without creating tunnels
    --config <FILE>     Use specific configuration file
    -h, --help          Print help information

ENVIRONMENT VARIABLES:
    RUST_LOG=level      Set log level (error, warn, info, debug, trace)
    METRICS_PORT=port   Enable metrics server on specified port
```

## 📊 Monitoring & Metrics

Enable the built-in metrics server:

```bash
METRICS_PORT=9090 m-tunnel
```

Access metrics at `http://localhost:9090/metrics`

Available metrics:
- Tunnel connection status
- Active connections count
- Retry attempts
- Connection duration

## 🔍 Logging

M-Tunnel provides detailed logging with colored output:

- **🔴 ERROR**: Critical errors
- **🟡 WARN**: Warnings and recoverable issues  
- **🟢 INFO**: General information (default level)
- **🔵 DEBUG**: Detailed debugging information
- **⚪ TRACE**: Very verbose tracing

Set log level with `RUST_LOG` environment variable:
```bash
RUST_LOG=debug m-tunnel
```

## 🛡️ Security Features

- **IP Masking**: Internal network IPs (192.168.x.x, 10.x.x.x, 172.16-31.x.x) are masked in logs
- **Server Names**: Use friendly names instead of exposing internal infrastructure
- **Secure Logging**: Connection commands and sensitive information are never logged

## 🔧 Troubleshooting

### Common Issues

**"Connection failed"**
- Verify M-Tunnel-Gate server is accessible
- Check authentication key permissions (should be 600)
- Ensure M-Tunnel-Gate server allows key-based authentication

**"Permission denied"**
- Verify authentication key is correct
- Check M-Tunnel-Gate server configuration
- Ensure user has necessary permissions

**"Connection refused"**
- Check if target service is running
- Verify firewall settings
- Confirm port numbers are correct

### Debug Mode
Enable detailed logging:
```bash
RUST_LOG=debug m-tunnel
```

### Dry Run
Test configuration without creating tunnels:
```bash
m-tunnel --dry-run
```

## 🏗️ Building from Source

### Prerequisites
- Rust 1.70+ 
- Cargo

### Build Steps
```bash
git clone https://github.com/mufkuw/m-tunnel-rust.git
cd m-tunnel-rust
cargo build --release
```

The binary will be available at `target/release/m-tunnel`

## 🤝 Contributing

1. Fork the repository
2. Create a feature branch
3. Make your changes
4. Add tests if applicable
5. Submit a pull request

## 📝 License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## 👨‍💻 Author

**Muffaddal Kalla** - [mufkuw@gmail.com](mailto:mufkuw@gmail.com)

## 🙏 Acknowledgments

- Built with [Rust](https://www.rust-lang.org/) and [Tokio](https://tokio.rs/)
- Uses secure tunnel protocols for maximum compatibility
- Inspired by the need for simple, reliable tunneling solutions

---

**M-Tunnel** - Secure, Fast, Reliable Tunneling