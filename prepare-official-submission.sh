#!/bin/bash
# Prepare m-tunnel-rust for submission to official Debian/Ubuntu repositories

set -e

PACKAGE_NAME="m-tunnel-rust"
VERSION=$(grep '^version' Cargo.toml | sed 's/version = "\(.*\)"/\1/')

echo "📋 Preparing M-Tunnel for Official Repository Submission"
echo "======================================================="
echo "Package: $PACKAGE_NAME"
echo "Version: $VERSION"
echo ""

# Check if this is a clean git state
if ! git diff-index --quiet HEAD --; then
    echo "⚠️  Warning: You have uncommitted changes"
    echo "   Please commit all changes before preparing for submission"
    read -p "Continue anyway? (y/N): " -n 1 -r
    echo
    if [[ ! $REPLY =~ ^[Yy]$ ]]; then
        exit 1
    fi
fi

echo "🔍 Checking package requirements for official repositories..."

# Check for required files
echo "📁 Checking required files..."
required_files=(
    "Cargo.toml"
    "Cargo.lock" 
    "src/main.rs"
    "README.md"
    "LICENSE"
    "debian/control"
    "debian/changelog"
    "debian/copyright"
    "debian/rules"
)

missing_files=()
for file in "${required_files[@]}"; do
    if [ ! -f "$file" ]; then
        missing_files+=("$file")
    fi
done

