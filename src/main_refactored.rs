mod config;
mod metrics;
mod security;
mod tunnel;
mod ssh;

#[cfg(test)]
mod tests;

use anyhow::Result;
use config::Config;
use log::info;
use metrics::MetricsCollector;
use std::sync::Arc;
use tokio::signal;

#[tokio::main]
async fn main() -> Result<()> {
    env_logger::init();
    
    let config = Config::load()?;
    let metrics = Arc::new(MetricsCollector::new());
    
    info!("Starting m-tunnel with {} tunnels", config.tunnels.len());
    
    // Start metrics server if enabled
    if let Some(metrics_port) = std::env::var("METRICS_PORT").ok() {
        let metrics_clone = Arc::clone(&metrics);
        tokio::spawn(async move {
            start_metrics_server(metrics_clone, metrics_port.parse().unwrap()).await;
        });
    }
    
    // Start tunnel manager
    let tunnel_manager = tunnel::TunnelManager::new(config, metrics).await?;
    
    // Wait for shutdown signal
    signal::ctrl_c().await?;
    info!("Shutdown signal received");
    
    tunnel_manager.shutdown().await?;
    info!("Clean shutdown completed");
    
    Ok(())
}

async fn start_metrics_server(metrics: Arc<MetricsCollector>, port: u16) {
    use warp::Filter;
    
    let metrics_route = warp::path("metrics")
        .map(move || {
            metrics.export_prometheus()
        });
    
    warp::serve(metrics_route)
        .run(([0, 0, 0, 0], port))
        .await;
}