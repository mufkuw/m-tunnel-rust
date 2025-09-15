#!/bin/bash
# Deploy APT repository to GitHub Pages

set -e

echo "🚀 Deploying M-Tunnel APT Repository to GitHub Pages"
echo "===================================================="

# Configuration
REPO_NAME="m-tunnel-rust"
GITHUB_PAGES_BRANCH="gh-pages"
APT_REPO_DIR="./apt-repository"
PAGES_DIR="./gh-pages-content"

# Check if we're in a git repository
if ! git rev-parse --git-dir >/dev/null 2>&1; then
    echo "❌ Error: Not in a git repository"
    exit 1
fi

# Check if APT repository exists
if [ ! -d "$APT_REPO_DIR" ]; then
    echo "❌ Error: APT repository not found at $APT_REPO_DIR"
    echo "   Run ./setup-apt-repo.sh first to create the repository"
    exit 1
fi

# Get current branch
CURRENT_BRANCH=$(git branch --show-current)
echo "📍 Current branch: $CURRENT_BRANCH"

# Stash any uncommitted changes
if ! git diff-index --quiet HEAD --; then
    echo "💾 Stashing uncommitted changes..."
    git stash push -m "Deploy script stash $(date)"
    STASHED=true
else
    STASHED=false
fi

echo "🧹 Preparing GitHub Pages content..."

# Create clean pages directory
rm -rf "$PAGES_DIR"
mkdir -p "$PAGES_DIR"

# Copy APT repository to pages directory
echo "📦 Copying APT repository..."
cp -r "$APT_REPO_DIR"/* "$PAGES_DIR/"

# Create index.html for the repository homepage
echo "📄 Creating repository homepage..."
cat > "$PAGES_DIR/index.html" <<EOF
<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>M-Tunnel APT Repository</title>
    <style>
        body {
            font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif;
            line-height: 1.6;
            color: #333;
            max-width: 800px;
            margin: 0 auto;
            padding: 20px;
            background: #f8f9fa;
        }
        .container {
            background: white;
            padding: 30px;
            border-radius: 8px;
            box-shadow: 0 2px 10px rgba(0,0,0,0.1);
        }
        .header {
            text-align: center;
            margin-bottom: 30px;
            padding-bottom: 20px;
            border-bottom: 2px solid #007acc;
        }
        .logo {
            font-size: 2.5em;
            margin-bottom: 10px;
            color: #007acc;
        }
        .install-box {
            background: #f1f3f4;
            border: 1px solid #dadce0;
            border-radius: 6px;
            padding: 20px;
            margin: 20px 0;
            font-family: 'Courier New', monospace;
        }
        .install-command {
            background: #2d3748;
            color: #e2e8f0;
            padding: 15px;
            border-radius: 6px;
            margin: 10px 0;
            overflow-x: auto;
        }
        .features {
            display: grid;
            grid-template-columns: repeat(auto-fit, minmax(250px, 1fr));
            gap: 20px;
            margin: 30px 0;
        }
        .feature {
            padding: 20px;
            background: #f8f9fa;
            border-radius: 6px;
            border-left: 4px solid #007acc;
        }
        .badge {
            display: inline-block;
            background: #007acc;
            color: white;
            padding: 4px 8px;
            border-radius: 12px;
            font-size: 0.8em;
            margin: 5px 5px 5px 0;
        }
        .stats {
            text-align: center;
            margin: 30px 0;
            padding: 20px;
            background: #e8f4fd;
            border-radius: 6px;
        }
        a {
            color: #007acc;
            text-decoration: none;
        }
        a:hover {
            text-decoration: underline;
        }
    </style>
</head>
<body>
    <div class="container">
        <div class="header">
            <div class="logo">🚇 M-Tunnel</div>
            <h1>APT Repository</h1>
            <p>Professional SSH tunneling utility for Linux systems</p>
        </div>

        <div class="stats">
            <h3>📊 Repository Status</h3>
            <span class="badge">✅ Active</span>
            <span class="badge">🔒 GPG Signed</span>
            <span class="badge">🏗️ Multi-Arch</span>
            <span class="badge">⚡ Auto-Updated</span>
        </div>

        <h2>🚀 Quick Installation</h2>
        <div class="install-box">
            <strong>One-liner installation:</strong>
            <div class="install-command">curl -fsSL https://$(echo "$(git config --get remote.origin.url)" | sed 's/.*github.com[:/]\([^/]*\)\/\([^.]*\).*/\1.github.io\/\2/')/install.sh | sudo bash</div>
        </div>

        <h2>📦 Manual Installation</h2>
        <div class="install-box">
            <strong>Add repository manually:</strong>
            <div class="install-command"># Add GPG key
