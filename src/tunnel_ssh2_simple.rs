use anyhow::{Context, Result};
use log::{error, info, warn};
use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
    time::{Duration, Instant},
};
use tokio::time;

use crate::config::{Config, TunnelConfig};
use crate::metrics::{MetricsCollector, TunnelStatus};

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

pub struct TunnelManager {
    pub config: Config,
    pub metrics: Arc<MetricsCollector>,
    pub connection_limiter: Arc<Mutex<ConnectionLimiter>>,
    pub shutdown: Arc<Mutex<bool>>,
}

impl TunnelManager {
    pub async fn new(config: Config, metrics: Arc<MetricsCollector>) -> Result<Self> {
        // Basic validation for now
        info!("Creating SSH2-based tunnel manager");
        
        let connection_limiter = Arc::new(Mutex::new(ConnectionLimiter::new(
            config.limits.max_attempts,
            Duration::from_secs(config.limits.retry_window_secs),
        )));

        Ok(Self {
            config,
            metrics,
            connection_limiter,
            shutdown: Arc::new(Mutex::new(false)),
        })
    }

    pub async fn start(&self) -> Result<()> {
        info!("🚀 Starting SSH2 tunnel manager with {} tunnels", self.config.tunnels.len());

        let mut handles = vec![];

        for tunnel_config in &self.config.tunnels {
            if !tunnel_config.enabled {
                info!("Skipping disabled tunnel: {}", tunnel_config.name);
                continue;
            }

            let tunnel = Tunnel::from(tunnel_config);
            let ssh_config = self.config.ssh.clone();
            let metrics = Arc::clone(&self.metrics);
            let shutdown = Arc::clone(&self.shutdown);

            handles.push(tokio::spawn(async move {
                Self::manage_tunnel_mock(tunnel, ssh_config, metrics, shutdown).await;
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

    // Mock implementation for testing
    async fn manage_tunnel_mock(
        tunnel: Tunnel,
        _ssh_config: crate::config::SshConfig,
        metrics: Arc<MetricsCollector>,
        shutdown: Arc<Mutex<bool>>,
    ) {
        info!("🧪 MOCK: Managing tunnel {} ({}:{} → {}:{})", 
              tunnel.id, tunnel.local_host, tunnel.local_port,
              tunnel.remote_host, tunnel.remote_port);

        metrics.update_tunnel_status(&tunnel.id, TunnelStatus::Connected);

        // Simulate tunnel running
        let mut iteration = 0;
        while !*shutdown.lock().unwrap() && iteration < 10 {
            time::sleep(Duration::from_secs(1)).await;
            iteration += 1;
            
            if iteration == 5 {
                info!("🧪 MOCK: Tunnel {} transferring data...", tunnel.id);
            }
        }

        metrics.update_tunnel_status(&tunnel.id, TunnelStatus::Disconnected);
        info!("🧪 MOCK: Tunnel {} completed", tunnel.id);
    }
}