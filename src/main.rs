use anyhow::{anyhow, Context, Result};
use dotenvy;
use log::{debug, error, info, warn};
use std::{
    collections::HashMap,
    env, fs,
    path::PathBuf,
    process::{Command, Stdio},
    sync::{Arc, Mutex},
    time::{Duration, Instant},
};
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::process::Command as TokioCommand;
use tokio::{signal, time};

#[derive(Debug, Clone, PartialEq, Eq)]
enum TunnelDirection {
    Receive, // local → remote
    Send,    // remote → local
}

#[derive(Debug, Clone)]
struct Tunnel {
    direction: TunnelDirection,
    from_host: String,
    from_port: u16,
    to_host: String,
    to_port: u16,
}

#[derive(Debug, Clone)]
struct SshConfig {
    host: String,
    user: String,
    port: String,
    key: String,
}

#[derive(Debug)]
struct TunnelMetrics {
    uptime_start: Instant,
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

fn parse_config(path: &PathBuf) -> Result<Vec<Tunnel>> {
    let content = fs::read_to_string(path)?;
    let mut tunnels = Vec::new();

    for (i, line) in content.lines().enumerate() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }

        let (direction, rest) = if let Some(r) = line.strip_prefix("send -- ") {
            (TunnelDirection::Send, r)
        } else if let Some(r) = line.strip_prefix("receive -- ") {
            (TunnelDirection::Receive, r)
        } else {
            return Err(anyhow!(
                "Line {}: must start with 'send --' or 'receive --'",
                i + 1
            ));
        };

        let parts: Vec<_> = match direction {
            TunnelDirection::Receive => rest.split(" from ").collect(),
            TunnelDirection::Send => rest.split(" to ").collect(),
        };

        if parts.len() != 2 {
            return Err(anyhow!(
                "Line {}: invalid format. Expected '{}' keyword for {:?}",
                i + 1,
                if direction == TunnelDirection::Receive {
                    "from"
                } else {
                    "to"
                },
                direction
            ));
        }

        let (from_host, from_port, to_host, to_port) = match direction {
            TunnelDirection::Send => {
                let (from_host, from_port) = parse_host_port(parts[0].trim())?;
                let (to_host, to_port) = parse_host_port(parts[1].trim())?;
                (from_host, from_port, to_host, to_port)
            }
            TunnelDirection::Receive => {
                let (to_host, to_port) = parse_host_port(parts[0].trim())?;
                let (from_host, from_port) = parse_host_port(parts[1].trim())?;
                (from_host, from_port, to_host, to_port)
            }
        };

        tunnels.push(Tunnel {
            direction,
            from_host,
            from_port,
            to_host,
            to_port,
        });
    }

    Ok(tunnels)
}

