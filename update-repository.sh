#!/bin/bash
# Update APT repository with new packages and generate metadata

set -e

REPO_DIR="m-tunnel-apt-repo"
PACKAGE_NAME="m-tunnel-rust"
DISTRIBUTIONS="jammy focal bullseye bookworm"
ARCHITECTURES="amd64 arm64 armhf"
COMPONENT="main"

# Get GPG key ID
GPG_KEY_ID=$(gpg --list-secret-keys --keyid-format LONG 2>/dev/null | grep sec | head -n1 | cut -d'/' -f2 | cut -d' ' -f1 || echo "")

if [ -z "$GPG_KEY_ID" ]; then
    echo "❌ No GPG key found. Run ./setup-apt-repo.sh first"
    exit 1
fi

echo "🔄 Updating APT Repository"
echo "========================="
echo "GPG Key: $GPG_KEY_ID"
echo "Repository: $REPO_DIR"
echo ""

# Check if repository exists
if [ ! -d "$REPO_DIR" ]; then
    echo "❌ Repository directory not found. Run ./setup-apt-repo.sh first"
    exit 1
fi

# Copy new packages from current directory
echo "📦 Copying packages..."
if [ -f "${PACKAGE_NAME}_*.deb" ]; then
    for deb_file in ${PACKAGE_NAME}_*.deb; do
        echo "  📥 Found: $deb_file"
        
        # Extract package info
        ARCH=$(dpkg-deb -f "$deb_file" Architecture)
        
        # Copy to all distributions (you might want to customize this)
        for dist in $DISTRIBUTIONS; do
            pool_dir="$REPO_DIR/pool/$dist/$COMPONENT/binary-$ARCH"
            mkdir -p "$pool_dir"
            cp "$deb_file" "$pool_dir/"
            echo "    📂 Copied to: $pool_dir/"
        done
    done
else
    echo "⚠️  No .deb packages found in current directory"
    echo "   Build packages first with: ./installer.sh"
fi

# Generate Packages files for each distribution and architecture
echo ""
echo "📋 Generating repository metadata..."

for dist in $DISTRIBUTIONS; do
    echo "  🐧 Processing distribution: $dist"
    
    dist_dir="$REPO_DIR/dists/$dist"
    mkdir -p "$dist_dir/$COMPONENT"
    
    for arch in $ARCHITECTURES; do
        binary_dir="$dist_dir/$COMPONENT/binary-$arch"
        pool_dir="$REPO_DIR/pool/$dist/$COMPONENT/binary-$arch"
        
        mkdir -p "$binary_dir"
        
        if [ -d "$pool_dir" ] && [ "$(ls -A $pool_dir 2>/dev/null)" ]; then
            echo "    📄 Generating Packages file for $arch..."
            
            # Generate Packages file
            cd "$REPO_DIR"
            dpkg-scanpackages "pool/$dist/$COMPONENT/binary-$arch" /dev/null > "dists/$dist/$COMPONENT/binary-$arch/Packages"
            
            # Compress Packages file
            gzip -9c "dists/$dist/$COMPONENT/binary-$arch/Packages" > "dists/$dist/$COMPONENT/binary-$arch/Packages.gz"
            
            cd - > /dev/null
        else
            echo "    ⚠️  No packages found for $arch"
        fi
    done
    
    # Generate Release file for distribution
    echo "  📄 Generating Release file for $dist..."
    
    release_file="$dist_dir/Release"
    cat > "$release_file" <<EOF
Origin: M-Tunnel Repository
Label: M-Tunnel
Suite: $dist
Codename: $dist
Architectures: $(echo $ARCHITECTURES | tr ' ' ' ')
Components: $COMPONENT
Description: M-Tunnel SSH tunneling utility repository
Date: $(date -u '+%a, %d %b %Y %H:%M:%S UTC')
EOF

    # Generate checksums
    cd "$dist_dir"
    
    # MD5Sum
    echo "MD5Sum:" >> Release
    find . -name "Packages*" -exec md5sum {} \; | sed 's|\./||' >> Release
    
    # SHA1
    echo "SHA1:" >> Release  
    find . -name "Packages*" -exec sha1sum {} \; | sed 's|\./||' >> Release
    
    # SHA256
    echo "SHA256:" >> Release
    find . -name "Packages*" -exec sha256sum {} \; | sed 's|\./||' >> Release
    
    cd - > /dev/null
    
    # Sign Release file
    echo "  🔐 Signing Release file..."
    cd "$dist_dir"
    gpg --default-key "$GPG_KEY_ID" --armor --detach-sign --output Release.gpg Release
    gpg --default-key "$GPG_KEY_ID" --clear-sign --output InRelease Release
    cd - > /dev/null
done

echo ""
echo "✅ Repository update complete!"
echo ""
echo "📊 Repository Statistics:"
for dist in $DISTRIBUTIONS; do
    echo "  📂 $dist:"
    for arch in $ARCHITECTURES; do
        pool_dir="$REPO_DIR/pool/$dist/$COMPONENT/binary-$arch"
        if [ -d "$pool_dir" ]; then
            count=$(ls -1 "$pool_dir"/*.deb 2>/dev/null | wc -l)
            echo "    $arch: $count packages"
        fi
    done
done

echo ""
echo "🌐 Upload the '$REPO_DIR' directory to your web server"
echo "📋 Share the installation instructions with users"
echo "🔑 Public key is available at: $REPO_DIR/public.key"