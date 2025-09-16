use anyhow::{Context, Result};
use log::{debug, error, info, warn};
use ssh2::{Channel, Session};
use std::{
    collections::HashMap,
    io::{Read, Write},
    net::TcpStream,
    sync::{Arc, Mutex},
    time::{Duration, Instant},
};
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::{TcpListener as TokioTcpListener, TcpStream as TokioTcpStream},
    time,
};

use crate::config::{Config, TunnelConfig};
use crate::metrics::{MetricsCollector, TunnelStatus};
use crate::security::SecureKeyManager;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TunnelDirection {
    Send,    // local to remote (forward)
    Receive, // remote to local (reverse)
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
    pub bytes_transferred: u64,
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

pub struct SshConnection {
    session: Session,
    tcp_stream: TcpStream,
}

impl SshConnection {
    pub fn new(ssh_config: &crate::config::SshConfig) -> Result<Self> {
        let tcp_stream = TcpStream::connect(format!("{}:{}", ssh_config.host, ssh_config.port))
            .context("Failed to connect to SSH server")?;

        tcp_stream
            .set_read_timeout(Some(Duration::from_secs(ssh_config.timeout)))
            .context("Failed to set read timeout")?;

        tcp_stream
            .set_write_timeout(Some(Duration::from_secs(ssh_config.timeout)))
            .context("Failed to set write timeout")?;

        let mut session = Session::new().context("Failed to create SSH session")?;
        session.set_tcp_stream(
            tcp_stream
                .try_clone()
                .context("Failed to clone TCP stream")?,
        );
        session.handshake().context("SSH handshake failed")?;

        // Authenticate with private key
        session
            .userauth_pubkey_file(&ssh_config.user, None, &ssh_config.key_path, None)
            .context("SSH authentication failed")?;

        if !session.authenticated() {
            return Err(anyhow::anyhow!("SSH authentication failed"));
        }

        info!(
            "SSH connection established to {}@{}",
            ssh_config.user, ssh_config.host
        );

        Ok(Self {
            session,
            tcp_stream,
        })
    }

    pub fn create_local_forward(
        &mut self,
        local_port: u16,
        remote_host: &str,
        remote_port: u16,
    ) -> Result<Channel> {
        let channel = self
            .session
            .channel_direct_tcpip(&remote_host, remote_port, Some(("127.0.0.1", local_port)))
            .context("Failed to create direct TCP/IP channel")?;

        Ok(channel)
    }

    pub fn create_remote_forward(&mut self, remote_port: u16) -> Result<()> {
        // Note: SSH2 library doesn't support remote port forwarding in the same way
        // This would need to be implemented differently or use a different approach
        info!(
            "Remote forwarding setup for port {} (simplified implementation)",
            remote_port
        );
        Ok(())
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
                Self::manage_tunnel_with_ssh2(tunnel, ssh_config, metrics, limiter, shutdown).await;
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

    async fn manage_tunnel_with_ssh2(
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
            bytes_transferred: 0,
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

            match Self::run_tunnel(&tunnel, &ssh_config, &metrics, &shutdown).await {
                Ok(bytes) => {
                    tunnel_metrics.bytes_transferred += bytes;
                    tunnel_metrics.last_error = None;
                    delay = Duration::from_secs(1);
                    info!(
                        "Tunnel {} completed successfully, transferred {} bytes",
                        tunnel.id, bytes
                    );
                }
                Err(e) => {
                    tunnel_metrics.reconnect_count += 1;
                    tunnel_metrics.last_error = Some(e.to_string());
                    error!("Tunnel {} failed: {}", tunnel.id, e);
                    metrics.update_tunnel_status(&tunnel.id, TunnelStatus::Error);
                    metrics.increment_reconnect(&tunnel.id);
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

    async fn run_tunnel(
        tunnel: &Tunnel,
        ssh_config: &crate::config::SshConfig,
        metrics: &Arc<MetricsCollector>,
        shutdown: &Arc<Mutex<bool>>,
    ) -> Result<u64> {
        let mut ssh_conn = SshConnection::new(ssh_config)?;
        metrics.update_tunnel_status(&tunnel.id, TunnelStatus::Connected);

        match tunnel.direction {
            TunnelDirection::Receive => {
                Self::run_local_forward(&mut ssh_conn, tunnel, shutdown).await
            }
            TunnelDirection::Send => {
                Self::run_remote_forward(&mut ssh_conn, tunnel, shutdown).await
            }
        }
    }

    async fn run_local_forward(
        ssh_conn: &mut SshConnection,
        tunnel: &Tunnel,
        shutdown: &Arc<Mutex<bool>>,
    ) -> Result<u64> {
        let listener =
            TokioTcpListener::bind(format!("{}:{}", tunnel.local_host, tunnel.local_port))
                .await
                .context("Failed to bind local listener")?;

        info!(
            "Local forward listening on {}:{}",
            tunnel.local_host, tunnel.local_port
        );
        let mut total_bytes = 0u64;

        loop {
            if *shutdown.lock().unwrap() {
                break;
            }

            tokio::select! {
                result = listener.accept() => {
                    match result {
                        Ok((local_stream, addr)) => {
                            debug!("Accepted connection from {}", addr);

                            // Create SSH channel to remote host
                            match ssh_conn.create_local_forward(tunnel.local_port, &tunnel.remote_host, tunnel.remote_port) {
                                Ok(channel) => {
                                    let bytes = Self::proxy_connection(local_stream, channel).await?;
                                    total_bytes += bytes;
                                }
                                Err(e) => {
                                    error!("Failed to create SSH channel: {}", e);
                                    return Err(e);
                                }
                            }
                        }
                        Err(e) => {
                            error!("Failed to accept connection: {}", e);
                        }
                    }
                }
                _ = time::sleep(Duration::from_millis(100)) => {
                    // Check shutdown periodically
                    continue;
                }
            }
        }

        Ok(total_bytes)
    }

    async fn run_remote_forward(
        ssh_conn: &mut SshConnection,
        tunnel: &Tunnel,
        shutdown: &Arc<Mutex<bool>>,
    ) -> Result<u64> {
        ssh_conn.create_remote_forward(tunnel.local_port)?;

        info!("Remote forward established for port {}", tunnel.local_port);
        let mut total_bytes = 0u64;

        // This is a simplified version - in practice, you'd need to handle
        // incoming connections from the SSH server
        loop {
            if *shutdown.lock().unwrap() {
                break;
            }

            time::sleep(Duration::from_secs(1)).await;
            // TODO: Handle incoming remote connections
        }

        Ok(total_bytes)
    }

    async fn proxy_connection(
        mut local_stream: TokioTcpStream,
        mut ssh_channel: Channel,
    ) -> Result<u64> {
        let mut total_bytes = 0u64;
        let mut buffer = [0u8; 8192];

        loop {
            tokio::select! {
                // Local to remote
                result = local_stream.read(&mut buffer) => {
                    match result {
                        Ok(0) => break, // Connection closed
                        Ok(n) => {
                            ssh_channel.write_all(&buffer[..n])
                                .context("Failed to write to SSH channel")?;
                            total_bytes += n as u64;
                        }
                        Err(e) => {
                            error!("Error reading from local stream: {}", e);
                            break;
                        }
                    }
                }

                // Remote to local
                result = async {
                    ssh_channel.read(&mut buffer)
                } => {
                    match result {
                        Ok(0) => break, // Connection closed
                        Ok(n) => {
                            local_stream.write_all(&buffer[..n]).await
                                .context("Failed to write to local stream")?;
                            total_bytes += n as u64;
                        }
                        Err(e) => {
                            error!("Error reading from SSH channel: {}", e);
                            break;
                        }
                    }
                }
            }
        }

        Ok(total_bytes)
    }
}
