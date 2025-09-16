mod config;
mod metrics;
mod security;
mod tunnel;

// SSH2 native implementation (simple version for testing)
mod tunnel_ssh2_simple;

#[cfg(test)]
mod tests;

// SSH2 specific tests
#[cfg(test)]
mod tests_ssh2;

use anyhow::Result;
use config::Config;
use log::info;
use metrics::MetricsCollector;
use std::sync::Arc;
use tokio::signal;

#[tokio::main]
async fn main() -> Result<()> {
    env_logger::init();

    info!("Starting m-tunnel-rust v{}", env!("CARGO_PKG_VERSION"));

    // Load configuration (supports both legacy and new TOML formats)
    let config = Config::load()?;

    info!("Loaded configuration with {} tunnels", config.tunnels.len());
    info!(
        "SSH target: {}@{}:{}",
        config.ssh.user, config.ssh.host, config.ssh.port
    );

    // Check for CLI arguments
    let args: Vec<String> = std::env::args().collect();
    let use_ssh2 =
        args.contains(&"--ssh2".to_string()) || std::env::var("M_TUNNEL_USE_SSH2").is_ok();
    let dry_run = args.contains(&"--dry-run".to_string());

    if dry_run {
        info!("🧪 DRY RUN MODE - Configuration validation only");
        info!("✅ Configuration is valid!");
        return Ok(());
    }

    info!(
        "Using {} implementation",
        if use_ssh2 { "SSH2 library" } else { "SSH CLI" }
    );

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

    // Create tunnel manager - use SSH2 implementation for testing
    let tunnel_manager = if use_ssh2 {
        info!("🚀 Using SSH2 library implementation");
        tunnel_ssh2_simple::TunnelManager::new(config, metrics).await?
    } else {
        info!("🔧 Using SSH CLI implementation (fallback)");
        // For now, fallback to SSH2 as well during testing
        tunnel_ssh2_simple::TunnelManager::new(config, metrics).await?
    };

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
