# M-Tunnel Installation Guide

This guide provides detailed installation instructions for M-Tunnel on different platforms.

## 📋 Prerequisites

### System Requirements
- **Operating System**: Windows 10+, Linux (Ubuntu 18.04+, CentOS 7+), macOS 10.15+
- **SSH Client**: Must be available in system PATH
- **Memory**: Minimum 10MB RAM per tunnel
- **Network**: Outbound SSH access (port 22 or custom)

### SSH Client Installation

#### Windows
SSH client is included in Windows 10 (1809+) and Windows 11 by default.

For older Windows versions:
```powershell
# Install OpenSSH Client (Windows 10)
Add-WindowsCapability -Online -Name OpenSSH.Client~~~~0.0.1.0

# Or install Git for Windows (includes SSH)
# Download from: https://git-scm.com/download/win
```

#### Linux
```bash
# Ubuntu/Debian
sudo apt update && sudo apt install openssh-client

# CentOS/RHEL/Fedora
sudo yum install openssh-clients
# or
sudo dnf install openssh-clients

# Arch Linux
sudo pacman -S openssh
```

#### macOS
SSH client is included by default. If needed:
```bash
# Using Homebrew
brew install openssh
```

## 🚀 Installation Methods

### Method 1: Download Binary (Recommended)

#### Windows
1. Download the latest `m-tunnel-windows.exe` from [releases](../../releases)
2. Rename to `m-tunnel.exe`
3. Move to a directory in your PATH (e.g., `C:\Windows\System32` or create `C:\bin` and add to PATH)
4. Open Command Prompt or PowerShell and verify:
   ```cmd
   m-tunnel --help
   ```

#### Linux
```bash
# Download latest release
curl -L https://github.com/mufkuw/m-tunnel-rust/releases/latest/download/m-tunnel-linux -o m-tunnel

# Make executable
chmod +x m-tunnel

# Move to system path
sudo mv m-tunnel /usr/local/bin/

# Verify installation
m-tunnel --help
```

#### macOS
```bash
# Download latest release
curl -L https://github.com/mufkuw/m-tunnel-rust/releases/latest/download/m-tunnel-macos -o m-tunnel

# Make executable
chmod +x m-tunnel

# Move to system path
sudo mv m-tunnel /usr/local/bin/

# Verify installation
m-tunnel --help
```

### Method 2: Build from Source

#### Prerequisites for Building
- Rust 1.70 or later
- Cargo (included with Rust)
- Git

#### Install Rust
```bash
# All platforms
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source ~/.cargo/env

# Windows alternative: Download from https://rustup.rs/
```

#### Build M-Tunnel
```bash
# Clone repository
git clone https://github.com/mufkuw/m-tunnel-rust.git
cd m-tunnel-rust

# Build release version
cargo build --release

# The binary will be at: target/release/m-tunnel
# Copy to system PATH or use directly
```

### Method 3: Package Managers

#### Cargo (Rust Package Manager)
```bash
cargo install m-tunnel
```

#### Homebrew (macOS/Linux)
```bash
# Coming soon
brew install m-tunnel
```

#### Chocolatey (Windows)
```powershell
# Coming soon
choco install m-tunnel
```

## 🔧 Post-Installation Setup

### 1. Create SSH Key (if needed)
```bash
# Generate new SSH key
ssh-keygen -t rsa -b 4096 -f ~/.ssh/m-tunnel-key

# Copy public key to server
ssh-copy-id -i ~/.ssh/m-tunnel-key.pub user@your-server.com
```

### 2. First Run
```bash
# Create initial configuration
m-tunnel
```

This will create a sample `config.toml` file in the current directory.

### 3. Configure M-Tunnel
Edit the generated `config.toml`:
```toml
[gate]
host = "your-server.com"
user = "your-username"
key_path = "/path/to/your/ssh-key"
# ... other settings
```

### 4. Test Configuration
```bash
# Validate configuration
m-tunnel --dry-run

# Start M-Tunnel
m-tunnel
```

## 📂 Configuration File Locations

M-Tunnel looks for configuration files in this order:

1. `/etc/m-tunnel/config.toml` (system-wide)
2. `./config.toml` (current directory)
3. Custom path via `--config` option

### System Installation
For system-wide installation, create the configuration directory:

#### Linux/macOS
```bash
sudo mkdir -p /etc/m-tunnel
sudo cp config.toml /etc/m-tunnel/
sudo chown root:root /etc/m-tunnel/config.toml
sudo chmod 644 /etc/m-tunnel/config.toml
```

