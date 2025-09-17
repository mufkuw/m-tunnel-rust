#Requires -Version 5.1
#Requires -RunAsAdministrator

# M-Tunnel Rust Installer for Windows v2.0 (SSH2 Enhanced)
# ========================================================

param(
    [switch]$Uninstall
)

# Check if running as administrator
$currentUser = [Security.Principal.WindowsIdentity]::GetCurrent()
$principal = New-Object Security.Principal.WindowsPrincipal($currentUser)
if (-not $principal.IsInRole([Security.Principal.WindowsBuiltInRole]::Administrator)) {
    Write-Error "This script must be run as Administrator. Please run PowerShell as Administrator and try again."
    exit 1
}

Write-Host "🚀 M-Tunnel Rust Installer v2.0 (SSH2 Enhanced) for Windows" -ForegroundColor Green
Write-Host "==========================================================" -ForegroundColor Green
Write-Host ""

if ($Uninstall) {
    Write-Host "🗑️  Uninstalling M-Tunnel..." -ForegroundColor Yellow
    # TODO: Implement uninstall logic
    exit 0
}

# Step 2: Extract metadata from Cargo.toml first
Write-Host "📋 Extracting metadata from Cargo.toml..." -ForegroundColor Cyan
$Name = Get-Content Cargo.toml | Where-Object { $_ -match '^name\s*=\s*"([^"]+)"' } | ForEach-Object { $matches[1] } | Select-Object -First 1
$Version = Get-Content Cargo.toml | Where-Object { $_ -match '^version\s*=\s*"([^"]+)"' } | ForEach-Object { $matches[1] } | Select-Object -First 1
$Author = Get-Content Cargo.toml | Where-Object { $_ -match '^authors\s*=\s*\["([^"]+)"' } | ForEach-Object { $matches[1] } | Select-Object -First 1
$Description = Get-Content Cargo.toml | Where-Object { $_ -match '^description\s*=\s*"([^"]+)"' } | ForEach-Object { $matches[1] } | Select-Object -First 1

if (-not $Description) {
    $Description = "SSH tunnel management utility with native SSH2 library support"
}

Write-Host "Package: $Name"
Write-Host "Version: $Version"
Write-Host "Author: $Author"
Write-Host "Description: $Description"
Write-Host ""

# Step 1: Build the Rust project (skip if already built)
if (-not (Test-Path "target\release\$Name.exe")) {
    Write-Host "📦 Building the Rust project..." -ForegroundColor Cyan
    & cargo build --release
    if ($LASTEXITCODE -ne 0) {
        Write-Error "Build failed"
        exit 1
    }
}
else {
    Write-Host "📦 Binary already built, skipping build..." -ForegroundColor Cyan
}

# Step 3: Define installation paths
$InstallDir = "$env:ProgramFiles\m-tunnel"
$DataDir = "$env:ProgramData\m-tunnel"
$LogDir = "$env:ProgramData\m-tunnel\logs"
$TempDir = "$env:TEMP\ssh-m-tunnel"

Write-Host "🏗️  Preparing installation directories..." -ForegroundColor Cyan
New-Item -ItemType Directory -Path $InstallDir -Force | Out-Null
New-Item -ItemType Directory -Path $DataDir -Force | Out-Null
New-Item -ItemType Directory -Path "$DataDir\examples" -Force | Out-Null
New-Item -ItemType Directory -Path "$InstallDir\docs" -Force | Out-Null
New-Item -ItemType Directory -Path "$InstallDir\tests" -Force | Out-Null
New-Item -ItemType Directory -Path $LogDir -Force | Out-Null
New-Item -ItemType Directory -Path $TempDir -Force | Out-Null

# Step 4: Copy binary
Write-Host "📦 Copying binary..." -ForegroundColor Cyan
Copy-Item "target\release\$Name.exe" "$InstallDir\" -Force

# Step 5: Copy configuration files
Write-Host "⚙️  Copying configuration files..." -ForegroundColor Cyan
if (Test-Path "configs\m-tunnel.conf") {
    Copy-Item "configs\m-tunnel.conf" "$DataDir\" -Force
}
else {
    Write-Warning "configs\m-tunnel.conf not found"
}

if (Test-Path ".env") {
    Copy-Item ".env" "$DataDir\" -Force
}
else {
    Write-Warning ".env not found"
}

if (Test-Path "configs\m-tunnel.key") {
    Copy-Item "configs\m-tunnel.key" "$DataDir\" -Force
}
else {
    Write-Warning "configs\m-tunnel.key not found"
}

# Copy example configurations
if (Test-Path "configs\m-tunnel.key.example") {
    Copy-Item "configs\m-tunnel.key.example" "$DataDir\examples\" -Force
}
else {
    Write-Warning "SSH key example not found"
}

if (Test-Path "configs\known_hosts.template") {
    Copy-Item "configs\known_hosts.template" "$DataDir\examples\" -Force
}
else {
    Write-Warning "known_hosts template not found"
}

if (Test-Path "configs\real_ssh_test.toml") {
    Copy-Item "configs\real_ssh_test.toml" "$DataDir\examples\" -Force
}
else {
    Write-Warning "TOML config example not found"
}

