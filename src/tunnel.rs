use anyhow::{Context, Result};
use log::{debug, error, info, warn};
use std::{
    collections::HashMap,
    process::{Command, Stdio},
    sync::{Arc, Mutex},
    time::{Duration, Instant},
};
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::process::Command as TokioCommand;
use tokio::time;

use crate::config::{Config, TunnelConfig};
use crate::metrics::{MetricsCollector, TunnelStatus};
use crate::security::SecureKeyManager;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TunnelDirection {
    Send,    // remote port forward
    Receive, // local port forward
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
    pub enabled: bool,
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
            enabled: config.enabled,
        }
    }
}

#[derive(Debug)]
pub struct TunnelMetrics {
    pub uptime_start: Instant,
    pub reconnect_count: u64,
    pub last_error: Option<String>,
}

#[derive(Debug)]
pub struct ConnectionLimiter {
    attempts: HashMap<String, (u32, Instant)>,
    max_attempts: u32,
    window: Duration,
}

impl ConnectionLimiter {
    pub fn new(max_attempts: u32, window: Duration) -> Self {
        Self {
            attempts: HashMap::new(),
            max_attempts,
            window,
        }
    }

    pub fn can_attempt(&mut self, host: &str) -> bool {
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
    shutdown: Arc<Mutex<bool>>,
}

impl TunnelManager {
    pub async fn new(config: Config, metrics: Arc<MetricsCollector>) -> Result<Self> {
        // Validate SSH configuration
        let (host, user) = SecureKeyManager::sanitize_ssh_args(&config.ssh.host, &config.ssh.user)
            .context("SSH configuration validation failed")?;

        let mut validated_config = config;
        validated_config.ssh.host = host;
        validated_config.ssh.user = user;

        let connection_limiter = Arc::new(Mutex::new(ConnectionLimiter::new(
            validated_config.limits.max_attempts,
            Duration::from_secs(validated_config.limits.retry_window_secs),
        )));

        Ok(Self {
            config: validated_config,
            metrics,
            connection_limiter,
            shutdown: Arc::new(Mutex::new(false)),
        })
    }

