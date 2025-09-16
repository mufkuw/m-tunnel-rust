#[cfg(test)]
mod tests {
    use crate::config::Config;
    use crate::tunnel::{ConnectionLimiter, TunnelDirection};
    use std::io::Write;
    use std::time::Duration;
    use tempfile::NamedTempFile;

    #[test]
    fn test_parse_host_port() {
        assert_eq!(
            Config::parse_host_port("localhost:8080").unwrap(),
            ("localhost".to_string(), 8080)
        );

        assert_eq!(
            Config::parse_host_port("127.0.0.1:22").unwrap(),
            ("127.0.0.1".to_string(), 22)
        );

        // Test error cases
        assert!(Config::parse_host_port("invalid").is_err());
        assert!(Config::parse_host_port("localhost:99999").is_err());
    }

    #[test]
    fn test_connection_limiter() {
        let mut limiter = ConnectionLimiter::new(2, Duration::from_secs(10));

        // First two attempts should succeed
        assert!(limiter.can_attempt("test.com"));
        assert!(limiter.can_attempt("test.com"));

        // Third attempt should fail
        assert!(!limiter.can_attempt("test.com"));

        // Different host should work
        assert!(limiter.can_attempt("other.com"));
    }

    #[test]
    fn test_parse_legacy_config() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "# Comment line").unwrap();
        writeln!(temp_file, "send -- 127.0.0.1:22 to 192.168.1.1:2222").unwrap();
        writeln!(temp_file, "receive -- 0.0.0.0:8080 from 10.0.0.1:80").unwrap();

        let config_path = temp_file.path().to_path_buf();
        let tunnels = Config::parse_legacy_tunnels(&config_path).unwrap();

        assert_eq!(tunnels.len(), 2);
        assert_eq!(tunnels[0].direction, "send");
        assert_eq!(tunnels[1].direction, "receive");
    }

    #[tokio::test]
    async fn test_tunnel_config_conversion() {
        use crate::config::TunnelConfig;
        use crate::tunnel::Tunnel;

        let config = TunnelConfig {
            name: "test-tunnel".to_string(),
            direction: "send".to_string(),
            local_host: "127.0.0.1".to_string(),
            local_port: 8080,
            remote_host: "remote.example.com".to_string(),
            remote_port: 80,
            enabled: true,
        };

        let tunnel = Tunnel::from(&config);
        assert_eq!(tunnel.id, "test-tunnel");
        assert_eq!(tunnel.direction, TunnelDirection::Send);
        assert_eq!(tunnel.local_port, 8080);
        assert_eq!(tunnel.remote_port, 80);
    }
}