curl -fsSL https://$(echo "$(git config --get remote.origin.url)" | sed 's/.*github.com[:/]\([^/]*\)\/\([^.]*\).*/\1.github.io\/\2/')/gpg-key.asc | sudo apt-key add -

# Add repository
echo "deb https://$(echo "$(git config --get remote.origin.url)" | sed 's/.*github.com[:/]\([^/]*\)\/\([^.]*\).*/\1.github.io\/\2/')/apt stable main" | sudo tee /etc/apt/sources.list.d/m-tunnel.list

# Install package
sudo apt update
sudo apt install m-tunnel-rust</div>
        </div>

        <h2>✨ Features</h2>
        <div class="features">
            <div class="feature">
                <h3>🔒 Enhanced Security</h3>
                <p>Strict SSH host verification, dedicated service user, and systemd security hardening</p>
            </div>
            <div class="feature">
                <h3>⚡ High Performance</h3>
                <p>Arc-based memory management, connection multiplexing, and intelligent reconnection</p>
            </div>
            <div class="feature">
                <h3>🔄 Auto-Reconnection</h3>
                <p>Automatic reconnection with exponential backoff and connection monitoring</p>
            </div>
            <div class="feature">
                <h3>🏗️ Multi-Architecture</h3>
                <p>Native support for amd64, arm64, and armhf architectures</p>
            </div>
        </div>

        <h2>📚 Documentation</h2>
        <ul>
            <li><a href="https://github.com/$(echo "$(git config --get remote.origin.url)" | sed 's/.*github.com[:/]\([^/]*\)\/\([^.]*\).*/\1\/\2/')/blob/master/README.md">Installation Guide</a></li>
            <li><a href="https://github.com/$(echo "$(git config --get remote.origin.url)" | sed 's/.*github.com[:/]\([^/]*\)\/\([^.]*\).*/\1\/\2/')/blob/master/APT-REPOSITORY.md">Repository Documentation</a></li>
            <li><a href="https://github.com/$(echo "$(git config --get remote.origin.url)" | sed 's/.*github.com[:/]\([^/]*\)\/\([^.]*\).*/\1\/\2/')/issues">Report Issues</a></li>
        </ul>

        <h2>🏷️ Available Packages</h2>
        <div class="install-box">
            <strong>m-tunnel-rust</strong> - SSH tunnel management utility
            <br><small>Architectures: amd64, arm64, armhf</small>
        </div>

        <footer style="text-align: center; margin-top: 40px; padding-top: 20px; border-top: 1px solid #dadce0; color: #666;">
            <p>Powered by GitHub Pages • <a href="https://github.com/$(echo "$(git config --get remote.origin.url)" | sed 's/.*github.com[:/]\([^/]*\)\/\([^.]*\).*/\1\/\2/')">Source Code</a> • Built with ❤️</p>
        </footer>
    </div>
</body>
</html>
EOF

# Create install script for one-liner installation
echo "📜 Creating installation script..."
cat > "$PAGES_DIR/install.sh" <<'EOF'
#!/bin/bash
# M-Tunnel APT Repository Installer

set -e

echo "🚇 Installing M-Tunnel APT Repository"
echo "====================================="

# Check if running as root
if [ "$EUID" -ne 0 ]; then
    echo "❌ Error: This script must be run as root (use sudo)"
    exit 1
fi

