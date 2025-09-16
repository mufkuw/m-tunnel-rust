use serde::Serialize;
use std::{
    collections::HashMap,
    sync::{Arc, RwLock},
    time::{Duration, Instant},
};

#[derive(Debug, Clone, Serialize)]
pub struct TunnelStats {
    pub tunnel_id: String,
    pub status: TunnelStatus,
    pub uptime: Duration,
    pub reconnect_count: u64,
    pub bytes_sent: u64,
    pub bytes_received: u64,
    pub last_error: Option<String>,
    pub connection_latency: Option<Duration>,
}

impl Default for TunnelStats {
    fn default() -> Self {
        Self {
            tunnel_id: String::new(),
            status: TunnelStatus::Disconnected,
            uptime: Duration::from_secs(0),
            reconnect_count: 0,
            bytes_sent: 0,
            bytes_received: 0,
            last_error: None,
            connection_latency: None,
        }
    }
}

#[derive(Debug, Clone, Serialize)]
#[allow(dead_code)]
pub enum TunnelStatus {
    Connected,
    Connecting,
    Disconnected,
    Error,
}

pub struct MetricsCollector {
    stats: Arc<RwLock<HashMap<String, TunnelStats>>>,
    start_time: Instant,
}

impl MetricsCollector {
    pub fn new() -> Self {
        Self {
            stats: Arc::new(RwLock::new(HashMap::new())),
            start_time: Instant::now(),
        }
    }

    pub fn update_tunnel_status(&self, tunnel_id: &str, status: TunnelStatus) {
        let mut stats = self.stats.write().unwrap();
        if let Some(tunnel_stats) = stats.get_mut(tunnel_id) {
            tunnel_stats.status = status;
        }
    }

    #[allow(dead_code)]
    pub fn increment_reconnect(&self, tunnel_id: &str) {
        let mut stats = self.stats.write().unwrap();
        stats
            .entry(tunnel_id.to_string())
            .or_default()
            .reconnect_count += 1;
    }

    #[allow(dead_code)]
    pub fn get_summary(&self) -> HashMap<String, TunnelStats> {
        let stats = self.stats.read().unwrap();
        stats.clone()
    }

    /// Export metrics in Prometheus format
    pub fn export_prometheus(&self) -> String {
        let stats = self.stats.read().unwrap();
        let mut output = String::new();

        output.push_str("# HELP mtunnel_uptime_seconds Total uptime in seconds\n");
        output.push_str("# TYPE mtunnel_uptime_seconds counter\n");
        output.push_str(&format!(
            "mtunnel_uptime_seconds {}\n",
            self.start_time.elapsed().as_secs()
        ));

        for (id, stat) in stats.iter() {
            output.push_str(&format!(
                "mtunnel_reconnects_total{{tunnel=\"{}\"}} {}\n",
                id, stat.reconnect_count
            ));

            let status_value = match stat.status {
                TunnelStatus::Connected => 1,
                TunnelStatus::Connecting => 2,
                TunnelStatus::Disconnected => 3,
                TunnelStatus::Error => 4,
            };

            output.push_str(&format!(
                "mtunnel_status{{tunnel=\"{}\"}} {}\n",
                id, status_value
            ));
        }

        output
    }
}
