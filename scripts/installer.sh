#!/bin/bash
set -e

echo "🚀 M-Tunnel Rust Installer v2.0 (SSH2 Enhanced)"
echo "================================================="
echo ""

# Step 1: Build the Rust project
echo "📦 Building the Rust project..."
cargo build --release

# Step 2: Extract metadata from Cargo.toml
echo "📋 Extracting metadata from Cargo.toml..."
NAME=$(grep -m1 '^name' Cargo.toml | sed 's/name *= *"\(.*\)"/\1/')
VERSION=$(grep -m1 '^version' Cargo.toml | sed 's/version *= *"\(.*\)"/\1/')
AUTHOR=$(grep -m1 '^authors' Cargo.toml | sed -E 's/authors *= *\["([^"]*)".*/\1/')
DESCRIPTION=$(grep -m1 '^description' Cargo.toml | sed 's/description *= *"\(.*\)"/\1/')

DESCRIPTION=${DESCRIPTION:-"SSH tunnel management utility with native SSH2 library support"}

DEB_DIR="${NAME}_${VERSION}-1"

echo "Package: $NAME"
echo "Version: $VERSION"
echo "Author: $AUTHOR"
echo "Description: $DESCRIPTION"
echo ""

# Step 3: Prepare packaging directory structure
echo "🏗️  Preparing packaging directory structure..."
rm -rf "$DEB_DIR"
mkdir -p "$DEB_DIR/usr/bin"
mkdir -p "$DEB_DIR/etc/m-tunnel"
mkdir -p "$DEB_DIR/etc/m-tunnel/examples"
mkdir -p "$DEB_DIR/usr/share/doc/m-tunnel"
mkdir -p "$DEB_DIR/usr/share/m-tunnel/tests"
mkdir -p "$DEB_DIR/lib/systemd/system"
mkdir -p "$DEB_DIR/var/log/m-tunnel"
mkdir -p "$DEB_DIR/DEBIAN"

# Step 4: Copy binary
echo "📦 Copying binary..."
cp "target/release/$NAME" "$DEB_DIR/usr/bin/"

# Step 5: Copy configuration files from configs/ directory
echo "⚙️  Copying configuration files..."
cp "configs/m-tunnel.conf" "$DEB_DIR/etc/m-tunnel/" 2>/dev/null || echo "Warning: configs/m-tunnel.conf not found"
cp ".env" "$DEB_DIR/etc/m-tunnel/" 2>/dev/null || echo "Warning: .env not found"
cp "configs/m-tunnel.key" "$DEB_DIR/etc/m-tunnel/" 2>/dev/null || echo "Warning: configs/m-tunnel.key not found"

# Copy example configurations
cp "configs/m-tunnel.key.example" "$DEB_DIR/etc/m-tunnel/examples/" 2>/dev/null || echo "Warning: SSH key example not found"
cp "configs/known_hosts.template" "$DEB_DIR/etc/m-tunnel/examples/" 2>/dev/null || echo "Warning: known_hosts template not found"
cp "configs/real_ssh_test.toml" "$DEB_DIR/etc/m-tunnel/examples/" 2>/dev/null || echo "Warning: TOML config example not found"
cp "config.toml.example" "$DEB_DIR/etc/m-tunnel/examples/" 2>/dev/null || echo "Warning: TOML config example not found"