# Detect the GitHub Pages URL from the script location
SCRIPT_URL="${BASH_SOURCE[0]}"
if [[ $SCRIPT_URL =~ ^https?:// ]]; then
    REPO_URL=$(echo "$SCRIPT_URL" | sed 's/\/install\.sh$//')
else
    echo "❌ Error: Unable to determine repository URL"
    exit 1
fi

echo "📍 Repository URL: $REPO_URL"

# Add GPG key
echo "🔑 Adding GPG key..."
curl -fsSL "$REPO_URL/gpg-key.asc" | apt-key add - 2>/dev/null || {
    # Fallback for newer systems that don't support apt-key
    curl -fsSL "$REPO_URL/gpg-key.asc" | gpg --dearmor -o /usr/share/keyrings/m-tunnel.gpg
    KEYRING_OPTION="[signed-by=/usr/share/keyrings/m-tunnel.gpg]"
}

# Add repository
echo "📦 Adding repository..."
echo "deb ${KEYRING_OPTION:-} $REPO_URL/apt stable main" > /etc/apt/sources.list.d/m-tunnel.list

# Update package list
echo "🔄 Updating package list..."
apt update

echo "✅ M-Tunnel repository added successfully!"
echo ""
echo "🚀 Install M-Tunnel with:"
echo "   sudo apt install m-tunnel-rust"
echo ""
echo "📚 Documentation:"
echo "   https://github.com/YOUR_USERNAME/m-tunnel-rust"
EOF

# Make install script executable
chmod +x "$PAGES_DIR/install.sh"

# Check if gh-pages branch exists
if git show-ref --quiet refs/heads/$GITHUB_PAGES_BRANCH; then
    echo "🌿 Switching to $GITHUB_PAGES_BRANCH branch..."
    git checkout $GITHUB_PAGES_BRANCH
else
    echo "🌱 Creating $GITHUB_PAGES_BRANCH branch..."
    git checkout --orphan $GITHUB_PAGES_BRANCH
    git rm -rf . 2>/dev/null || true
fi

# Copy pages content to current directory
echo "📋 Copying content to GitHub Pages branch..."
cp -r "$PAGES_DIR"/* .
cp -r "$PAGES_DIR"/.* . 2>/dev/null || true

# Add all files
git add .

# Check if there are changes to commit
if git diff --cached --quiet; then
    echo "ℹ️  No changes to deploy"
else
    echo "💾 Committing changes..."
    git commit -m "Deploy APT repository to GitHub Pages - $(date)"
    
    echo "🚀 Pushing to GitHub..."
    git push origin $GITHUB_PAGES_BRANCH
    
    echo "✅ Successfully deployed to GitHub Pages!"
fi

# Switch back to original branch
echo "🔄 Switching back to $CURRENT_BRANCH..."
git checkout $CURRENT_BRANCH

# Restore stashed changes if any
if [ "$STASHED" = true ]; then
    echo "🔄 Restoring stashed changes..."
    git stash pop
fi

# Clean up
rm -rf "$PAGES_DIR"

echo ""
echo "🎉 Deployment Complete!"
echo "======================================"
echo ""
echo "📍 Your APT repository is now available at:"

# Extract GitHub username and repo name from remote URL
GITHUB_URL=$(git config --get remote.origin.url)
if [[ $GITHUB_URL =~ github\.com[:/]([^/]+)/([^.]+) ]]; then
    USERNAME="${BASH_REMATCH[1]}"
    REPO="${BASH_REMATCH[2]}"
    PAGES_URL="https://$USERNAME.github.io/$REPO"
    
    echo "   🌐 Repository: $PAGES_URL"
    echo "   🔑 GPG Key:    $PAGES_URL/gpg-key.asc"
    echo "   📜 Installer:  $PAGES_URL/install.sh"
    echo ""
    echo "👥 Users can install with:"
    echo "   curl -fsSL $PAGES_URL/install.sh | sudo bash"
    echo ""
    echo "⏱️  Note: It may take a few minutes for GitHub Pages to update"
else
    echo "   Check your GitHub repository settings for the Pages URL"
fi

echo ""
echo "🔧 Next steps:"
echo "   1. Enable GitHub Pages in repository settings if not already done"
echo "   2. Wait for GitHub Pages to deploy (usually 1-5 minutes)"
echo "   3. Test the installation on a clean system"
echo "   4. Share your repository with users!"
EOF