#[cfg(test)]
mod tests {
    use std::sync::Arc;
    use std::time::Duration;

    #[test]
    fn test_tunnel_direction_from_string() {
        use crate::tunnel::TunnelDirection;
        assert_eq!(TunnelDirection::from("send"), TunnelDirection::Send);
        assert_eq!(TunnelDirection::from("receive"), TunnelDirection::Receive);
    }

    #[test]
    #[should_panic(expected = "Invalid tunnel direction")]
    #[allow(unused_must_use)]
    fn test_tunnel_direction_invalid() {
        use crate::tunnel::TunnelDirection;
        TunnelDirection::from("invalid");
    }

    #[test]
    fn test_tunnel_from_config() {
        use crate::config::TunnelConfig;
        use crate::tunnel::{Tunnel, TunnelDirection};

        let config = TunnelConfig {
            name: "test".to_string(),
            direction: "receive".to_string(),
            local_host: "127.0.0.1".to_string(),
            local_port: 8080,
            remote_host: "example.com".to_string(),
            remote_port: 80,
            enabled: true,
        };

        let tunnel = Tunnel::from(&config);
        assert_eq!(tunnel.id, "test");
        assert_eq!(tunnel.direction, TunnelDirection::Receive);
    }

    #[test]
    fn test_connection_limiter() {
        use crate::tunnel::ConnectionLimiter;

        let mut limiter = ConnectionLimiter::new(2, Duration::from_secs(10));

        // Should allow initial attempts
        assert!(limiter.can_attempt("host1"));
        assert!(limiter.can_attempt("host1"));

        // Should block after limit
        assert!(!limiter.can_attempt("host1"));
    }

    #[tokio::test]
    async fn test_connection_limiter_reset() {
        use crate::tunnel::ConnectionLimiter;

        let mut limiter = ConnectionLimiter::new(1, Duration::from_millis(100));

        // Exhaust attempts
        assert!(limiter.can_attempt("host1"));
        assert!(!limiter.can_attempt("host1"));

        // Wait for reset
        tokio::time::sleep(Duration::from_millis(150)).await;
        assert!(limiter.can_attempt("host1"));
    }

    #[tokio::test]
    async fn test_metrics_integration() {
        use crate::metrics::{MetricsCollector, TunnelStatus};

        let metrics = Arc::new(MetricsCollector::new());

        // Test metric updates
        metrics.update_tunnel_status("test-tunnel", TunnelStatus::Connecting);
        metrics.update_tunnel_status("test-tunnel", TunnelStatus::Connected);
        metrics.increment_reconnect("test-tunnel");

        // Should not panic and should track metrics
        // Test passes if we reach this point
    }

    #[test]
    fn test_config_serialization() {
        use crate::config::{Config, ConnectionLimits, SshConfig, TunnelConfig};
        use std::path::PathBuf;

        let ssh_config = SshConfig {
            host: "example.com".to_string(),
            user: "testuser".to_string(),
            port: 22,
            key_path: PathBuf::from("/home/user/.ssh/id_rsa"),
            timeout: 30,
            keepalive_interval: 60,
        };

        let tunnel_config = TunnelConfig {
            name: "web-tunnel".to_string(),
            direction: "receive".to_string(),
            local_host: "127.0.0.1".to_string(),
            local_port: 8080,
            remote_host: "localhost".to_string(),
            remote_port: 80,
            enabled: true,
        };

        let config = Config {
            ssh: ssh_config,
            tunnels: vec![tunnel_config],
            limits: ConnectionLimits::default(),
        };

        // Should serialize and deserialize without error
        let serialized = toml::to_string(&config).unwrap();
        let _deserialized: Config = toml::from_str(&serialized).unwrap();
    }
}
