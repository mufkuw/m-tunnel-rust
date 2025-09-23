use anyhow::{Context, Result};
use log::{error, info, warn};
use std::{
    collections::HashMap,
    net::IpAddr,
    process::Stdio,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc, Mutex,
    },
    time::{Duration, Instant},
};
use tokio::{process::Command, time};

use crate::config::{Config, TunnelConfig};
use crate::metrics::{MetricsCollector, TunnelStatus};

/// Get display name for server (use configured name or hide internal IPs)
fn get_server_display_name(ip_or_host: &str, server_name: &Option<String>) -> String {
    if is_server_internal_ip(ip_or_host) {
        server_name
            .as_ref()
            .unwrap_or(&"server_internal".to_string())
            .clone()
    } else {
        ip_or_host.to_string()
    }
}

/// Check if IP is a server internal network (hide completely)
fn is_server_internal_ip(ip_or_host: &str) -> bool {
    if let Ok(ip) = ip_or_host.parse::<IpAddr>() {
        match ip {
            IpAddr::V4(ipv4) => {
                let octets = ipv4.octets();
                // Server internal networks: 192.168.x.x, 10.x.x.x, 172.16-31.x.x
                (octets[0] == 192 && octets[1] == 168)
                    || (octets[0] == 10)
                    || (octets[0] == 172 && octets[1] >= 16 && octets[1] <= 31)
            }
            IpAddr::V6(_) => false, // Assume IPv6 is not server internal for now
        }
    } else {
        false
    }
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TunnelDirection {
    Send,    // Local push (SSH -R) - push local service to remote server
    Receive, // Remote pull (SSH -L) - pull remote service to local
}

impl From<&str> for TunnelDirection {
    fn from(s: &str) -> Self {
        match s {
            "send" => TunnelDirection::Send,
            "receive" => TunnelDirection::Receive,
            _ => panic!("Invalid tunnel direction: {}", s),
        }
    }
}

#[derive(Debug, Clone)]
pub struct Tunnel {
    pub id: String,
    pub direction: TunnelDirection,
    pub local_host: String,
    pub local_port: u16,
    pub remote_host: String,
    pub remote_port: u16,
}

impl From<&TunnelConfig> for Tunnel {
    fn from(config: &TunnelConfig) -> Self {
        Self {
            id: config.name.clone(),
            direction: TunnelDirection::from(config.direction.as_str()),
            local_host: config.local_host.clone(),
            local_port: config.local_port,
            remote_host: config.remote_host.clone(),
            remote_port: config.remote_port,
        }
    }
}

#[derive(Debug)]
struct TunnelMetrics {
    reconnect_count: u64,
    last_error: Option<String>,
}

#[derive(Debug)]
struct ConnectionLimiter {
    attempts: HashMap<String, (u32, Instant)>,
    max_attempts: u32,
    window: Duration,
}

impl ConnectionLimiter {
    fn new(max_attempts: u32, window: Duration) -> Self {
        Self {
            attempts: HashMap::new(),
            max_attempts,
            window,
        }
    }

    fn can_attempt(&mut self, host: &str) -> bool {
        let now = Instant::now();
        let key = host.to_string();

        // Clean old entries
        self.attempts
            .retain(|_, (_, time)| now.duration_since(*time) < self.window);

        // Check current attempts
        match self.attempts.get_mut(&key) {
            Some((count, time)) => {
                if now.duration_since(*time) >= self.window {
                    *count = 1;
                    *time = now;
                    true
                } else if *count >= self.max_attempts {
                    false
                } else {
                    *count += 1;
                    true
                }
            }
            None => {
                self.attempts.insert(key, (1, now));
                true
            }
        }
    }
}

pub struct TunnelManager {
    config: Config,
    metrics: Arc<MetricsCollector>,
    connection_limiter: Arc<Mutex<ConnectionLimiter>>,
    pub shutdown: Arc<AtomicBool>,
}

impl TunnelManager {
    pub async fn new(config: Config, metrics: Arc<MetricsCollector>) -> Result<Self> {
        info!("Initializing tunnel manager");

        let connection_limiter = Arc::new(Mutex::new(ConnectionLimiter::new(
            config.limits.max_attempts,
            Duration::from_secs(config.limits.retry_window_secs),
        )));

        Ok(Self {
            config,
            metrics,
            connection_limiter,
            shutdown: Arc::new(AtomicBool::new(false)),
        })
    }

    pub async fn start(&self) -> Result<()> {
        info!(
            "Starting tunnel manager: {} configured tunnels",
            self.config.tunnels.len()
        );

        let mut handles = vec![];

        // Start status monitoring task
        let status_metrics = Arc::clone(&self.metrics);
        let status_config = self.config.clone();
        let status_shutdown = Arc::clone(&self.shutdown);
        handles.push(tokio::spawn(async move {
            Self::monitor_tunnel_status(status_metrics, status_config, status_shutdown).await;
        }));

        for tunnel_config in &self.config.tunnels {
            if !tunnel_config.enabled {
                info!("Skipping disabled tunnel: {}", tunnel_config.name);
                continue;
            }

            let tunnel = Tunnel::from(tunnel_config);
            let ssh_config = self.config.gate.clone();
            let metrics = Arc::clone(&self.metrics);
            let limiter = Arc::clone(&self.connection_limiter);
            let shutdown = Arc::clone(&self.shutdown);

            handles.push(tokio::spawn(async move {
                Self::manage_ssh_cli_tunnel(tunnel, ssh_config, metrics, limiter, shutdown).await;
            }));
        }

        // Wait for shutdown signal
        while !self.shutdown.load(Ordering::Relaxed) {
            time::sleep(Duration::from_secs(1)).await;
        }

        // Cancel all tunnel tasks
        for handle in handles {
            handle.abort();
        }

        Ok(())
    }

    pub async fn shutdown(&self) -> Result<()> {
        info!("Initiating graceful shutdown...");
        self.shutdown.store(true, Ordering::Relaxed);

        // Give tunnels time to clean up
        time::sleep(Duration::from_secs(2)).await;

        Ok(())
    }

    async fn monitor_tunnel_status(
        metrics: Arc<MetricsCollector>,
        config: Config,
        shutdown: Arc<AtomicBool>,
    ) {
        info!("Starting tunnel status monitoring");

        let mut interval = tokio::time::interval(Duration::from_secs(60)); // Check every minute
        interval.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);

        while !shutdown.load(Ordering::Relaxed) {
            interval.tick().await;

            let mut status_report = String::new();
            let mut active_count = 0;
            let mut total_count = 0;

            for tunnel_config in &config.tunnels {
                total_count += 1;

                // Check if tunnel is actively running
                let stats_map = metrics.get_summary();
                let stats = stats_map.get(&tunnel_config.name);
                let is_connected = stats
                    .as_ref()
                    .map(|s| s.status == TunnelStatus::Connected)
                    .unwrap_or(false);
                let attempts = stats.as_ref().map(|s| s.reconnect_count).unwrap_or(0);

                if is_connected {
                    active_count += 1;
                    status_report.push_str(&format!(
                        "  ✓ {} → {}:{} (Active)\n",
                        tunnel_config.name, config.gate.host, tunnel_config.remote_port
                    ));
                } else if attempts > 0 {
                    status_report.push_str(&format!(
                        "  ⚠ {} → {}:{} (Reconnecting, {} attempts)\n",
                        tunnel_config.name, config.gate.host, tunnel_config.remote_port, attempts
                    ));
                } else {
                    status_report.push_str(&format!(
                        "  ✗ {} → {}:{} (Inactive)\n",
                        tunnel_config.name, config.gate.host, tunnel_config.remote_port
                    ));
                }
            }

            info!(
                "Tunnel Status Report ({}/{} active):\n{}",
                active_count, total_count, status_report
            );
        }

        info!("Tunnel status monitoring stopped");
    }

    async fn manage_ssh_cli_tunnel(
        tunnel: Tunnel,
        ssh_config: crate::config::SshConfig,
        metrics: Arc<MetricsCollector>,
        connection_limiter: Arc<Mutex<ConnectionLimiter>>,
        shutdown: Arc<AtomicBool>,
    ) {
        let mut delay = Duration::from_secs(1);
        let mut tunnel_metrics = TunnelMetrics {
            reconnect_count: 0,
            last_error: None,
        };

        let server_display = get_server_display_name(&ssh_config.host, &ssh_config.server_name);

        info!(
            "Tunnel '{}' -> {} (Direction: {}) - Initializing connection",
            tunnel.id,
            server_display,
            if tunnel.direction == TunnelDirection::Send {
                "LocalPush"
            } else {
                "RemotePull"
            }
        );

        metrics.update_tunnel_status(&tunnel.id, TunnelStatus::Connecting);

        loop {
            if shutdown.load(Ordering::Relaxed) {
                info!(
                    "Tunnel '{}' -> {} - Shutting down",
                    tunnel.id, server_display
                );
                metrics.update_tunnel_status(&tunnel.id, TunnelStatus::Disconnected);
                break;
            }

            // Check connection rate limiting
            let can_attempt = {
                let mut limiter = connection_limiter.lock().unwrap();
                limiter.can_attempt(&ssh_config.host)
            };

            if !can_attempt {
                warn!(
                    "Tunnel '{}' -> {} - Rate limit exceeded, waiting to retry...",
                    tunnel.id, server_display
                );
                metrics.update_tunnel_status(&tunnel.id, TunnelStatus::Error);
                time::sleep(Duration::from_secs(60)).await;
                continue;
            }

            // Log establishment attempt with clear status
            tunnel_metrics.reconnect_count += 1;
            if tunnel_metrics.reconnect_count == 1 {
                info!(
                    "Tunnel '{}' -> {} - Establishing connection...",
                    tunnel.id, server_display
                );
            } else {
                warn!(
                    "Tunnel '{}' -> {} - Reconnecting (attempt #{}) after disconnection",
                    tunnel.id, server_display, tunnel_metrics.reconnect_count
                );
            }

            metrics.update_tunnel_status(&tunnel.id, TunnelStatus::Connecting);

            // Log direction-specific details
            let server_display =
                get_server_display_name(&tunnel.remote_host, &ssh_config.server_name);
            match tunnel.direction {
                TunnelDirection::Receive => {
                    // Remote pull: SSH -L (pull remote service to local)
                    info!(
                        "Remote Pull: Listening {}:{} on {}:{}",
                        server_display, tunnel.remote_port, tunnel.local_host, tunnel.local_port
                    );
                }
                TunnelDirection::Send => {
                    // Local push: SSH -R (push local service to remote server)
                    info!(
                        "Local Push: Sending {}:{} to {}:{}",
                        tunnel.local_host, tunnel.local_port, server_display, tunnel.remote_port
                    );
                }
            }

            match Self::run_ssh_cli_tunnel(&tunnel, &ssh_config, &metrics, &shutdown).await {
                Ok(_) => {
                    tunnel_metrics.last_error = None;
                    delay = Duration::from_secs(1);
                    warn!(
                        "Tunnel '{}' -> {} - Connection terminated normally, preparing to reconnect...",
                        tunnel.id, server_display
                    );
                    metrics.update_tunnel_status(&tunnel.id, TunnelStatus::Disconnected);
                }
                Err(e) => {
                    tunnel_metrics.last_error = Some(e.to_string());
                    error!(
                        "Tunnel '{}' -> {} - Connection failed: {}",
                        tunnel.id, server_display, e
                    );
                    metrics.update_tunnel_status(&tunnel.id, TunnelStatus::Error);

                    // Show retry information
                    info!(
                        "Tunnel '{}' -> {} - Will retry in {} seconds...",
                        tunnel.id,
                        server_display,
                        delay.as_secs()
                    );
                }
            }

            if !shutdown.load(Ordering::Relaxed) {
                warn!("Reconnecting tunnel {} in {}s", tunnel.id, delay.as_secs());
                metrics.update_tunnel_status(&tunnel.id, TunnelStatus::Connecting);
                time::sleep(delay).await;
                delay = std::cmp::min(delay * 2, Duration::from_secs(60));
            }
        }

        metrics.update_tunnel_status(&tunnel.id, TunnelStatus::Disconnected);
    }

    async fn run_ssh_cli_tunnel(
        tunnel: &Tunnel,
        ssh_config: &crate::config::SshConfig,
        metrics: &Arc<MetricsCollector>,
        shutdown: &Arc<AtomicBool>,
    ) -> Result<()> {
        let server_display = get_server_display_name(&ssh_config.host, &ssh_config.server_name);

        metrics.update_tunnel_status(&tunnel.id, TunnelStatus::Connected);

        // Log successful connection establishment
        info!(
            "Tunnel '{}' -> {} - Connection established successfully ✓",
            tunnel.id, server_display
        );

        let mut ssh_args = vec![
            "-N".to_string(), // Don't execute remote command
            "-o".to_string(),
            "StrictHostKeyChecking=no".to_string(),
            "-o".to_string(),
            "UserKnownHostsFile=/dev/null".to_string(),
            "-o".to_string(),
            "LogLevel=ERROR".to_string(),
            "-o".to_string(),
            "ServerAliveInterval=30".to_string(), // Keep alive
            "-o".to_string(),
            "ServerAliveCountMax=3".to_string(),
            "-p".to_string(),
            ssh_config.port.to_string(),
            "-i".to_string(),
            ssh_config.key_path.to_string_lossy().to_string(),
        ];

        // Add tunnel-specific arguments
        match tunnel.direction {
            TunnelDirection::Receive => {
                // Remote pull: SSH -L (pull remote service to local)
                ssh_args.push("-L".to_string());
                ssh_args.push(format!(
                    "{}:{}:{}",
                    tunnel.local_port, tunnel.remote_host, tunnel.remote_port
                ));
            }
            TunnelDirection::Send => {
                // Local push: SSH -R (push local service to remote server)
                ssh_args.push("-R".to_string());
                ssh_args.push(format!(
                    "{}:{}:{}",
                    tunnel.remote_port, tunnel.local_host, tunnel.local_port
                ));
            }
        }

        ssh_args.push(format!("{}@{}", ssh_config.user, ssh_config.host));

        let mut ssh_process = Command::new("ssh")
            .args(&ssh_args)
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::piped())
            .spawn()
            .context("Failed to start tunnel process")?;

        // Wait for shutdown or process exit
        loop {
            if shutdown.load(Ordering::Relaxed) {
                info!(
                    "Tunnel '{}' -> {} - Shutdown signal received, terminating process",
                    tunnel.id, server_display
                );
                let _ = ssh_process.kill().await;
                break;
            }

            // Check if process is still running
            match ssh_process.try_wait() {
                Ok(Some(status)) => {
                    warn!(
                        "Tunnel '{}' -> {} - Process terminated (status: {}), connection lost",
                        tunnel.id, server_display, status
                    );
                    return Err(anyhow::anyhow!(
                        "Connection process exited with status: {}",
                        status
                    ));
                }
                Ok(None) => {
                    // Process still running, continue monitoring
                    time::sleep(Duration::from_millis(500)).await;
                }
                Err(e) => {
                    error!(
                        "Tunnel '{}' -> {} - Failed to monitor process: {}",
                        tunnel.id, server_display, e
                    );
                    return Err(anyhow::anyhow!(
                        "Failed to check tunnel process status: {}",
                        e
                    ));
                }
            }
        }

        Ok(())
    }
}
