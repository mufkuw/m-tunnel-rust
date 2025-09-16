use anyhow::{anyhow, Context, Result};
use serde::{Deserialize, Serialize};
use std::{env, fs, path::PathBuf};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub ssh: SshConfig,
    pub tunnels: Vec<TunnelConfig>,
    pub limits: ConnectionLimits,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SshConfig {
    pub host: String,
    pub user: String,
    pub port: u16,
    pub key_path: PathBuf,
    pub timeout: u64,
    pub keepalive_interval: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TunnelConfig {
    pub name: String,
    pub direction: String,
    pub local_host: String,
    pub local_port: u16,
    pub remote_host: String,
    pub remote_port: u16,
    pub enabled: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConnectionLimits {
    pub max_attempts: u32,
    pub retry_window_secs: u64,
    pub max_backoff_secs: u64,
}

impl Default for ConnectionLimits {
    fn default() -> Self {
        Self {
            max_attempts: 5,
            retry_window_secs: 300,
            max_backoff_secs: 60,
        }
    }
}

impl Config {
    pub fn load() -> Result<Self> {
        // Try loading from TOML first (more structured)
        if let Ok(config) = Self::load_toml() {
            return Ok(config);
        }

        // Fall back to legacy format
        Self::load_legacy()
    }

    fn load_toml() -> Result<Self> {
        let config_paths = ["/etc/m-tunnel/config.toml", "./config.toml"];

        for path in &config_paths {
            if let Ok(content) = fs::read_to_string(path) {
                return toml::from_str(&content).context("Failed to parse TOML configuration");
            }
        }

        Err(anyhow!("No TOML config found"))
    }

    fn load_legacy() -> Result<Self> {
        // Load from .env and m-tunnel.conf (your current format)
        use crate::security::SecureKeyManager;

        // Load environment variables
        let env_path = if PathBuf::from("/etc/m-tunnel/.env").exists() {
            "/etc/m-tunnel/.env"
        } else {
            ".env"
        };

        dotenvy::from_path(env_path).context("Failed to load .env file")?;

        let ssh_host = env::var("SSH_HOST").context("SSH_HOST not set")?;
        let ssh_user = env::var("SSH_USER").context("SSH_USER not set")?;
        let ssh_port = env::var("SSH_PORT")
            .context("SSH_PORT not set")?
            .parse::<u16>()
            .context("Invalid SSH_PORT")?;
        let ssh_key_raw = env::var("SSH_PRIVATE_KEY").context("SSH_PRIVATE_KEY not set")?;

        // Resolve SSH key path
        let ssh_key_path = PathBuf::from(&ssh_key_raw);
        let ssh_key = if ssh_key_path.is_absolute() {
            ssh_key_path
        } else {
            let system_key_path = PathBuf::from("/etc/m-tunnel").join(&ssh_key_path);
            if system_key_path.exists() {
                system_key_path
            } else {
                let local_path = env::current_dir()
                    .context("Failed to get current directory")?
                    .join(&ssh_key_path);

                if !local_path.exists() {
                    return Err(anyhow!(
                        "SSH key file not found. Looked in:\n  - {}\n  - {}",
                        system_key_path.to_string_lossy(),
                        local_path.to_string_lossy()
                    ));
                }
                local_path
            }
        };

        let ssh_key = ssh_key
            .canonicalize()
            .context("Failed to canonicalize SSH key path")?;

        // Validate SSH key security
        SecureKeyManager::validate_key_security(&ssh_key)
            .context("SSH key security validation failed")?;

        // Parse tunnel configuration
        let config_path = if PathBuf::from("/etc/m-tunnel/m-tunnel.conf").exists() {
            PathBuf::from("/etc/m-tunnel/m-tunnel.conf")
        } else {
            PathBuf::from("m-tunnel.conf")
        };

        if !config_path.exists() {
            return Err(anyhow!(
                "Missing m-tunnel.conf. Expected locations:\n  - /etc/m-tunnel/m-tunnel.conf (system install)\n  - ./m-tunnel.conf (development)"
            ));
        }

        let tunnels = Self::parse_legacy_tunnels(&config_path)?;

        Ok(Config {
            ssh: SshConfig {
                host: ssh_host,
                user: ssh_user,
                port: ssh_port,
                key_path: ssh_key,
                timeout: 30,
                keepalive_interval: 60,
            },
            tunnels,
            limits: ConnectionLimits::default(),
        })
    }

    pub fn parse_legacy_tunnels(path: &PathBuf) -> Result<Vec<TunnelConfig>> {
        let content = fs::read_to_string(path)?;
        let mut tunnels = Vec::new();

        for (i, line) in content.lines().enumerate() {
            let line = line.trim();
            if line.is_empty() || line.starts_with('#') {
                continue;
            }

            let (direction, rest) = if let Some(r) = line.strip_prefix("send -- ") {
                ("send", r)
            } else if let Some(r) = line.strip_prefix("receive -- ") {
                ("receive", r)
            } else {
                return Err(anyhow!(
                    "Line {}: must start with 'send --' or 'receive --'",
                    i + 1
                ));
            };

            let parts: Vec<_> = match direction {
                "receive" => rest.split(" from ").collect(),
                "send" => rest.split(" to ").collect(),
                _ => unreachable!(),
            };

            if parts.len() != 2 {
                return Err(anyhow!(
                    "Line {}: invalid format. Expected '{}' keyword for {}",
                    i + 1,
                    if direction == "receive" { "from" } else { "to" },
                    direction
                ));
            }

            let (local_host, local_port, remote_host, remote_port) = match direction {
                "send" => {
                    let (lh, lp) = Self::parse_host_port(parts[0].trim())?;
                    let (rh, rp) = Self::parse_host_port(parts[1].trim())?;
                    (lh, lp, rh, rp)
                }
                "receive" => {
                    let (rh, rp) = Self::parse_host_port(parts[0].trim())?;
                    let (lh, lp) = Self::parse_host_port(parts[1].trim())?;
                    (lh, lp, rh, rp)
                }
                _ => unreachable!(),
            };

            tunnels.push(TunnelConfig {
                name: format!("tunnel_{}", i + 1),
                direction: direction.to_string(),
                local_host,
                local_port,
                remote_host,
                remote_port,
                enabled: true,
            });
        }

        Ok(tunnels)
    }

    pub fn parse_host_port(s: &str) -> Result<(String, u16)> {
        let mut parts = s.split(':');
        let host = parts
            .next()
            .ok_or_else(|| anyhow!("Missing host"))?
            .to_string();
        let port = parts
            .next()
            .ok_or_else(|| anyhow!("Missing port"))?
            .parse::<u16>()
            .context("Invalid port")?;
        Ok((host, port))
    }
}
