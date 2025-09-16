#!/bin/bash

echo "🔗 SSH2 Real Server Testing Guide"
echo "================================="

echo ""
echo "This script helps you test the SSH2 implementation with a real SSH server."
echo ""

# Colors for output
GREEN='\033[0;32m'
RED='\033[0;31m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

echo -e "${BLUE}Option 1: Test with localhost SSH server${NC}"
echo "-------------------------------------------"
echo ""
echo "1. Install SSH server on your system:"
echo "   sudo apt-get install openssh-server    # Ubuntu/Debian"
echo "   sudo yum install openssh-server        # CentOS/RHEL"
echo ""
echo "2. Start SSH service:"
echo "   sudo systemctl start ssh               # Ubuntu/Debian"
echo "   sudo systemctl start sshd              # CentOS/RHEL"
echo ""
echo "3. Test SSH connection:"
echo "   ssh localhost"
echo ""
echo "4. Create test configuration:"

cat > real_ssh_test.toml << 'EOF'
[ssh]
host = "localhost"
user = "$USER"
port = 22
key_path = "$HOME/.ssh/id_rsa"
timeout = 30
keepalive_interval = 60

[limits]
max_attempts = 3
retry_window_secs = 300

[[tunnels]]
name = "test-tunnel"
direction = "forward"
local_host = "127.0.0.1"
local_port = 8080
remote_host = "127.0.0.1"
remote_port = 80
enabled = true
EOF

# Replace variables with actual values
sed -i "s/\$USER/$USER/g" real_ssh_test.toml
sed -i "s|\$HOME|$HOME|g" real_ssh_test.toml

echo ""
echo "✅ Created real_ssh_test.toml with your settings:"
echo "   Host: localhost"
echo "   User: $USER"
echo "   Key: $HOME/.ssh/id_rsa"
echo ""

echo -e "${BLUE}Option 2: Test with remote SSH server${NC}"
echo "--------------------------------------------"
echo ""
echo "1. Edit real_ssh_test.toml manually:"
echo "   [ssh]"
echo "   host = \"your-server.com\""
echo "   user = \"your-username\""
echo "   key_path = \"path/to/your/private/key\""
echo ""

echo -e "${BLUE}Testing Commands${NC}"
echo "----------------"
echo ""
echo "1. Test SSH connection manually first:"
echo "   ssh -i \$HOME/.ssh/id_rsa $USER@localhost"
echo ""
echo "2. Test with dry-run mode:"
echo "   cargo run -- --ssh2 --config real_ssh_test.toml --dry-run"
echo ""
echo "3. Test actual SSH2 connection:"
echo "   cargo run -- --ssh2 --config real_ssh_test.toml"
echo ""

echo -e "${YELLOW}⚠️  Important Security Notes:${NC}"
echo "• Make sure your SSH key has correct permissions (600)"
echo "• Test with a non-production server first"
echo "• Monitor logs for any connection issues"
echo "• Use port forwarding to test tunnel functionality"
echo ""

echo -e "${BLUE}Performance Benchmarking${NC}"
echo "------------------------"
echo ""
echo "Compare SSH2 vs CLI performance:"
echo ""
echo "1. CLI version timing:"
echo "   time cargo run -- --config real_ssh_test.toml"
echo ""
echo "2. SSH2 version timing:"
echo "   time cargo run -- --ssh2 --config real_ssh_test.toml"
echo ""

echo -e "${BLUE}Troubleshooting${NC}"
echo "---------------"
echo ""
echo "If you encounter issues:"
echo ""
echo "1. Check SSH key permissions:"
echo "   ls -la $HOME/.ssh/id_rsa"
echo "   # Should show: -rw------- (600)"
echo ""
echo "2. Test SSH connection manually:"
echo "   ssh -v $USER@localhost"
echo "   # Use -v for verbose debugging"
echo ""
echo "3. Check SSH server status:"
echo "   sudo systemctl status ssh"
echo ""
echo "4. View SSH server logs:"
echo "   sudo journalctl -u ssh -f"
echo ""
echo "5. Test port availability:"
echo "   netstat -tlnp | grep :22"
echo ""

echo -e "${GREEN}Ready to test!${NC}"
echo ""
echo "🚀 Run this command to start testing:"
echo -e "${GREEN}cargo run -- --ssh2 --config real_ssh_test.toml --dry-run${NC}"
echo ""
echo "📊 For performance comparison:"
echo -e "${GREEN}time cargo run -- --ssh2 --config real_ssh_test.toml${NC}"
echo ""
echo "🎯 Configuration file created: real_ssh_test.toml"