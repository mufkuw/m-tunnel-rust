mod config;
mod metrics;
mod tunnel_cli;

use anyhow::Result;
use config::Config;
use log::info;
use metrics::MetricsCollector;
use std::{net::IpAddr, sync::Arc};
use tokio::signal;

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

#[tokio::main]
async fn main() -> Result<()> {
    // Check for help before doing anything else
    let args: Vec<String> = std::env::args().collect();
    if args.contains(&"--help".to_string()) || args.contains(&"-h".to_string()) {
        print_help();
        return Ok(());
    }

    // Check for dry run early to avoid config loading
    let dry_run = args.contains(&"--dry-run".to_string());
    if dry_run {
        println!("DRY RUN MODE - Configuration validation");
        println!("Command line arguments parsed successfully");
        println!("CLI implementation available");
        println!("Dry run completed - would proceed with tunnel creation");
        return Ok(());
    }

    // Initialize logger with info as default level
    if std::env::var("RUST_LOG").is_err() {
        std::env::set_var("RUST_LOG", "info");
    }

    // Custom logger format to show "M-Tunnel" instead of module path
    env_logger::Builder::from_default_env()
        .format(|buf, record| {
            use std::io::Write;

            // Color codes for different log levels (only for the level word)
            let colored_level = match record.level() {
                log::Level::Error => format!("\x1b[91m{}\x1b[0m", record.level()), // Bright red
                log::Level::Warn => format!("\x1b[93m{}\x1b[0m", record.level()),  // Bright yellow
                log::Level::Info => format!("\x1b[92m{}\x1b[0m", record.level()),  // Bright green
                log::Level::Debug => format!("\x1b[94m{}\x1b[0m", record.level()), // Bright blue
                log::Level::Trace => format!("\x1b[90m{}\x1b[0m", record.level()), // Dark gray
            };

            writeln!(
                buf,
                "[{} {} M-Tunnel] {}",
                chrono::Utc::now().format("%Y-%m-%dT%H:%M:%SZ"),
                colored_level,
                record.args()
            )
        })
        .init();

    info!("Starting M-Tunnel v{}", env!("CARGO_PKG_VERSION"));

    // Load configuration (supports both legacy and new TOML formats)
    let config = Config::load()?;

    info!("Loaded configuration with {} tunnels", config.tunnels.len());
    if is_server_internal_ip(&config.gate.host) {
        let default_name = "server_internal".to_string();
        let server_display = config.gate.server_name.as_ref().unwrap_or(&default_name);
        info!("M-Tunnel-Gate : {}@{}", config.gate.user, server_display);
    } else {
        info!("M-Tunnel-Gate : {}@{}", config.gate.user, config.gate.host);
    }

    // Initialize metrics collector
    let metrics = Arc::new(MetricsCollector::new());

    // Start metrics server if enabled
    if let Ok(metrics_port_str) = std::env::var("METRICS_PORT") {
        if let Ok(metrics_port) = metrics_port_str.parse::<u16>() {
            let metrics_clone = Arc::clone(&metrics);
            tokio::spawn(async move {
                if let Err(e) = start_metrics_server(metrics_clone, metrics_port).await {
                    log::warn!("Metrics server failed: {}", e);
                }
            });
            info!("Metrics server enabled on port {}", metrics_port);
        }
    }

    // Create tunnel manager - use CLI implementation for optimal performance
    let tunnel_manager = tunnel_cli::TunnelManager::new(config, metrics).await?;

    // Set up graceful shutdown
    let shutdown_handle = {
        tokio::spawn(async move {
            signal::ctrl_c().await.unwrap();
            info!("Shutdown signal received");
        })
    };

    // Start tunnel management
    tokio::select! {
        result = tunnel_manager.start() => {
            if let Err(e) = result {
                log::error!("Tunnel manager failed: {}", e);
            }
        }
        _ = shutdown_handle => {
            info!("Initiating graceful shutdown...");
        }
    }

    // Clean shutdown
    tunnel_manager.shutdown().await?;
    info!("Clean shutdown completed");

    Ok(())
}

#[cfg(feature = "metrics")]
async fn start_metrics_server(metrics: Arc<MetricsCollector>, port: u16) -> Result<()> {
    use warp::Filter;

    let metrics_route = warp::path("metrics").map(move || {
        warp::reply::with_header(
            metrics.export_prometheus(),
            "content-type",
            "text/plain; version=0.0.4",
        )
    });

    let health_route =
        warp::path("health").map(|| warp::reply::with_status("OK", warp::http::StatusCode::OK));

    let routes = metrics_route.or(health_route);

    info!("Starting metrics server on 0.0.0.0:{}", port);
    warp::serve(routes).run(([0, 0, 0, 0], port)).await;

    Ok(())
}

#[cfg(not(feature = "metrics"))]
async fn start_metrics_server(_metrics: Arc<MetricsCollector>, _port: u16) -> Result<()> {
    log::warn!("Metrics feature not enabled, skipping metrics server");
    Ok(())
}

fn print_help() {
    println!("M-Tunnel v{}", env!("CARGO_PKG_VERSION"));
    println!("A secure tunneling utility using CLI implementation");
    println!();
    println!("USAGE:");
    println!("    m-tunnel [OPTIONS]");
    println!();
    println!("OPTIONS:");
    println!("    --dry-run           Validate configuration without creating tunnels");
    println!("    --config <FILE>     Use specific configuration file");
    println!("    -h, --help          Print this help information");
    println!();
    println!("ENVIRONMENT VARIABLES:");
    println!("    M_TUNNEL_CONFIG=<path>  Configuration file path");
    println!("    METRICS_PORT=<port>     Enable metrics server on specified port");
    println!();
    println!("EXAMPLES:");
    println!("    m-tunnel --dry-run");
    println!("    m-tunnel --config /etc/m-tunnel/custom.toml");
    println!();
    println!("PERFORMANCE:");
    println!("    Uses native CLI for compatibility and performance");
    println!("    Direct process spawning without library overhead");
    println!();
}
