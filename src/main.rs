use anyhow::{anyhow, Context, Result};
use dotenvy::dotenv;
use log::{error, info, warn};
use std::{
    env, fs,
    path::PathBuf,
    process::{Command, Stdio},
    sync::{Arc, Mutex},
    time::Duration,
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

        let parts: Vec<_> = rest.split(" to ").collect();
        if parts.len() != 2 {
            return Err(anyhow!("Line {}: invalid format", i + 1));
        }

        let (from_host, from_port) = parse_host_port(parts[0].trim())?;
        let (to_host, to_port) = parse_host_port(parts[1].trim())?;

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

fn build_ssh_command(tunnel: &Tunnel, ssh_user: &str, ssh_host: &str, ssh_key: &str) -> Command {
    let mut cmd = Command::new("ssh");
    cmd.arg("-i").arg(ssh_key);
    cmd.arg("-vv");
    cmd.arg("-o").arg("StrictHostKeyChecking=no");
    cmd.arg("-o").arg("ExitOnForwardFailure=yes");
    cmd.arg("-N");

    match tunnel.direction {
        TunnelDirection::Receive => {
            // local port forward
            let spec = format!(
                "{}:{}:{}",
                tunnel.from_host,
                tunnel.from_port,
                format!("{}:{}", tunnel.to_host, tunnel.to_port)
            );
            cmd.arg("-L").arg(spec);
        }
        TunnelDirection::Send => {
            // remote port forward
            let spec = format!(
                "{}:{}:{}",
                tunnel.from_host,
                tunnel.from_port,
                format!("{}:{}", tunnel.to_host, tunnel.to_port)
            );
            cmd.arg("-R").arg(spec);
        }
    }

    cmd.arg(format!("{}@{}", ssh_user, ssh_host));
    cmd.stdout(Stdio::null());
    cmd.stderr(Stdio::piped());
    cmd
}

async fn manage_tunnel(tunnel: Tunnel, ssh_user: String, ssh_host: String, ssh_key: String) {
    let mut delay = Duration::from_secs(1);

    loop {
        let command = build_ssh_command(&tunnel, &ssh_user, &ssh_host, &ssh_key);

        // Convert std::process::Command to tokio::process::Command
        let mut command = TokioCommand::from(command);

        command.stdout(Stdio::null());
        command.stderr(Stdio::piped());

        info!(
            "Starting tunnel: {:?} {}:{} → {}:{}",
            tunnel.direction, tunnel.from_host, tunnel.from_port, tunnel.to_host, tunnel.to_port
        );

        match command.spawn() {
            Ok(mut child) => {
                let stderr = child.stderr.take().expect("No stderr pipe");
                let reader = BufReader::new(stderr);
                let mut lines = reader.lines();

                // Spawn task to log stderr output
                let logging_task = tokio::spawn(async move {
                    while let Ok(Some(line)) = lines.next_line().await {
                        info!("[ssh ] {}", line);
                    }
                });

                match child.wait().await {
                    Ok(status) => {
                        warn!("Tunnel process exited with: {}", status);
                    }
                    Err(e) => {
                        error!("Failed to wait on tunnel process: {:?}", e);
                    }
                }

                let _ = logging_task.await;
            }
            Err(e) => {
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
    dotenv().context("Failed to load .env")?;

    env_logger::init();

    let ssh_host = env::var("SSH_HOST")?;
    let ssh_user = env::var("SSH_USER")?;
    let ssh_key_raw = env::var("SSH_PRIVATE_KEY")?;
    let _ssh_key_path = PathBuf::from(&ssh_key_raw);

    let ssh_key_raw = env::var("SSH_PRIVATE_KEY")?;
    let ssh_key_path = PathBuf::from(&ssh_key_raw);

    let ssh_key = if ssh_key_path.is_absolute() {
        ssh_key_path
    } else {
        let abs_path = env::current_dir()
            .context("Failed to get current directory")?
            .join(&ssh_key_path);

        if !abs_path.exists() {
            return Err(anyhow!(
                "SSH key file not found: {}",
                abs_path.to_string_lossy()
            ));
        }

        abs_path
            .canonicalize()
            .context("Failed to canonicalize SSH key path")?
    };

    let config_path = PathBuf::from("m-tunnel.conf");

    if !config_path.exists() {
        return Err(anyhow!(
            "Missing m-tunnel.conf. Create it in the root directory."
        ));
    }

    let tunnels = parse_config(&config_path)?;

    let shutdown_notify = Arc::new(Mutex::new(false));

    // Graceful shutdown
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
        let ssh_user = ssh_user.clone();
        let ssh_host = ssh_host.clone();
        let ssh_key = ssh_key.to_string_lossy().to_string();

        handles.push(tokio::spawn(manage_tunnel(
            tunnel, ssh_user, ssh_host, ssh_key,
        )));
    }

    // Wait until shutdown
    while !*shutdown_notify.lock().unwrap() {
        time::sleep(Duration::from_secs(1)).await;
    }

    info!("Exiting...");
    Ok(())
}