# Step 6: Copy documentation
echo "📚 Copying documentation..."
cp "README.md" "$DEB_DIR/usr/share/doc/m-tunnel/" 2>/dev/null || echo "Warning: README.md not found"
cp docs/*.md "$DEB_DIR/usr/share/doc/m-tunnel/" 2>/dev/null || echo "Warning: Documentation files not found"

# Step 7: Copy test scripts
echo "🧪 Copying test scripts..."
cp tests/*.sh "$DEB_DIR/usr/share/m-tunnel/tests/" 2>/dev/null || echo "Warning: Test scripts not found"
chmod +x "$DEB_DIR/usr/share/m-tunnel/tests/"*.sh 2>/dev/null || true

# Set proper permissions for config files
echo "🔒 Setting secure permissions..."
chmod 644 "$DEB_DIR/etc/m-tunnel/m-tunnel.conf" 2>/dev/null || true
chmod 600 "$DEB_DIR/etc/m-tunnel/.env" 2>/dev/null || true
chmod 600 "$DEB_DIR/etc/m-tunnel/m-tunnel.key" 2>/dev/null || true
chmod 644 "$DEB_DIR/etc/m-tunnel/examples/"* 2>/dev/null || true

# Step 8: Generate enhanced systemd service file (SSH2 ready)
SERVICE_NAME="m-tunnel"
echo "⚙️  Generating systemd service file..."
cat <<EOF > "$DEB_DIR/lib/systemd/system/${SERVICE_NAME}.service"
[Unit]
Description=$DESCRIPTION (SSH2 Enhanced)
After=network.target
Wants=network-online.target
After=network-online.target

[Service]
Type=simple
ExecStart=/usr/bin/$NAME --ssh2
ExecStartPre=/bin/bash -c 'echo "🚀 Starting M-Tunnel with SSH2 library support"'
Restart=always
RestartSec=5
User=root
Group=root
WorkingDirectory=/etc/m-tunnel
Environment=RUST_LOG=info
Environment=M_TUNNEL_CONFIG=/etc/m-tunnel/config.toml

# Security settings (enhanced)
NoNewPrivileges=true
ProtectSystem=strict
ProtectHome=true
ReadWritePaths=/var/log/m-tunnel /tmp/ssh-m-tunnel
PrivateTmp=true
ProtectKernelTunables=true
ProtectControlGroups=true
RestrictRealtime=true

[Install]
WantedBy=multi-user.target
EOF

# Step 9: Generate enhanced control file
echo "📦 Generating package control file..."
cat <<EOF > "$DEB_DIR/DEBIAN/control"
Package: $NAME
Version: $VERSION-1
Section: utils
Priority: optional
Architecture: amd64
Maintainer: $AUTHOR
Description: $DESCRIPTION
 Enhanced SSH tunnel management utility with native SSH2 library support.
 Features include:
 - Native Rust SSH2 implementation (no external SSH CLI dependency)
 - Multi-tunnel management with TOML configuration
 - Rate limiting and connection throttling
 - Comprehensive security features
 - Async performance with Tokio runtime
 - Both CLI and library-based SSH implementations
 .
 Configuration files organized in /etc/m-tunnel/ following Linux FHS standards.
 Documentation and examples included in /usr/share/doc/m-tunnel/.
 Test scripts available in /usr/share/m-tunnel/tests/.
EOF

# Step 10: Generate enhanced postinst script
echo "⚙️  Generating post-installation script..."
cat <<EOF > "$DEB_DIR/DEBIAN/postinst"
#!/bin/bash
set -e

echo "🚀 M-Tunnel SSH2 Enhanced - Post Installation"
echo "============================================="

# Create dedicated user for security
if ! id "m-tunnel" &>/dev/null; then
    useradd -r -s /bin/false -d /etc/m-tunnel -c "M-Tunnel Service User" m-tunnel
    echo "✅ Created m-tunnel user"
fi

# Create and secure directories
mkdir -p /var/log/m-tunnel
mkdir -p /etc/m-tunnel
touch /etc/m-tunnel/known_hosts

# Create SSH2 specific directories
mkdir -p /tmp/ssh-m-tunnel
mkdir -p /etc/m-tunnel/keys

# Set proper ownership and permissions
chown -R m-tunnel:m-tunnel /etc/m-tunnel
chown -R m-tunnel:m-tunnel /var/log/m-tunnel

# Secure permissions
chmod 750 /etc/m-tunnel
chmod 750 /var/log/m-tunnel
chmod 644 /etc/m-tunnel/m-tunnel.conf 2>/dev/null || true
chmod 600 /etc/m-tunnel/.env 2>/dev/null || true
chmod 600 /etc/m-tunnel/m-tunnel.key 2>/dev/null || true
chmod 644 /etc/m-tunnel/known_hosts

# SSH2 specific permissions
chmod 700 /tmp/ssh-m-tunnel
chown m-tunnel:m-tunnel /tmp/ssh-m-tunnel

# Reload systemd and enable service
systemctl daemon-reload
systemctl enable ${SERVICE_NAME}.service

echo ""
echo "🎉 M-Tunnel Installation Complete!"
echo "=================================="
echo ""
echo "📁 Configuration files location: /etc/m-tunnel/"
echo "📚 Documentation: /usr/share/doc/m-tunnel/"
echo "🧪 Test scripts: /usr/share/m-tunnel/tests/"
echo "📋 Examples: /etc/m-tunnel/examples/"
echo ""
echo "🚀 SSH2 Implementation Available!"
echo "- Use '--ssh2' flag for native SSH2 library"
echo "- TOML configuration support"
echo "- Enhanced security and performance"
echo ""
echo "⚙️  Setup Instructions:"
echo "1. Edit /etc/m-tunnel/.env with your SSH settings (legacy)"
echo "2. OR create /etc/m-tunnel/config.toml (recommended)"
echo "3. Edit tunnel definitions as needed"
echo "4. Add SSH host key: ssh-keyscan -H your-ssh-host >> /etc/m-tunnel/known_hosts"
echo "5. Start the service: systemctl start ${SERVICE_NAME}.service"
echo ""
echo "🧪 Testing:"
echo "- Quick test: /usr/share/m-tunnel/tests/test_quick.sh"
echo "- Stress test: /usr/share/m-tunnel/tests/test_stress.sh"
echo ""
echo "🔒 Security: Service runs as dedicated 'm-tunnel' user"
echo "📊 Logs: journalctl -u ${SERVICE_NAME} -f"
echo "🆘 Help: m-tunnel-rust --help"
EOF

chmod 755 "$DEB_DIR/DEBIAN/postinst"

# Step 11: Generate enhanced postrm script
echo "🗑️  Generating post-removal script..."
cat <<EOF > "$DEB_DIR/DEBIAN/postrm"
#!/bin/bash
set -e

echo "🗑️  M-Tunnel SSH2 Enhanced - Post Removal"
echo "=========================================="

case "\$1" in
    remove)
        echo "🛑 Stopping M-Tunnel service..."
        systemctl stop ${SERVICE_NAME}.service || true
        systemctl disable ${SERVICE_NAME}.service || true
        systemctl daemon-reload
        # Clean up SSH control sockets
        rm -rf /tmp/ssh-m-tunnel
        echo "✅ Service stopped and disabled"
        ;;
    purge)
        echo "🧹 Purging M-Tunnel data..."
        systemctl stop ${SERVICE_NAME}.service || true
        systemctl disable ${SERVICE_NAME}.service || true
        systemctl daemon-reload
        
        # Remove log directory on purge
        rm -rf /var/log/m-tunnel
        rm -rf /tmp/ssh-m-tunnel
        
        # Remove user (but keep home directory with configs)
        if id "m-tunnel" &>/dev/null; then
            userdel m-tunnel 2>/dev/null || true
            echo "✅ Removed m-tunnel user"
        fi
        
        echo ""
        echo "📁 Configuration files in /etc/m-tunnel/ preserved."
        echo "🗑️  Remove manually if no longer needed: rm -rf /etc/m-tunnel/"
        echo "📚 Documentation preserved in /usr/share/doc/m-tunnel/"
        ;;
esac

echo "✅ M-Tunnel removal completed"
EOF

chmod 755 "$DEB_DIR/DEBIAN/postrm"

# Step 12: Build the .deb package
echo ""
echo "📦 Building .deb package..."
dpkg-deb --build "$DEB_DIR"

echo ""
echo "🎉 Package Creation Complete!"
echo "============================="
echo "✅ Created: ${DEB_DIR}.deb"
echo ""
echo "📁 Installation will place files as follows:"
echo "   Binary:         /usr/bin/${NAME}"
echo "   Configuration:  /etc/m-tunnel/"
echo "   Examples:       /etc/m-tunnel/examples/"
echo "   Documentation:  /usr/share/doc/m-tunnel/"
echo "   Test Scripts:   /usr/share/m-tunnel/tests/"
echo "   Service:        /lib/systemd/system/${SERVICE_NAME}.service"
echo "   Logs:           /var/log/m-tunnel/"
echo ""
echo "🚀 SSH2 Features:"
echo "   • Native SSH2 library support (--ssh2 flag)"
echo "   • TOML configuration format"
echo "   • Enhanced security and performance"
echo "   • Comprehensive testing suite included"
echo ""
echo "🔧 Installation:"
echo "   sudo dpkg -i ${DEB_DIR}.deb"
echo ""
echo "🧪 Testing after installation:"
echo "   /usr/share/m-tunnel/tests/test_quick.sh"
