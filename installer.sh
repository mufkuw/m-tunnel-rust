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

# Step 3: Prepare packaging directory
rm -rf "$DEB_DIR"
mkdir -p "$DEB_DIR/usr/bin"
mkdir -p "$DEB_DIR/lib/systemd/system"
mkdir -p "$DEB_DIR/DEBIAN"

# Step 4: Copy binary
cp "target/release/$NAME" "$DEB_DIR/usr/bin/"

# Step 5: Generate systemd service file
cat <<EOF > "$DEB_DIR/lib/systemd/system/${NAME}.service"
[Unit]
Description=$DESCRIPTION
After=network.target

[Service]
ExecStart=/usr/bin/$NAME
Restart=always
User=nobody
Group=nogroup

[Install]
WantedBy=multi-user.target
EOF

# Step 6: Generate control file
cat <<EOF > "$DEB_DIR/DEBIAN/control"
Package: $NAME
Version: $VERSION-1
Section: utils
Priority: optional
Architecture: amd64
Maintainer: $AUTHOR
Description: $DESCRIPTION
EOF

# Step 7: Generate postinst script
cat <<EOF > "$DEB_DIR/DEBIAN/postinst"
#!/bin/bash
set -e
systemctl daemon-reexec || true
systemctl daemon-reload
systemctl enable ${NAME}.service
systemctl start ${NAME}.service
EOF

chmod 755 "$DEB_DIR/DEBIAN/postinst"

# Step 8: Generate postrm script
cat <<EOF > "$DEB_DIR/DEBIAN/postrm"
#!/bin/bash
set -e
systemctl stop ${NAME}.service || true
systemctl disable ${NAME}.service || true
systemctl daemon-reload
EOF

chmod 755 "$DEB_DIR/DEBIAN/postrm"

# Step 9: Build the .deb package
echo "Building .deb package..."
dpkg-deb --build "$DEB_DIR"

echo "Done. Created ${DEB_DIR}.deb"
