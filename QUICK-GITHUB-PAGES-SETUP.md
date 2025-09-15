# 🚀 Quick GitHub Pages Setup for M-Tunnel APT Repository

## Step-by-Step Setup

### 1. **Enable GitHub Pages** (5 minutes)

1. Go to your repository on GitHub: `https://github.com/YOUR_USERNAME/m-tunnel-rust`
2. Click **Settings** tab
3. Scroll down to **Pages** section
4. Set:
   - **Source**: Deploy from a branch
   - **Branch**: `gh-pages`
   - **Folder**: `/ (root)`
5. Click **Save**

### 2. **Set Up Repository Secrets** (3 minutes)

1. In your repository, go to **Settings** → **Secrets and variables** → **Actions**
2. Add these secrets:
   ```
   GPG_PRIVATE_KEY: [Your GPG private key - get from ./setup-apt-repo.sh]
   GPG_KEY_ID: [Your GPG key ID - displayed when you run setup]
   GPG_PASSPHRASE: [Your GPG key passphrase - if you set one]
   ```

### 3. **Create Your First Release** (2 minutes)

```bash
# Tag a release (triggers automatic build and deployment)
git tag v1.0.0
git push origin v1.0.0
```

## 🎯 **That's It!**

Your APT repository will be automatically built and deployed to:

```
🌐 Repository URL: https://YOUR_USERNAME.github.io/m-tunnel-rust/
📦 APT Repository: https://YOUR_USERNAME.github.io/m-tunnel-rust/apt
🔑 GPG Key: https://YOUR_USERNAME.github.io/m-tunnel-rust/gpg-key.asc
📜 Installer: https://YOUR_USERNAME.github.io/m-tunnel-rust/install.sh
```

## 📱 **How Users Install Your Package**

### Option 1: One-liner (Recommended)
```bash
curl -fsSL https://YOUR_USERNAME.github.io/m-tunnel-rust/install.sh | sudo bash
sudo apt install m-tunnel-rust
```

### Option 2: Manual
```bash
# Add GPG key
curl -fsSL https://YOUR_USERNAME.github.io/m-tunnel-rust/gpg-key.asc | sudo apt-key add -

# Add repository
echo "deb https://YOUR_USERNAME.github.io/m-tunnel-rust/apt stable main" | sudo tee /etc/apt/sources.list.d/m-tunnel.list

# Install
sudo apt update
sudo apt install m-tunnel-rust
```

## 🔄 **Automatic Updates**

Every time you create a new tag (e.g., `v1.0.1`), GitHub Actions will:

1. ✅ Build packages for all architectures (amd64, arm64, armhf)
2. ✅ Sign packages with your GPG key
3. ✅ Update repository metadata
4. ✅ Deploy to GitHub Pages automatically
5. ✅ Create GitHub release with .deb files

## 🛠️ **Local Development**

For manual control, you can also use:

```bash
# Set up GPG and repository structure
./setup-apt-repo.sh

# Build packages locally
./build-multi-arch.sh

# Deploy manually to GitHub Pages
./deploy-to-github-pages.sh
```

## 🎉 **Your Repository is Live!**

Users worldwide can now install your enhanced m-tunnel-rust with a single command. The repository automatically handles:

- 🔒 **Security**: GPG-signed packages
- 🏗️ **Multi-Architecture**: amd64, arm64, armhf support  
- ⚡ **Performance**: Optimized for fast downloads
- 🔄 **Auto-Updates**: CI/CD pipeline keeps everything current

**Share your repository**: `https://YOUR_USERNAME.github.io/m-tunnel-rust/`