if [ ${#missing_files[@]} -gt 0 ]; then
    echo "❌ Missing required files for Debian packaging:"
    printf '   %s\n' "${missing_files[@]}"
    echo ""
    echo "🔧 Creating missing Debian packaging files..."
    
    # Create debian directory structure
    mkdir -p debian/source
    
    # Create debian/control
    if [ ! -f "debian/control" ]; then
        cat > debian/control <<EOF
Source: $PACKAGE_NAME
Section: utils
Priority: optional
Maintainer: $(grep '^authors' Cargo.toml | sed -E 's/authors = \["([^"]*)".*/\1/')
Build-Depends: debhelper-compat (= 13),
               dh-cargo (>= 25),
               cargo:native,
               rustc:native,
               libstd-rust-dev,
               pkg-config
Standards-Version: 4.6.0
Homepage: $(grep '^repository' Cargo.toml | sed 's/repository = "\(.*\)"/\1/')
Vcs-Git: $(grep '^repository' Cargo.toml | sed 's/repository = "\(.*\)"/\1/')
Vcs-Browser: $(grep '^repository' Cargo.toml | sed 's/repository = "\(.*\)"/\1/')
Rules-Requires-Root: no

Package: $PACKAGE_NAME
Architecture: any
Depends: \${misc:Depends}, \${shlibs:Depends}, openssh-client
Description: $(grep '^description' Cargo.toml | sed 's/description = "\(.*\)"/\1/')
 M-Tunnel is a lightweight and production-ready SSH tunnel utility written in 
 Rust. It provides both forward and reverse SSH tunneling capabilities with
 automatic reconnection, enhanced security features, and performance 
 optimizations.
 .
 Key features include:
  * Forward (local → remote) and reverse (remote → local) tunnels
  * Automatic reconnection with exponential backoff
  * SSH connection multiplexing for performance
  * Dedicated service user for security
  * Rate limiting and connection monitoring
  * Systemd integration with security hardening
EOF
    fi
    
    # Create debian/changelog
    if [ ! -f "debian/changelog" ]; then
        cat > debian/changelog <<EOF
$PACKAGE_NAME ($VERSION-1) unstable; urgency=medium

  * Initial release
  * SSH tunneling utility with enhanced security
  * Multi-architecture support (amd64, arm64, armhf)
  * Systemd integration with hardening
  * Automatic reconnection and monitoring

 -- $(grep '^authors' Cargo.toml | sed -E 's/authors = \["([^"]*)".*/\1/')  $(date -R)
EOF
    fi
    
    # Create debian/copyright
    if [ ! -f "debian/copyright" ]; then
        cat > debian/copyright <<EOF
Format: https://www.debian.org/doc/packaging-manuals/copyright-format/1.0/
Upstream-Name: $PACKAGE_NAME
Upstream-Contact: $(grep '^authors' Cargo.toml | sed -E 's/authors = \["([^"]*)".*/\1/')
Source: $(grep '^repository' Cargo.toml | sed 's/repository = "\(.*\)"/\1/')

Files: *
Copyright: $(date +%Y) $(grep '^authors' Cargo.toml | sed -E 's/authors = \["([^"]*)".*/\1/')
License: MIT

License: MIT
 Permission is hereby granted, free of charge, to any person obtaining a
 copy of this software and associated documentation files (the "Software"),
 to deal in the Software without restriction, including without limitation
 the rights to use, copy, modify, merge, publish, distribute, sublicense,
 and/or sell copies of the Software, and to permit persons to whom the
 Software is furnished to do so, subject to the following conditions:
 .
 The above copyright notice and this permission notice shall be included
 in all copies or substantial portions of the Software.
 .
 THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS
 OR IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF
 MERCHANTABILITY, FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT.
 IN NO EVENT SHALL THE AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY
 CLAIM, DAMAGES OR OTHER LIABILITY, WHETHER IN AN ACTION OF CONTRACT,
 TORT OR OTHERWISE, ARISING FROM, OUT OF OR IN CONNECTION WITH THE
 SOFTWARE OR THE USE OR OTHER DEALINGS IN THE SOFTWARE.
EOF
    fi
    
    # Create debian/rules
    if [ ! -f "debian/rules" ]; then
        cat > debian/rules <<'EOF'
#!/usr/bin/make -f

%:
	dh $@ --buildsystem=cargo

override_dh_auto_install:
	dh_auto_install
	# Install systemd service
	install -D -m 644 m-tunnel.service debian/m-tunnel-rust/lib/systemd/system/m-tunnel.service
	# Install configuration files
	install -D -m 644 m-tunnel.conf debian/m-tunnel-rust/etc/m-tunnel/m-tunnel.conf
	install -D -m 600 .env debian/m-tunnel-rust/etc/m-tunnel/.env
	install -D -m 644 known_hosts.template debian/m-tunnel-rust/etc/m-tunnel/known_hosts
	# Create log directory
	install -d debian/m-tunnel-rust/var/log/m-tunnel

override_dh_systemd_enable:
	dh_systemd_enable --name=m-tunnel

override_dh_systemd_start:
	dh_systemd_start --name=m-tunnel
EOF
        chmod +x debian/rules
    fi
    
    # Create debian/source/format
    if [ ! -f "debian/source/format" ]; then
        echo "3.0 (quilt)" > debian/source/format
    fi
    
    # Create debian/watch (for upstream monitoring)
    if [ ! -f "debian/watch" ]; then
        cat > debian/watch <<EOF
version=4
opts="filenamemangle=s%(?:.*?)?v?(\d[\d.]*)\.tar\.gz%m-tunnel-rust-\$1.tar.gz%" \\
$(grep '^repository' Cargo.toml | sed 's/repository = "\(.*\)"/\1/')/tags \\
(?:.*?/)?v?(\d[\d.]*)\.tar\.gz debian uupdate
EOF
    fi
    
    echo "✅ Created Debian packaging files"
fi

# Check for LICENSE file
if [ ! -f "LICENSE" ]; then
    echo "❌ Missing LICENSE file"
    echo "   Creating MIT license file..."
    cat > LICENSE <<EOF
MIT License

Copyright (c) $(date +%Y) $(grep '^authors' Cargo.toml | sed -E 's/authors = \["([^"]*)".*/\1/')

Permission is hereby granted, free of charge, to any person obtaining a copy
of this software and associated documentation files (the "Software"), to deal
in the Software without restriction, including without limitation the rights
to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
copies of the Software, and to permit persons to whom the Software is
furnished to do so, subject to the following conditions:

The above copyright notice and this permission notice shall be included in all
copies or substantial portions of the Software.

THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
SOFTWARE.
EOF
    echo "✅ Created LICENSE file"
fi

echo ""
echo "🔧 Running package quality checks..."

# Check if debuild is available
if command -v debuild >/dev/null 2>&1; then
    echo "📦 Testing Debian package build..."
    debuild -us -uc --lintian-opts --profile debian
    
    echo "✅ Package builds successfully"
else
    echo "⚠️  debuild not found. Install with: apt install devscripts"
    echo "   This is required for official repository submission"
fi

# Check if lintian is available for quality checks
if command -v lintian >/dev/null 2>&1; then
    echo "🔍 Running lintian quality checks..."
    if [ -f "../${PACKAGE_NAME}_${VERSION}-1_amd64.deb" ]; then
        lintian "../${PACKAGE_NAME}_${VERSION}-1_amd64.deb" || true
    else
        echo "   No .deb file found to check"
    fi
else
    echo "⚠️  lintian not found. Install with: apt install lintian"
fi

echo ""
echo "📋 Official Repository Submission Checklist:"
echo ""
echo "✅ Package Structure:"
echo "   [$([ -d debian ] && echo "✓" || echo "✗")] debian/ directory with proper packaging files"
echo "   [$([ -f LICENSE ] && echo "✓" || echo "✗")] LICENSE file"
echo "   [$([ -f README.md ] && echo "✓" || echo "✗")] README.md documentation"
echo "   [$([ -f Cargo.lock ] && echo "✓" || echo "✗")] Cargo.lock for reproducible builds"
echo ""
echo "📋 Next Steps for Official Submission:"
echo ""
echo "🐧 For Debian:"
echo "   1. Create Salsa account: https://salsa.debian.org/"
echo "   2. Find a sponsor in the Rust team: https://wiki.debian.org/Teams/RustPackaging"
echo "   3. Submit RFP (Request for Package): https://www.debian.org/devel/wnpp/"
echo "   4. Follow mentors.debian.net process"
echo ""
echo "🟠 For Ubuntu:"
echo "   1. Create Launchpad account: https://launchpad.net/"
echo "   2. Upload to your PPA first: dput ppa:yourusername/ppa"
echo "   3. Request inclusion in Universe: https://wiki.ubuntu.com/UbuntuDevelopment/NewPackages"
echo ""
echo "🔗 Useful Resources:"
echo "   - Debian New Maintainer Guide: https://www.debian.org/doc/manuals/maint-guide/"
echo "   - Ubuntu Packaging Guide: https://packaging.ubuntu.com/html/"
echo "   - Rust Packaging in Debian: https://wiki.debian.org/Teams/RustPackaging"
echo ""
echo "💡 Tip: Start with your own APT repository and PPA to gain experience"
echo "    before submitting to official repositories."