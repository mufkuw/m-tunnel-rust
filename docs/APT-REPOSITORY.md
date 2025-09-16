# M-Tunnel APT Repository

## 🚀 Quick Installation

### Step 1: Add Repository Key
```bash
# Download and add the GPG key
curl -fsSL https://your-domain.com/m-tunnel-apt-repo/public.key | sudo gpg --dearmor -o /usr/share/keyrings/m-tunnel.gpg

# Alternative: Add key from GitHub Pages
curl -fsSL https://yourusername.github.io/m-tunnel-rust/public.key | sudo gpg --dearmor -o /usr/share/keyrings/m-tunnel.gpg
```

### Step 2: Add Repository Source
```bash
# For Ubuntu/Debian systems
echo "deb [signed-by=/usr/share/keyrings/m-tunnel.gpg] https://your-domain.com/m-tunnel-apt-repo $(lsb_release -cs) main" | sudo tee /etc/apt/sources.list.d/m-tunnel.list

# For GitHub Pages hosting
echo "deb [signed-by=/usr/share/keyrings/m-tunnel.gpg] https://yourusername.github.io/m-tunnel-rust $(lsb_release -cs) main" | sudo tee /etc/apt/sources.list.d/m-tunnel.list
```

### Step 3: Install Package
```bash
# Update package list
sudo apt update

# Install m-tunnel-rust
sudo apt install m-tunnel-rust
```

## 🔧 Configuration

### Initial Setup
```bash
# 1. Configure SSH settings
sudo nano /etc/m-tunnel/.env

# 2. Define your tunnels
sudo nano /etc/m-tunnel/m-tunnel.conf

# 3. Add SSH host key for security
ssh-keyscan -H your-ssh-server.com | sudo tee -a /etc/m-tunnel/known_hosts

# 4. Start the service
sudo systemctl start m-tunnel
sudo systemctl enable m-tunnel
```

### Example Configuration

**`/etc/m-tunnel/.env`:**
```bash
SSH_HOST=your-server.com
SSH_PORT=22
SSH_USER=your-username
SSH_PRIVATE_KEY=m-tunnel.key
RUST_LOG=info
```

**`/etc/m-tunnel/m-tunnel.conf`:**
```
# Forward tunnels (local → remote)
receive -- 127.0.0.1:8080 from 10.0.0.5:80
receive -- 127.0.0.1:3306 from database-server:3306

# Reverse tunnels (remote → local)
send -- 0.0.0.0:2222 to 127.0.0.1:22
send -- 0.0.0.0:8000 to 127.0.0.1:8000
```

## 📋 Management Commands

```bash
# Check service status
sudo systemctl status m-tunnel

# View logs
sudo journalctl -u m-tunnel -f

# Restart service (after config changes)
sudo systemctl restart m-tunnel

# Test configuration
sudo -u m-tunnel m-tunnel-rust --check-config
```

## 🏗️ Supported Platforms

| Distribution | Codename | Architectures |
|--------------|----------|---------------|
| Ubuntu 22.04 | jammy    | amd64, arm64, armhf |
| Ubuntu 20.04 | focal    | amd64, arm64, armhf |
| Debian 12    | bookworm | amd64, arm64, armhf |
| Debian 11    | bullseye | amd64, arm64, armhf |

## 🔒 Security Features

- ✅ **Dedicated User**: Runs as `m-tunnel` user (not root)
- ✅ **SSH Security**: Strict host key verification
- ✅ **Rate Limiting**: Protection against brute force
- ✅ **Systemd Hardening**: Full privilege separation
- ✅ **Secure Permissions**: Proper file access controls

## 🐛 Troubleshooting

### Common Issues

**1. Repository not found**
```bash
# Check if repository is properly added
cat /etc/apt/sources.list.d/m-tunnel.list

# Verify GPG key
sudo apt-key list | grep -i m-tunnel
```

**2. SSH connection fails**
```bash
# Check SSH host key
sudo cat /etc/m-tunnel/known_hosts

# Test SSH connection manually
sudo -u m-tunnel ssh -i /etc/m-tunnel/m-tunnel.key user@host
```

**3. Permission denied**
```bash
# Check file permissions
ls -la /etc/m-tunnel/

# Reset permissions
sudo chown -R m-tunnel:m-tunnel /etc/m-tunnel/
sudo chmod 600 /etc/m-tunnel/.env
sudo chmod 600 /etc/m-tunnel/m-tunnel.key
```

### Log Analysis
```bash
# View detailed logs
sudo journalctl -u m-tunnel --since "1 hour ago"

# Check for errors
sudo journalctl -u m-tunnel -p err

# Follow logs in real-time
sudo journalctl -u m-tunnel -f
```

## 🔄 Updates

The package will automatically update when you run:
```bash
sudo apt update && sudo apt upgrade
```

To get the latest version immediately:
```bash
sudo apt update
sudo apt install --only-upgrade m-tunnel-rust
```

## 📞 Support

- **GitHub Issues**: [Report bugs and request features](https://github.com/mufkuw/m-tunnel-rust/issues)
- **Documentation**: [Full documentation](https://github.com/mufkuw/m-tunnel-rust)
- **Security Issues**: Contact maintainer directly

## 🗑️ Uninstallation

```bash
# Stop and disable service
sudo systemctl stop m-tunnel
sudo systemctl disable m-tunnel

# Remove package
sudo apt remove m-tunnel-rust

# Complete removal (including config files)
sudo apt purge m-tunnel-rust

# Remove repository (optional)
sudo rm /etc/apt/sources.list.d/m-tunnel.list
sudo rm /usr/share/keyrings/m-tunnel.gpg
```

---

## 📦 Repository Statistics

- **Total Packages**: Auto-updated
- **Last Updated**: Auto-updated via CI/CD
- **GPG Signed**: ✅ All packages cryptographically signed
- **Multi-Architecture**: ✅ amd64, arm64, armhf support