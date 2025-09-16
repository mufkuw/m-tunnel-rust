#!/bin/bash
# Setup script for creating an APT repository for m-tunnel-rust
# This script helps you create and maintain your own APT repository

set -e

REPO_NAME="m-tunnel-apt-repo"
PACKAGE_NAME="m-tunnel-rust"
MAINTAINER_NAME="Muffaddal Kalla"
MAINTAINER_EMAIL="mufkuw@gmail.com"
REPO_DOMAIN="your-domain.com"  # Change this to your domain
DISTRIBUTIONS="jammy focal bullseye bookworm"  # Ubuntu 22.04, 20.04, Debian 11, 12
ARCHITECTURES="amd64 arm64 armhf"

echo "🚀 M-Tunnel APT Repository Setup"
echo "=================================="
echo "This will help you create your own APT repository for $PACKAGE_NAME"
echo ""

# Step 1: Check dependencies
echo "📋 Checking dependencies..."
command -v gpg >/dev/null 2>&1 || { echo "❌ GPG is required but not installed. Install with: apt install gnupg"; exit 1; }
command -v dpkg-scanpackages >/dev/null 2>&1 || { echo "❌ dpkg-dev is required. Install with: apt install dpkg-dev"; exit 1; }
command -v apt-ftparchive >/dev/null 2>&1 || { echo "❌ apt-utils is required. Install with: apt install apt-utils"; exit 1; }

echo "✅ All dependencies found"

# Step 2: Create GPG key for signing (if it doesn't exist)
echo ""
echo "🔐 Setting up GPG signing key..."

GPG_KEY_ID=$(gpg --list-secret-keys --keyid-format LONG 2>/dev/null | grep sec | head -n1 | cut -d'/' -f2 | cut -d' ' -f1 || echo "")

if [ -z "$GPG_KEY_ID" ]; then
    echo "No GPG key found. Creating one..."
    echo "Please provide the following information for your GPG key:"
    
    read -p "Full Name [$MAINTAINER_NAME]: " USER_NAME
    USER_NAME=${USER_NAME:-$MAINTAINER_NAME}
    
    read -p "Email [$MAINTAINER_EMAIL]: " USER_EMAIL
    USER_EMAIL=${USER_EMAIL:-$MAINTAINER_EMAIL}
    
    # Create GPG key batch file
    cat > /tmp/gpg-batch <<EOF
%echo Generating GPG key for APT repository
Key-Type: RSA
Key-Length: 4096
Subkey-Type: RSA
Subkey-Length: 4096
Name-Real: $USER_NAME
Name-Email: $USER_EMAIL
Expire-Date: 2y
Passphrase: 
%commit
%echo GPG key generation complete
EOF

    echo "🔑 Generating GPG key (this may take a while)..."
    gpg --batch --generate-key /tmp/gpg-batch
    rm /tmp/gpg-batch
    
    GPG_KEY_ID=$(gpg --list-secret-keys --keyid-format LONG | grep sec | head -n1 | cut -d'/' -f2 | cut -d' ' -f1)
    echo "✅ GPG key created: $GPG_KEY_ID"
else
    echo "✅ Found existing GPG key: $GPG_KEY_ID"
fi

# Step 3: Create repository structure
echo ""
echo "📁 Creating repository structure..."

mkdir -p $REPO_NAME/{pool,dists}
mkdir -p $REPO_NAME/scripts

# Create pool directory structure
for dist in $DISTRIBUTIONS; do
    for arch in $ARCHITECTURES; do
        mkdir -p "$REPO_NAME/pool/$dist/main/binary-$arch"
        mkdir -p "$REPO_NAME/dists/$dist/main/binary-$arch"
    done
done

echo "✅ Repository structure created"

# Step 4: Export public key for users
echo ""
echo "🔑 Exporting public key..."
gpg --armor --export $GPG_KEY_ID > $REPO_NAME/public.key
echo "✅ Public key exported to $REPO_NAME/public.key"

echo ""
echo "🎉 Repository setup complete!"
echo ""
echo "📋 Next steps:"
echo "1. Copy your .deb packages to the appropriate pool directories"
echo "2. Run ./update-repository.sh to generate repository metadata"
echo "3. Upload the repository to your web server or GitHub Pages"
echo "4. Share the installation instructions with users"
echo ""
echo "🔑 Your GPG Key ID: $GPG_KEY_ID"
echo "📂 Repository directory: $REPO_NAME/"
echo "🌐 Configure your domain in the scripts: $REPO_DOMAIN"