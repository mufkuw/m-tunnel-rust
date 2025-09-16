#!/bin/bash
# Build m-tunnel-rust packages for multiple architectures

set -e

PACKAGE_NAME="m-tunnel-rust"
VERSION=$(grep '^version' Cargo.toml | sed 's/version = "\(.*\)"/\1/')
TARGETS="x86_64-unknown-linux-gnu aarch64-unknown-linux-gnu armv7-unknown-linux-gnueabihf"

echo "🔨 Building M-Tunnel for Multiple Architectures"
echo "==============================================="
echo "Package: $PACKAGE_NAME"
echo "Version: $VERSION"
echo "Targets: $TARGETS"
echo ""

# Check if cross is installed
if ! command -v cross &> /dev/null; then
    echo "📥 Installing 'cross' for cross-compilation..."
    cargo install cross --git https://github.com/cross-rs/cross
fi

# Clean previous builds
echo "🧹 Cleaning previous builds..."
cargo clean

# Build for each target
for target in $TARGETS; do
    echo ""
    echo "🔨 Building for target: $target"
    
    # Determine architecture name for .deb package
    case $target in
        "x86_64-unknown-linux-gnu")
            deb_arch="amd64"
            ;;
        "aarch64-unknown-linux-gnu") 
            deb_arch="arm64"
            ;;
        "armv7-unknown-linux-gnueabihf")
            deb_arch="armhf"
            ;;
        *)
            echo "⚠️  Unknown target: $target"
            continue
            ;;
    esac
    
    echo "  Target: $target -> Debian arch: $deb_arch"
    
    # Add target if not already added
    rustup target add $target 2>/dev/null || true
    
    # Build with cross
    echo "  🚀 Compiling..."
    cross build --target $target --release
    
    if [ $? -eq 0 ]; then
        echo "  ✅ Build successful for $target"
        
        # Create architecture-specific package
        echo "  📦 Creating .deb package..."
        
        # Create temp directory for this architecture
        DEB_DIR="${PACKAGE_NAME}_${VERSION}-1_${deb_arch}"
        rm -rf "$DEB_DIR"
        
        # Create package structure
        mkdir -p "$DEB_DIR/usr/bin"
        mkdir -p "$DEB_DIR/etc/m-tunnel"
        mkdir -p "$DEB_DIR/lib/systemd/system" 
        mkdir -p "$DEB_DIR/var/log/m-tunnel"
        mkdir -p "$DEB_DIR/DEBIAN"
        
        # Copy binary for this target
        cp "target/$target/release/$PACKAGE_NAME" "$DEB_DIR/usr/bin/"
        
        # Copy config files
        cp "m-tunnel.conf" "$DEB_DIR/etc/m-tunnel/" 2>/dev/null || echo "Warning: m-tunnel.conf not found"
        cp ".env" "$DEB_DIR/etc/m-tunnel/" 2>/dev/null || echo "Warning: .env not found"
        cp "m-tunnel.key" "$DEB_DIR/etc/m-tunnel/" 2>/dev/null || echo "Warning: m-tunnel.key not found"
        cp "known_hosts.template" "$DEB_DIR/etc/m-tunnel/known_hosts" 2>/dev/null || echo "Warning: known_hosts.template not found"
        
        # Set permissions
        chmod 644 "$DEB_DIR/etc/m-tunnel/m-tunnel.conf" 2>/dev/null || true
        chmod 600 "$DEB_DIR/etc/m-tunnel/.env" 2>/dev/null || true  
        chmod 600 "$DEB_DIR/etc/m-tunnel/m-tunnel.key" 2>/dev/null || true
        chmod 644 "$DEB_DIR/etc/m-tunnel/known_hosts" 2>/dev/null || true
        
        # Create systemd service file
        SERVICE_NAME="m-tunnel"
        cat > "$DEB_DIR/lib/systemd/system/${SERVICE_NAME}.service" <<EOF
[Unit]
Description=M-Tunnel SSH Tunnel Service
After=network.target

[Service]
Type=simple
ExecStart=/usr/bin/$PACKAGE_NAME
Restart=always
RestartSec=5
User=m-tunnel
Group=m-tunnel
WorkingDirectory=/etc/m-tunnel
Environment=RUST_LOG=info

# Enhanced Security settings
NoNewPrivileges=true
ProtectSystem=strict
ProtectHome=true
ProtectKernelTunables=true
ProtectKernelModules=true
ProtectControlGroups=true
RestrictSUIDSGID=true
RestrictRealtime=true
LockPersonality=true
MemoryDenyWriteExecute=true
PrivateTmp=true
PrivateDevices=true
SystemCallFilter=@system-service
SystemCallErrorNumber=EPERM
ReadWritePaths=/var/log/m-tunnel /tmp
RemainAfterExit=false