fn parse_host_port(s: &str) -> Result<(String, u16)> {
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

// Added ssh_port parameter here
fn build_ssh_command(
    tunnel: &Tunnel,
    ssh_user: &str,
    ssh_host: &str,
    ssh_port: &str,
    ssh_key: &str,
) -> Command {
    let mut cmd = Command::new("ssh");
    cmd.arg("-i").arg(ssh_key);
    cmd.arg("-p").arg(ssh_port);

    // Security: Enable strict host key checking with known_hosts
    cmd.arg("-o").arg("StrictHostKeyChecking=yes");
    cmd.arg("-o")
        .arg("UserKnownHostsFile=/etc/m-tunnel/known_hosts");
    cmd.arg("-o").arg("HashKnownHosts=yes");

    // Performance: SSH multiplexing for connection reuse
    cmd.arg("-o").arg("ControlMaster=auto");
    cmd.arg("-o").arg("ControlPath=/tmp/ssh-m-tunnel-%r@%h:%p");
    cmd.arg("-o").arg("ControlPersist=60s");

    // Performance: Network optimizations
    cmd.arg("-o").arg("TCPKeepAlive=yes");
    cmd.arg("-o").arg("ServerAliveInterval=60");
    cmd.arg("-o").arg("ServerAliveCountMax=3");
    cmd.arg("-o").arg("Compression=yes");

    // Security: Reduce verbosity to avoid info leakage (remove -vv)
    cmd.arg("-o").arg("LogLevel=ERROR");
    cmd.arg("-o").arg("ExitOnForwardFailure=yes");
    cmd.arg("-N");

    match tunnel.direction {
        TunnelDirection::Receive => {
            // local port forward: -L [bind_address:]port:host:hostport
            let spec = format!(
                "{}:{}:{}:{}",
                tunnel.from_host, tunnel.from_port, tunnel.to_host, tunnel.to_port
            );
            cmd.arg("-L").arg(spec);
        }
        TunnelDirection::Send => {
            // remote port forward: -R [bind_address:]port:host:hostport
            let spec = format!(
                "{}:{}:{}:{}",
                tunnel.from_host, tunnel.from_port, tunnel.to_host, tunnel.to_port
            );
            cmd.arg("-R").arg(spec);
        }
    }

    cmd.arg(format!("{}@{}", ssh_user, ssh_host));
    cmd.stdout(Stdio::null());
    cmd.stderr(Stdio::piped());
    cmd
}

// Optimized tunnel management with shared configuration and security
async fn manage_tunnel_optimized(
    tunnel: Tunnel,
    ssh_config: Arc<SshConfig>,
    connection_limiter: Arc<Mutex<ConnectionLimiter>>,
) {
    let mut delay = Duration::from_secs(1);
    let mut metrics = TunnelMetrics {
        uptime_start: Instant::now(),
        reconnect_count: 0,
        last_error: None,
    };

    loop {
        // Check connection rate limiting for security
        let can_attempt = {
            let mut limiter = connection_limiter.lock().unwrap();
            limiter.can_attempt(&ssh_config.host)
        }; // MutexGuard is dropped here

        if !can_attempt {
            warn!(
                "Rate limit exceeded for host {}, waiting...",
                ssh_config.host
            );
            time::sleep(Duration::from_secs(60)).await;
            continue;
        }

        let command = build_ssh_command(
            &tunnel,
            &ssh_config.user,
            &ssh_config.host,
            &ssh_config.port,
            &ssh_config.key,
        );

        // Convert std::process::Command to tokio::process::Command
        let mut command = TokioCommand::from(command);
        command.stdout(Stdio::null());
        command.stderr(Stdio::piped());

        info!(
            "Starting tunnel: {:?} {}:{} → {}:{} (attempt #{})",
            tunnel.direction,
            tunnel.from_host,
            tunnel.from_port,
            tunnel.to_host,
            tunnel.to_port,
            metrics.reconnect_count + 1
        );

        match command.spawn() {
            Ok(mut child) => {
                let stderr = child.stderr.take().expect("No stderr pipe");
                let reader = BufReader::new(stderr);
                let mut lines = reader.lines();

                let logging_task = tokio::spawn(async move {
                    while let Ok(Some(line)) = lines.next_line().await {
                        // Only log errors and warnings to reduce noise
                        if line.contains("ERROR") || line.contains("WARNING") {
                            error!("[ssh] {}", line);
                        } else {
                            debug!("[ssh] {}", line);
                        }
                    }
                });

                match child.wait().await {
                    Ok(status) => {
                        metrics.reconnect_count += 1;
                        let uptime = metrics.uptime_start.elapsed();
                        warn!(
                            "Tunnel process exited with: {} (uptime: {:?}, reconnects: {})",
                            status, uptime, metrics.reconnect_count
                        );

                        if status.success() {
                            metrics.last_error = None;
                            // Reset delay on successful connection
                            delay = Duration::from_secs(1);
                        } else {
                            metrics.last_error = Some(format!("Exit code: {}", status));
                        }
                    }
                    Err(e) => {
                        metrics.reconnect_count += 1;
                        metrics.last_error = Some(format!("Process error: {}", e));
                        error!("Failed to wait on tunnel process: {:?}", e);
                    }
                }

                let _ = logging_task.await;
            }
            Err(e) => {
                metrics.last_error = Some(format!("Spawn error: {}", e));
                error!("Failed to spawn ssh tunnel: {:?}", e);
            }
        }

        warn!("Restarting tunnel in {}s...", delay.as_secs());
        time::sleep(delay).await;
        delay = std::cmp::min(delay * 2, Duration::from_secs(60));
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    // Load .env from /etc/m-tunnel/ or current directory for development
    let env_path = if PathBuf::from("/etc/m-tunnel/.env").exists() {
        "/etc/m-tunnel/.env"
    } else {
        ".env"
    };

    dotenvy::from_path(env_path).context("Failed to load .env file")?;
    env_logger::init();

    let ssh_host = env::var("SSH_HOST")?;
    let ssh_user = env::var("SSH_USER")?;
    let ssh_port = env::var("SSH_PORT")?;
    let ssh_key_raw = env::var("SSH_PRIVATE_KEY")?;

    // Resolve SSH key path - check /etc/m-tunnel/ first, then relative paths
    let ssh_key_path = PathBuf::from(&ssh_key_raw);
    let ssh_key = if ssh_key_path.is_absolute() {
        ssh_key_path
    } else {
        // Try /etc/m-tunnel/ first for installed system
        let system_key_path = PathBuf::from("/etc/m-tunnel").join(&ssh_key_path);
        if system_key_path.exists() {
            system_key_path
        } else {
            // Fall back to current directory for development
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

    // Resolve config path - check /etc/m-tunnel/ first, then current directory
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

    info!("Using config: {}", config_path.display());
    info!("Using SSH key: {}", ssh_key.display());

    let tunnels = parse_config(&config_path)?;

    // Create shared SSH configuration using Arc to avoid cloning
    let ssh_config = Arc::new(SshConfig {
        host: ssh_host,
        user: ssh_user,
        port: ssh_port,
        key: ssh_key.to_string_lossy().to_string(),
    });

    // Initialize connection limiter for security
    let connection_limiter = Arc::new(Mutex::new(ConnectionLimiter::new(
        5,                        // max 5 attempts
        Duration::from_secs(300), // within 5 minutes
    )));

    let shutdown_notify = Arc::new(Mutex::new(false));

    {
        let notify = shutdown_notify.clone();
        tokio::spawn(async move {
            signal::ctrl_c().await.unwrap();
            *notify.lock().unwrap() = true;
            info!("Shutdown requested");
        });
    }

    let mut handles = vec![];

    for tunnel in tunnels {
        let config = Arc::clone(&ssh_config);
        let limiter = Arc::clone(&connection_limiter);

        handles.push(tokio::spawn(manage_tunnel_optimized(
            tunnel, config, limiter,
        )));
    }

    while !*shutdown_notify.lock().unwrap() {
        time::sleep(Duration::from_secs(1)).await;
    }

    info!("Exiting...");
    Ok(())
}