if (Test-Path "config.toml.example") {
    Copy-Item "config.toml.example" "$DataDir\examples\" -Force
}
else {
    Write-Warning "TOML config example not found"
}

# Step 6: Copy documentation
Write-Host "📚 Copying documentation..." -ForegroundColor Cyan
if (Test-Path "README.md") {
    Copy-Item "README.md" "$InstallDir\docs\" -Force
}
else {
    Write-Warning "README.md not found"
}

Get-ChildItem "docs\*.md" -ErrorAction SilentlyContinue | ForEach-Object {
    Copy-Item $_.FullName "$InstallDir\docs\" -Force
}

# Step 7: Copy test scripts
Write-Host "🧪 Copying test scripts..." -ForegroundColor Cyan
Get-ChildItem "tests\*.sh" -ErrorAction SilentlyContinue | ForEach-Object {
    Copy-Item $_.FullName "$InstallDir\tests\" -Force
}

Get-ChildItem "tests\*.ps1" -ErrorAction SilentlyContinue | ForEach-Object {
    Copy-Item $_.FullName "$InstallDir\tests\" -Force
}

# Step 8: Set permissions (Windows equivalent)
Write-Host "🔒 Setting permissions..." -ForegroundColor Cyan
# For config files, set appropriate permissions
$acl = Get-Acl "$DataDir"
$acl.SetAccessRuleProtection($true, $false)
$adminRule = New-Object System.Security.AccessControl.FileSystemAccessRule("Administrators", "FullControl", "Allow")
$systemRule = New-Object System.Security.AccessControl.FileSystemAccessRule("SYSTEM", "FullControl", "Allow")
$acl.SetAccessRule($adminRule)
$acl.SetAccessRule($systemRule)
Set-Acl "$DataDir" $acl

# Step 9: Create Windows service
$ServiceName = "m-tunnel"
Write-Host "⚙️  Creating Windows service..." -ForegroundColor Cyan

# Stop and delete existing service if it exists
if (Get-Service $ServiceName -ErrorAction SilentlyContinue) {
    Stop-Service $ServiceName -ErrorAction SilentlyContinue
    sc.exe delete $ServiceName | Out-Null
}

# Create the service
$servicePath = "`"$InstallDir\$Name.exe`" --ssh2"
$createService = "sc.exe create $ServiceName binPath= $servicePath start= auto"
Invoke-Expression $createService

# Set service description
sc.exe description $ServiceName "$Description (SSH2 Enhanced)"

# Configure service to run as Local System (or specific user if needed)
# For security, you might want to create a dedicated user, but for simplicity:
sc.exe config $ServiceName obj= "LocalSystem"

# Step 10: Create known_hosts file
New-Item -ItemType File -Path "$DataDir\known_hosts" -Force | Out-Null

# Step 11: Add to PATH (optional)
$pathEnv = [Environment]::GetEnvironmentVariable("Path", "Machine")
if ($pathEnv -notlike "*$InstallDir*") {
    [Environment]::SetEnvironmentVariable("Path", "$pathEnv;$InstallDir", "Machine")
    Write-Host "✅ Added $InstallDir to system PATH" -ForegroundColor Green
}

Write-Host ""
Write-Host "🎉 M-Tunnel Installation Complete!" -ForegroundColor Green
Write-Host "==================================" -ForegroundColor Green
Write-Host ""
Write-Host "📁 Installation locations:" -ForegroundColor Yellow
Write-Host "   Binary:         $InstallDir\$Name.exe"
Write-Host "   Configuration:  $DataDir\"
Write-Host "   Examples:       $DataDir\examples\"
Write-Host "   Documentation:  $InstallDir\docs\"
Write-Host "   Test Scripts:   $InstallDir\tests\"
Write-Host "   Logs:           $LogDir\"
Write-Host ""
Write-Host "🚀 SSH2 Implementation Available!" -ForegroundColor Cyan
Write-Host "- Use '--ssh2' flag for native SSH2 library"
Write-Host "- TOML configuration support"
Write-Host "- Enhanced security and performance"
Write-Host ""
Write-Host "⚙️  Setup Instructions:" -ForegroundColor Yellow
Write-Host "1. Edit $DataDir\.env with your SSH settings (legacy)"
Write-Host "2. OR create $DataDir\config.toml (recommended)"
Write-Host "3. Edit tunnel definitions as needed"
Write-Host "4. Add SSH host key: ssh-keyscan -H your-ssh-host >> $DataDir\known_hosts"
Write-Host "5. Start the service: Start-Service $ServiceName"
Write-Host ""
Write-Host "🧪 Testing:" -ForegroundColor Yellow
Write-Host "- Quick test: & '$InstallDir\tests\test_quick.ps1' (if converted)"
Write-Host "- Or run the .sh scripts with bash if available"
Write-Host ""
Write-Host "🔒 Security: Service runs as Local System" -ForegroundColor Yellow
Write-Host "📊 Logs: Get-EventLog -LogName Application -Source $ServiceName (if configured)"
Write-Host "🆘 Help: & '$InstallDir\$Name.exe' --help"
Write-Host ""

# Optional: Start the service
$startService = Read-Host "Do you want to start the M-Tunnel service now? (y/n)"
if ($startService -eq 'y') {
    Start-Service $ServiceName
    Write-Host "✅ Service started" -ForegroundColor Green
}