[Install]
WantedBy=multi-user.target
EOF

        # Extract metadata
        AUTHOR=$(grep '^authors' Cargo.toml | sed -E 's/authors = \["([^"]*)".*/\1/')
        DESCRIPTION=$(grep '^description' Cargo.toml | sed 's/description = "\(.*\)"/\1/')
        
        # Create control file
        cat > "$DEB_DIR/DEBIAN/control" <<EOF
Package: $PACKAGE_NAME
Version: $VERSION-1
Section: utils
Priority: optional
Architecture: $deb_arch
Maintainer: $AUTHOR
Description: $DESCRIPTION
 .
 SSH tunnel management utility with configuration files
 organized in /etc/m-tunnel/ following Linux FHS standards.
 Built for $deb_arch architecture.
Depends: openssh-client
EOF

        # Create postinst script (same for all architectures)
        cat > "$DEB_DIR/DEBIAN/postinst" <<'EOF'
#!/bin/bash
set -e

# Create dedicated user for security
if ! id "m-tunnel" &>/dev/null; then
    useradd -r -s /bin/false -d /etc/m-tunnel -c "M-Tunnel Service User" m-tunnel
    echo "Created m-tunnel user"
fi

# Create and secure directories  
mkdir -p /var/log/m-tunnel
mkdir -p /etc/m-tunnel
touch /etc/m-tunnel/known_hosts

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

# Allow m-tunnel user to create SSH control sockets
mkdir -p /tmp/ssh-m-tunnel
chown m-tunnel:m-tunnel /tmp/ssh-m-tunnel
chmod 700 /tmp/ssh-m-tunnel

# Reload systemd and enable service
systemctl daemon-reload
systemctl enable m-tunnel.service

echo "=== M-Tunnel Installation Complete ==="
echo "Configuration files are located in /etc/m-tunnel/"
echo "1. Edit /etc/m-tunnel/.env with your SSH settings"
echo "2. Edit /etc/m-tunnel/m-tunnel.conf with tunnel definitions"
echo "3. Add SSH host key: ssh-keyscan -H your-ssh-host >> /etc/m-tunnel/known_hosts"
echo "4. Start the service: systemctl start m-tunnel.service"
echo ""
echo "Security: Service runs as dedicated 'm-tunnel' user"
echo "Logs: journalctl -u m-tunnel -f"
EOF

        chmod 755 "$DEB_DIR/DEBIAN/postinst"
        
        # Create postrm script
        cat > "$DEB_DIR/DEBIAN/postrm" <<'EOF'
#!/bin/bash
set -e

case "$1" in
    remove)
        systemctl stop m-tunnel.service || true
        systemctl disable m-tunnel.service || true
        systemctl daemon-reload
        # Clean up SSH control sockets
        rm -rf /tmp/ssh-m-tunnel
        ;;
    purge)
        systemctl stop m-tunnel.service || true
        systemctl disable m-tunnel.service || true
        systemctl daemon-reload
        
        # Remove log directory on purge
        rm -rf /var/log/m-tunnel
        rm -rf /tmp/ssh-m-tunnel
        
        # Remove user (but keep home directory with configs)
        if id "m-tunnel" &>/dev/null; then
            userdel m-tunnel 2>/dev/null || true
            echo "Removed m-tunnel user"
        fi
        
        echo "Configuration files in /etc/m-tunnel/ preserved."
        echo "Remove manually if no longer needed: rm -rf /etc/m-tunnel/"
        ;;
esac
EOF
        
        chmod 755 "$DEB_DIR/DEBIAN/postrm"
        
        # Build .deb package
        echo "  📦 Building .deb package..."
        dpkg-deb --build "$DEB_DIR"
        
        if [ $? -eq 0 ]; then
            echo "  ✅ Package created: ${DEB_DIR}.deb"
        else
            echo "  ❌ Package creation failed for $target"
        fi
        
        # Clean up temp directory
        rm -rf "$DEB_DIR"
        
    else
        echo "  ❌ Build failed for $target"
    fi
done

echo ""
echo "🎉 Multi-architecture build complete!"
echo ""
echo "📦 Generated packages:"
ls -la *.deb 2>/dev/null || echo "No packages generated"

echo ""
echo "📋 Next steps:"
echo "1. Run ./update-repository.sh to add packages to APT repository"
echo "2. Upload repository to your web server"
echo "3. Share installation instructions with users"