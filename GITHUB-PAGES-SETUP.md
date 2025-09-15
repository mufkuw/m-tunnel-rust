# 🚀 Deploying Your APT Repository on GitHub Pages

This guide shows you how to host your m-tunnel-rust APT repository on GitHub Pages for free, making it accessible to users worldwide.

## 📋 Prerequisites

- GitHub account
- Your m-tunnel-rust repository on GitHub
- GPG key for package signing
- Built packages ready for distribution

## 🏗️ GitHub Pages Setup

### Step 1: Repository Configuration

1. **Go to your GitHub repository settings**:
   ```
   https://github.com/YOUR_USERNAME/m-tunnel-rust/settings
   ```

2. **Enable GitHub Pages**:
   - Scroll to "Pages" section
   - Source: "Deploy from a branch"
   - Branch: `gh-pages` (we'll create this)
   - Folder: `/ (root)`

### Step 2: Create GitHub Pages Branch

```bash
# Create and switch to gh-pages branch
git checkout --orphan gh-pages
git rm -rf .
git commit --allow-empty -m "Initial GitHub Pages commit"
git push origin gh-pages

# Switch back to main branch
git checkout master
```

### Step 3: Set Up Repository Structure

Your GitHub Pages will serve files from the `gh-pages` branch:

```
gh-pages/
├── index.html              # Repository homepage
├── apt/                    # APT repository files
│   ├── dists/
│   │   └── stable/
│   │       ├── Release
│   │       ├── Release.gpg
│   │       └── main/
│   │           └── binary-*/
│   └── pool/
│       └── main/
│           └── *.deb files
├── gpg-key.asc            # Public GPG key
└── install.sh             # One-liner installer script
```

## 🔧 Automated GitHub Pages Deployment

### Method 1: Using Our Enhanced GitHub Actions

Your existing `.github/workflows/release.yml` can be updated to deploy to GitHub Pages:

```yaml
# Add this job to your existing workflow
deploy-to-pages:
  needs: [build-and-release]
  runs-on: ubuntu-latest
  permissions:
    contents: read
    pages: write
    id-token: write
  environment:
    name: github-pages
    url: ${{ steps.deployment.outputs.page_url }}
  steps:
    - name: Download repository artifacts
      uses: actions/download-artifact@v3
      with:
        name: apt-repository
        path: ./gh-pages-content
    
    - name: Setup Pages
      uses: actions/configure-pages@v3
    
    - name: Upload Pages artifact
      uses: actions/upload-pages-artifact@v2
      with:
        path: ./gh-pages-content
    
    - name: Deploy to GitHub Pages
      id: deployment
      uses: actions/deploy-pages@v2
```

### Method 2: Manual Deployment Script

Use our `deploy-to-github-pages.sh` script (created below) for manual deployment.

## 🌐 Repository URLs

Once deployed, your repository will be accessible at:

```
Repository URL: https://YOUR_USERNAME.github.io/m-tunnel-rust/apt
GPG Key URL:    https://YOUR_USERNAME.github.io/m-tunnel-rust/gpg-key.asc
Install Script: https://YOUR_USERNAME.github.io/m-tunnel-rust/install.sh
```

## 📦 User Installation

Users can add your repository with:

```bash
# One-liner installation
curl -fsSL https://YOUR_USERNAME.github.io/m-tunnel-rust/install.sh | sudo bash

# Or manual installation
curl -fsSL https://YOUR_USERNAME.github.io/m-tunnel-rust/gpg-key.asc | sudo apt-key add -
echo "deb https://YOUR_USERNAME.github.io/m-tunnel-rust/apt stable main" | sudo tee /etc/apt/sources.list.d/m-tunnel.list
sudo apt update
sudo apt install m-tunnel-rust
```

## 🔒 Security Considerations

### Custom Domain (Recommended)

1. **Purchase a domain** (e.g., `apt.yourdomain.com`)
2. **Configure DNS**:
   ```
   Type: CNAME
   Name: apt
   Value: YOUR_USERNAME.github.io
   ```
3. **Set custom domain in GitHub Pages settings**
4. **Enable HTTPS** (automatic with custom domain)

### GPG Key Management

- **Never commit private keys** to the repository
- Store GPG private key in GitHub Secrets
- Use environment variables in GitHub Actions
- Regularly rotate signing keys

## 📊 Repository Statistics

GitHub Pages provides basic analytics, or you can add:

```html
<!-- Add to index.html for Google Analytics -->
<script async src="https://www.googletagmanager.com/gtag/js?id=GA_MEASUREMENT_ID"></script>
<script>
  window.dataLayer = window.dataLayer || [];
  function gtag(){dataLayer.push(arguments);}
  gtag('js', new Date());
  gtag('config', 'GA_MEASUREMENT_ID');
</script>
```

## 🚀 Best Practices

### 1. Repository Maintenance
- **Regular updates**: Use GitHub Actions for automatic package updates
- **Version management**: Tag releases properly
- **Clean old packages**: Remove outdated versions periodically

### 2. Documentation
- **Clear README**: Installation instructions and features
- **Changelog**: Document version changes
- **Issue templates**: Help users report problems

### 3. Monitoring
- **Check repository health**: Verify packages install correctly
- **Monitor usage**: Track download statistics
- **User feedback**: Respond to issues promptly

## 🔧 Troubleshooting

### Common Issues

1. **404 errors**: Check GitHub Pages is enabled and branch exists
2. **GPG verification fails**: Ensure public key is accessible
3. **Package not found**: Verify repository metadata is correct
4. **HTTPS issues**: Use custom domain for better SSL support

### Debug Commands

```bash
# Test repository accessibility
curl -I https://YOUR_USERNAME.github.io/m-tunnel-rust/apt/dists/stable/Release

# Verify GPG key
curl https://YOUR_USERNAME.github.io/m-tunnel-rust/gpg-key.asc | gpg --import

# Test package installation
apt-cache policy m-tunnel-rust
```

## 💡 Advanced Features

### Multiple Distributions
Support different Ubuntu/Debian versions:
```
dists/
├── focal/     # Ubuntu 20.04
├── jammy/     # Ubuntu 22.04
├── bullseye/  # Debian 11
└── bookworm/  # Debian 12
```

### Beta/Testing Channels
```
dists/
├── stable/    # Stable releases
├── testing/   # Beta versions
└── nightly/   # Development builds
```

## 📈 Success Metrics

Track your repository success:
- **Download counts**: Monitor package downloads
- **GitHub stars**: Repository popularity
- **Issue resolution**: Community engagement
- **Update frequency**: Regular maintenance

## 🎯 Next Steps

1. Run `./deploy-to-github-pages.sh` to deploy your repository
2. Test installation on a clean system
3. Share your repository URL with users
4. Monitor and maintain your packages

Your APT repository will be live at `https://YOUR_USERNAME.github.io/m-tunnel-rust/` 🎉