#### Windows
```powershell
# Create directory (as Administrator)
New-Item -ItemType Directory -Path "C:\ProgramData\m-tunnel" -Force
Copy-Item "config.toml" "C:\ProgramData\m-tunnel\"
```

## 🔐 SSH Key Setup

### Key Generation
```bash
# Ed25519 (recommended)
ssh-keygen -t ed25519 -f ~/.ssh/m-tunnel-ed25519

# RSA (fallback)
ssh-keygen -t rsa -b 4096 -f ~/.ssh/m-tunnel-rsa
```

### Key Permissions
Ensure proper SSH key permissions:

#### Linux/macOS
```bash
chmod 600 ~/.ssh/m-tunnel-key
chmod 644 ~/.ssh/m-tunnel-key.pub
```

#### Windows
```powershell
# Using icacls
icacls "path\to\ssh-key" /inheritance:r /grant:r "%USERNAME%:R"
```

### Server Setup
Add your public key to the server:
```bash
# Method 1: ssh-copy-id
ssh-copy-id -i ~/.ssh/m-tunnel-key.pub user@server.com

# Method 2: Manual
cat ~/.ssh/m-tunnel-key.pub | ssh user@server.com "mkdir -p ~/.ssh && cat >> ~/.ssh/authorized_keys"
```

## 🚀 Running as a Service

### Linux (systemd)
Create service file:
```bash
sudo nano /etc/systemd/system/m-tunnel.service
```

Content:
```ini
[Unit]
Description=M-Tunnel SSH Tunneling Service
After=network.target

[Service]
Type=simple
User=tunnel-user
WorkingDirectory=/etc/m-tunnel
ExecStart=/usr/local/bin/m-tunnel
Restart=always
RestartSec=5

[Install]
WantedBy=multi-user.target
```

Enable and start:
```bash
sudo systemctl daemon-reload
sudo systemctl enable m-tunnel
sudo systemctl start m-tunnel
```

### Windows (NSSM)
1. Download NSSM from https://nssm.cc/
2. Install as service:
```cmd
nssm install M-Tunnel "C:\path\to\m-tunnel.exe"
nssm set M-Tunnel AppDirectory "C:\path\to\config"
nssm start M-Tunnel
```

### macOS (launchd)
Create plist file:
```bash
sudo nano /Library/LaunchDaemons/com.m-tunnel.plist
```

Content:
```xml
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <key>Label</key>
    <string>com.m-tunnel</string>
    <key>ProgramArguments</key>
    <array>
        <string>/usr/local/bin/m-tunnel</string>
    </array>
    <key>WorkingDirectory</key>
    <string>/etc/m-tunnel</string>
    <key>KeepAlive</key>
    <true/>
</dict>
</plist>
```

Load service:
```bash
sudo launchctl load /Library/LaunchDaemons/com.m-tunnel.plist
```

## 🔍 Verification

### Test Installation
```bash
# Check version
m-tunnel --help

# Test SSH connectivity
ssh -i /path/to/key user@server.com "echo 'SSH works'"

# Validate configuration
m-tunnel --dry-run

# Start with debug logging
RUST_LOG=debug m-tunnel
```

### Check Service Status
```bash
# Linux
sudo systemctl status m-tunnel

# Windows
sc query M-Tunnel

# macOS
sudo launchctl list | grep m-tunnel
```

## 🆘 Troubleshooting

### Common Issues

**"m-tunnel: command not found"**
- Add binary location to PATH
- Use full path to binary

**"SSH command failed"**
- Verify SSH client installation: `ssh -V`
- Test SSH connectivity manually
- Check SSH key permissions

**"Permission denied"**
- Verify SSH key is correct
- Check server SSH configuration
- Ensure key is added to server's authorized_keys

**Service won't start**
- Check configuration file permissions
- Verify user has access to SSH keys
- Review service logs

### Get Help
- Check logs: `RUST_LOG=debug m-tunnel`
- Test SSH manually: `ssh -i key user@host`
- Validate config: `m-tunnel --dry-run`

## 📚 Next Steps

After installation:
1. Read the [Usage Guide](USAGE.md) for detailed configuration
2. Check the [README](README.md) for examples
3. Configure monitoring with metrics if needed
4. Set up as a system service for production use

---

For more help, see the [main documentation](README.md) or create an issue on GitHub.