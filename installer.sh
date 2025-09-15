#!/bin/bash
set -e

# Step 1: Build the Rust project
echo "Building the Rust project..."
cargo build --release

# Step 2: Extract metadata from Cargo.toml
echo "Extracting metadata from Cargo.toml..."
NAME=$(grep -m1 '^name' Cargo.toml | sed 's/name *= *"\(.*\)"/\1/')
VERSION=$(grep -m1 '^version' Cargo.toml | sed 's/version *= *"\(.*\)"/\1/')
AUTHOR=$(grep -m1 '^authors' Cargo.toml | sed -E 's/authors *= *\["([^"]*)".*/\1/')
DESCRIPTION=$(grep -m1 '^description' Cargo.toml | sed 's/description *= *"\(.*\)"/\1/')

DESCRIPTION=${DESCRIPTION:-"A utility written in Rust."}

DEB_DIR="${NAME}_${VERSION}-1"

echo "Package: $NAME"
echo "Version: $VERSION"
echo "Author: $AUTHOR"
echo "Description: $DESCRIPTION"

# Step 3: Prepare packaging directory structure
rm -rf "$DEB_DIR"
mkdir -p "$DEB_DIR/usr/bin"
mkdir -p "$DEB_DIR/etc/m-tunnel"
mkdir -p "$DEB_DIR/lib/systemd/system"
mkdir -p "$DEB_DIR/var/log/m-tunnel"
mkdir -p "$DEB_DIR/DEBIAN"

# Step 4: Copy binary
cp "target/release/$NAME" "$DEB_DIR/usr/bin/"

# Step 5: Copy configuration files to /etc/m-tunnel/
cp "m-tunnel.conf" "$DEB_DIR/etc/m-tunnel/" || echo "Warning: m-tunnel.conf not found"
cp ".env" "$DEB_DIR/etc/m-tunnel/" || echo "Warning: .env not found"
cp "m-tunnel.key" "$DEB_DIR/etc/m-tunnel/" || echo "Warning: m-tunnel.key not found"

# Set proper permissions for config files
chmod 644 "$DEB_DIR/etc/m-tunnel/m-tunnel.conf" 2>/dev/null || true
chmod 600 "$DEB_DIR/etc/m-tunnel/.env" 2>/dev/null || true
chmod 600 "$DEB_DIR/etc/m-tunnel/m-tunnel.key" 2>/dev/null || true

# Step 6: Generate systemd service file (using m-tunnel as service name)
SERVICE_NAME="m-tunnel"
cat <<EOF > "$DEB_DIR/lib/systemd/system/${SERVICE_NAME}.service"
[Unit]
Description=$DESCRIPTION
After=network.target

[Service]
Type=simple
ExecStart=/usr/bin/$NAME
Restart=always
RestartSec=5
User=root
Group=root
WorkingDirectory=/etc/m-tunnel
Environment=RUST_LOG=info

# Security settings
NoNewPrivileges=true
ProtectSystem=strict
ProtectHome=true
ReadWritePaths=/var/log/m-tunnel
PrivateTmp=true

[Install]
WantedBy=multi-user.target
EOF

# Step 7: Generate control file
cat <<EOF > "$DEB_DIR/DEBIAN/control"
Package: $NAME
Version: $VERSION-1
Section: utils
Priority: optional
Architecture: amd64
Maintainer: $AUTHOR
Description: $DESCRIPTION
 .
 SSH tunnel management utility with configuration files
 organized in /etc/m-tunnel/ following Linux FHS standards.
EOF

# Step 8: Generate postinst script
cat <<EOF > "$DEB_DIR/DEBIAN/postinst"
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
systemctl enable ${SERVICE_NAME}.service

echo "=== M-Tunnel Installation Complete ==="
echo "Configuration files are located in /etc/m-tunnel/"
echo "1. Edit /etc/m-tunnel/.env with your SSH settings"
echo "2. Edit /etc/m-tunnel/m-tunnel.conf with tunnel definitions"  
echo "3. Add SSH host key: ssh-keyscan -H your-ssh-host >> /etc/m-tunnel/known_hosts"
echo "4. Start the service: systemctl start ${SERVICE_NAME}.service"
echo ""
echo "Security: Service runs as dedicated 'm-tunnel' user"
echo "Logs: journalctl -u ${SERVICE_NAME} -f"
EOF

chmod 755 "$DEB_DIR/DEBIAN/postinst"

# Step 9: Generate postrm script
cat <<EOF > "$DEB_DIR/DEBIAN/postrm"
#!/bin/bash
set -e

case "\$1" in
    remove)
        systemctl stop ${SERVICE_NAME}.service || true
        systemctl disable ${SERVICE_NAME}.service || true
        systemctl daemon-reload
        # Clean up SSH control sockets
        rm -rf /tmp/ssh-m-tunnel
        ;;
    purge)
        systemctl stop ${SERVICE_NAME}.service || true
        systemctl disable ${SERVICE_NAME}.service || true
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

# Step 10: Build the .deb package
echo "Building .deb package..."
dpkg-deb --build "$DEB_DIR"

echo "Done. Created ${DEB_DIR}.deb"
echo ""
echo "Installation will place files as follows:"
echo "  Binary:        /usr/bin/${NAME}"
echo "  Configuration: /etc/m-tunnel/"
echo "  Service:       /lib/systemd/system/${SERVICE_NAME}.service"
echo "  Logs:          /var/log/m-tunnel/"
