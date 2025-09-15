# Installation Guide

## Quick Install

```bash
# Build and create .deb package
./installer.sh

# Install the package
sudo dpkg -i m-tunnel-rust_1.0.0-1.deb

# Add SSH host key for security
ssh-keyscan -H your-ssh-server.com | sudo tee -a /etc/m-tunnel/known_hosts

# Configure your tunnels
sudo nano /etc/m-tunnel/.env
sudo nano /etc/m-tunnel/m-tunnel.conf

# Start the service
sudo systemctl start m-tunnel.service
```

## Security & Performance Features ⚡🛡️

### 🔒 **Security Enhancements**
- **✅ Dedicated Service User**: Runs as `m-tunnel` user (not root)
- **✅ SSH Host Verification**: Strict host key checking with known_hosts
- **✅ Connection Rate Limiting**: Protection against brute force attacks  
- **✅ Enhanced Systemd Security**: Full privilege separation and sandboxing
- **✅ Secure File Permissions**: Proper ownership and access controls
- **✅ SSH Multiplexing**: Reuses existing connections for efficiency

### ⚡ **Performance Optimizations**
- **✅ Memory Efficient**: Arc-based shared configuration (no string cloning)
- **✅ Connection Pooling**: SSH control master for connection reuse
- **✅ Smart Reconnection**: Exponential backoff with connection metrics
- **✅ Reduced Logging**: Error-focused logging to minimize overhead
- **✅ TCP Optimizations**: Keep-alive and compression enabled

## File Organization

The installation follows Linux FHS (Filesystem Hierarchy Standard):

```
/etc/m-tunnel/               # Configuration directory (750 m-tunnel:m-tunnel)
├── .env                    # Environment variables - SSH_HOST, SSH_USER, etc. (600)
├── m-tunnel.conf           # Tunnel definitions (644)
├── m-tunnel.key           # SSH private key (600)
└── known_hosts            # SSH host keys for verification (644)

/usr/bin/                   # System binaries
└── m-tunnel-rust          # Main executable

/lib/systemd/system/        # Systemd services
└── m-tunnel.service       # Service definition (runs as m-tunnel user)

/var/log/m-tunnel/         # Log files (750 m-tunnel:m-tunnel)
└── (application logs)

/tmp/ssh-m-tunnel/         # SSH control sockets (700 m-tunnel:m-tunnel)
└── (SSH connection multiplexing)
```

## Security Setup

### 🔑 **SSH Host Key Verification**
```bash
# IMPORTANT: Add your SSH server's host key for security
ssh-keyscan -H your-ssh-server.com | sudo tee -a /etc/m-tunnel/known_hosts

# For multiple hosts:
ssh-keyscan -H server1.com server2.com | sudo tee -a /etc/m-tunnel/known_hosts

# Verify host keys are added:
sudo cat /etc/m-tunnel/known_hosts
```

### 👤 **Service User Security**
The service now runs as a dedicated `m-tunnel` user with minimal privileges:
- No shell access (`/bin/false`)
- Home directory: `/etc/m-tunnel`
- Owns only necessary files and directories
- Cannot escalate privileges

### 🔐 **File Permissions**
```bash
/etc/m-tunnel/           # 750 (m-tunnel:m-tunnel)
├── .env                # 600 (sensitive SSH credentials)
├── m-tunnel.conf       # 644 (tunnel configuration)
├── m-tunnel.key       # 600 (SSH private key)
└── known_hosts        # 644 (SSH host keys)
```

## Configuration

### Environment Variables (`/etc/m-tunnel/.env`)
```bash
SSH_HOST=your-remote-server.com
SSH_PORT=22
SSH_USER=your-username
SSH_PRIVATE_KEY=m-tunnel.key
RUST_LOG=info
```

### Tunnel Configuration (`/etc/m-tunnel/m-tunnel.conf`)
```
# Forward tunnels (local → remote)
receive -- 127.0.0.1:8080 from 10.0.0.5:80

# Reverse tunnels (remote → local)  
send -- 0.0.0.0:2222 to 127.0.0.1:22
```

## Service Management

```bash
# Check status
sudo systemctl status m-tunnel

# Start/stop service
sudo systemctl start m-tunnel
sudo systemctl stop m-tunnel

# Enable/disable auto-start
sudo systemctl enable m-tunnel
sudo systemctl disable m-tunnel

# View logs
sudo journalctl -u m-tunnel -f
```

## Development Mode

For development, the application will look for config files in the current directory:
- `./.env`
- `./m-tunnel.conf`

This allows development and testing without system installation.

## Security

- SSH private key permissions: `600` (owner read/write only)
- Environment file permissions: `600` (contains sensitive data)
- Config file permissions: `644` (readable by system)
- Service runs with systemd security restrictions