    pub async fn start(&self) -> Result<()> {
        info!("Starting {} tunnels", self.config.tunnels.len());

        let mut handles = vec![];

        for tunnel_config in &self.config.tunnels {
            if !tunnel_config.enabled {
                info!("Skipping disabled tunnel: {}", tunnel_config.name);
                continue;
            }

            let tunnel = Tunnel::from(tunnel_config);
            let ssh_config = self.config.ssh.clone();
            let metrics = Arc::clone(&self.metrics);
            let limiter = Arc::clone(&self.connection_limiter);
            let shutdown = Arc::clone(&self.shutdown);

            handles.push(tokio::spawn(async move {
                Self::manage_tunnel(tunnel, ssh_config, metrics, limiter, shutdown).await;
            }));
        }

        // Wait for shutdown signal
        while !*self.shutdown.lock().unwrap() {
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
        *self.shutdown.lock().unwrap() = true;
        
        // Give tunnels time to clean up
        time::sleep(Duration::from_secs(2)).await;
        
        Ok(())
    }

    async fn manage_tunnel(
        tunnel: Tunnel,
        ssh_config: crate::config::SshConfig,
        metrics: Arc<MetricsCollector>,
        connection_limiter: Arc<Mutex<ConnectionLimiter>>,
        shutdown: Arc<Mutex<bool>>,
    ) {
        let mut delay = Duration::from_secs(1);
        let mut tunnel_metrics = TunnelMetrics {
            uptime_start: Instant::now(),
            reconnect_count: 0,
            last_error: None,
        };

        metrics.update_tunnel_status(&tunnel.id, TunnelStatus::Connecting);

        loop {
            if *shutdown.lock().unwrap() {
                info!("Shutting down tunnel: {}", tunnel.id);
                break;
            }

            // Check connection rate limiting
            let can_attempt = {
                let mut limiter = connection_limiter.lock().unwrap();
                limiter.can_attempt(&ssh_config.host)
            };

            if !can_attempt {
                warn!(
                    "Rate limit exceeded for host {}, waiting...",
                    ssh_config.host
                );
                metrics.update_tunnel_status(&tunnel.id, TunnelStatus::Error);
                time::sleep(Duration::from_secs(60)).await;
                continue;
            }

            let mut command = Self::build_ssh_command(&tunnel, &ssh_config);

            info!(
                "Starting tunnel: {} {:?} {}:{} → {}:{} (attempt #{})",
                tunnel.id,
                tunnel.direction,
                tunnel.local_host,
                tunnel.local_port,
                tunnel.remote_host,
                tunnel.remote_port,
                tunnel_metrics.reconnect_count + 1
            );

            let tunnel_id = tunnel.id.clone(); // Clone before moving into spawn

            match command.spawn() {
                Ok(mut child) => {
                    metrics.update_tunnel_status(&tunnel.id, TunnelStatus::Connected);
                    
                    let stderr = child.stderr.take().expect("No stderr pipe");
                    let reader = BufReader::new(stderr);
                    let mut lines = reader.lines();

                    let logging_task = tokio::spawn(async move {
                        while let Ok(Some(line)) = lines.next_line().await {
                            if line.contains("ERROR") || line.contains("WARNING") {
                                error!("[ssh:{}] {}", tunnel_id, line);
                            } else {
                                debug!("[ssh:{}] {}", tunnel_id, line);
                            }
                        }
                    });

                    match child.wait().await {
                        Ok(status) => {
                            tunnel_metrics.reconnect_count += 1;
                            metrics.increment_reconnect(&tunnel.id);
                            
                            let uptime = tunnel_metrics.uptime_start.elapsed();
                            warn!(
                                "Tunnel {} exited with: {} (uptime: {:?}, reconnects: {})",
                                tunnel.id, status, uptime, tunnel_metrics.reconnect_count
                            );

                            if status.success() {
                                tunnel_metrics.last_error = None;
                                delay = Duration::from_secs(1);
                            } else {
                                tunnel_metrics.last_error = Some(format!("Exit code: {}", status));
                                metrics.update_tunnel_status(&tunnel.id, TunnelStatus::Error);
                            }
                        }
                        Err(e) => {
                            tunnel_metrics.reconnect_count += 1;
                            tunnel_metrics.last_error = Some(format!("Process error: {}", e));
                            error!("Failed to wait on tunnel process {}: {:?}", tunnel.id, e);
                            metrics.update_tunnel_status(&tunnel.id, TunnelStatus::Error);
                        }
                    }

                    let _ = logging_task.await;
                }
                Err(e) => {
                    tunnel_metrics.last_error = Some(format!("Spawn error: {}", e));
                    error!("Failed to spawn ssh tunnel {}: {:?}", tunnel.id, e);
                    metrics.update_tunnel_status(&tunnel.id, TunnelStatus::Error);
                }
            }

            if !*shutdown.lock().unwrap() {
                warn!("Restarting tunnel {} in {}s...", tunnel.id, delay.as_secs());
                metrics.update_tunnel_status(&tunnel.id, TunnelStatus::Connecting);
                time::sleep(delay).await;
                delay = std::cmp::min(delay * 2, Duration::from_secs(60));
            }
        }

        metrics.update_tunnel_status(&tunnel.id, TunnelStatus::Disconnected);
    }

    fn build_ssh_command(tunnel: &Tunnel, ssh_config: &crate::config::SshConfig) -> TokioCommand {
        let mut cmd = Command::new("ssh");
        cmd.arg("-i").arg(&ssh_config.key_path);
        cmd.arg("-p").arg(ssh_config.port.to_string());

        // Security: Enable strict host key checking
        cmd.arg("-o").arg("StrictHostKeyChecking=yes");
        cmd.arg("-o").arg("UserKnownHostsFile=/etc/m-tunnel/known_hosts");
        cmd.arg("-o").arg("HashKnownHosts=yes");

        // Performance: SSH multiplexing
        cmd.arg("-o").arg("ControlMaster=auto");
        cmd.arg("-o").arg("ControlPath=/tmp/ssh-m-tunnel-%r@%h:%p");
        cmd.arg("-o").arg("ControlPersist=60s");

        // Network optimizations
        cmd.arg("-o").arg("TCPKeepAlive=yes");
        cmd.arg("-o").arg(format!("ServerAliveInterval={}", ssh_config.keepalive_interval));
        cmd.arg("-o").arg("ServerAliveCountMax=3");
        cmd.arg("-o").arg("Compression=yes");

        // Timeouts
        cmd.arg("-o").arg(format!("ConnectTimeout={}", ssh_config.timeout));
        cmd.arg("-o").arg("LogLevel=ERROR");
        cmd.arg("-o").arg("ExitOnForwardFailure=yes");
        cmd.arg("-N");

        match tunnel.direction {
            TunnelDirection::Receive => {
                // Local port forward: -L [bind_address:]port:host:hostport
                let spec = format!(
                    "{}:{}:{}:{}",
                    tunnel.local_host, tunnel.local_port,
                    tunnel.remote_host, tunnel.remote_port
                );
                cmd.arg("-L").arg(spec);
            }
            TunnelDirection::Send => {
                // Remote port forward: -R [bind_address:]port:host:hostport
                let spec = format!(
                    "{}:{}:{}:{}",
                    tunnel.local_host, tunnel.local_port,
                    tunnel.remote_host, tunnel.remote_port
                );
                cmd.arg("-R").arg(spec);
            }
        }

        cmd.arg(format!("{}@{}", ssh_config.user, ssh_config.host));
        cmd.stdout(Stdio::null());
        cmd.stderr(Stdio::piped());

        TokioCommand::from(cmd)
    }
}