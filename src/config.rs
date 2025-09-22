use anyhow::{anyhow, Context, Result};
use serde::{Deserialize, Serialize};
use std::{fs, path::PathBuf};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub gate: SshConfig,
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
    pub server_name: Option<String>, // Display name for the server
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
        // Load from TOML configuration
        Self::load_toml()
    }

    fn load_toml() -> Result<Self> {
        let config_paths = ["/etc/m-tunnel/config.toml", "./config.toml"];

        // Check if any config file exists
        let mut config_exists = false;
        for path in &config_paths {
            if PathBuf::from(path).exists() {
                config_exists = true;
                break;
            }
        }

        // If no config exists, create a sample config.toml
        if !config_exists {
            let sample_config = r#"# M-Tunnel Configuration (TOML Format)
# This is the new structured configuration format
# Please edit the values below to match your SSH server and tunnel requirements

[gate]
host = "your-ssh-server.com"
user = "your-username"
port = 22
key_path = "./m-tunnel.key"
timeout = 30
keepalive_interval = 60

[limits]
max_attempts = 5
retry_window_secs = 300
max_backoff_secs = 60

# Tunnel configurations - add your tunnels here
# Example: Forward local port 8080 to remote port 80
[[tunnels]]
name = "web-tunnel"
direction = "send"  # "send" for local→remote, "receive" for remote→local
local_host = "127.0.0.1"
local_port = 8080
remote_host = "127.0.0.1"
remote_port = 80
enabled = false  # Set to true when configured

# Example: Reverse tunnel from remote port 2222 to local port 22
[[tunnels]]
name = "ssh-reverse"
direction = "receive"
local_host = "127.0.0.1"
local_port = 22
remote_host = "0.0.0.0"
remote_port = 2222
enabled = false  # Set to true when configured
"#;

            fs::write("./config.toml", sample_config)
                .context("Failed to create sample config.toml file")?;

            return Err(anyhow!("Created sample config.toml file. Please edit it with your SSH server details and tunnel configurations, then run again."));
        }

        // Try loading existing config
        for path in &config_paths {
            if let Ok(content) = fs::read_to_string(path) {
                return toml::from_str(&content).context("Failed to parse TOML configuration");
            }
        }

        Err(anyhow!("No TOML config found"))
    }
}
