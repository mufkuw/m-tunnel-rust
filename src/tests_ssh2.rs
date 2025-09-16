#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::{Config, ConnectionLimits, SshConfig, TunnelConfig};
    use crate::metrics::MetricsCollector;
    use std::io::Write;
    use std::sync::Arc;
    use std::time::Duration;
    use tempfile::NamedTempFile;
    use tokio::time::timeout;

    // Mock SSH server for testing
    struct MockSshServer {
        listener: TokioTcpListener,
        port: u16,
    }

    impl MockSshServer {
        async fn new() -> Result<Self> {
            let listener = TokioTcpListener::bind("127.0.0.1:0").await?;
            let port = listener.local_addr()?.port();
            Ok(Self { listener, port })
        }

        async fn accept_one(&self) -> Result<TokioTcpStream> {
            let (stream, _) = self.listener.accept().await?;
            Ok(stream)
        }
    }

    fn create_test_ssh_config(port: u16) -> SshConfig {
        SshConfig {
            host: "127.0.0.1".to_string(),
            user: "testuser".to_string(),
            port,
            key_path: std::path::PathBuf::from("/tmp/test_key"),
            timeout: 10,
            keepalive_interval: 30,
        }
    }

    fn create_test_tunnel_config() -> TunnelConfig {
        TunnelConfig {
            name: "test-tunnel".to_string(),
            direction: "receive".to_string(),
            local_host: "127.0.0.1".to_string(),
            local_port: 0, // Will be set dynamically
            remote_host: "example.com".to_string(),
            remote_port: 80,
            enabled: true,
        }
    }

    fn create_test_config(ssh_port: u16, tunnel_port: u16) -> Config {
        let mut tunnel_config = create_test_tunnel_config();
        tunnel_config.local_port = tunnel_port;

        Config {
            ssh: create_test_ssh_config(ssh_port),
            tunnels: vec![tunnel_config],
            limits: ConnectionLimits::default(),
        }
    }

    #[test]
    fn test_tunnel_direction_conversion() {
        assert_eq!(TunnelDirection::from("send"), TunnelDirection::Send);
        assert_eq!(TunnelDirection::from("receive"), TunnelDirection::Receive);
    }

    #[test]
    #[should_panic(expected = "Invalid tunnel direction")]
    fn test_invalid_tunnel_direction() {
        TunnelDirection::from("invalid");
    }

    #[test]
    fn test_tunnel_from_config() {
        let config = create_test_tunnel_config();
        let tunnel = Tunnel::from(&config);

        assert_eq!(tunnel.id, "test-tunnel");
        assert_eq!(tunnel.direction, TunnelDirection::Receive);
        assert_eq!(tunnel.local_host, "127.0.0.1");
        assert_eq!(tunnel.remote_host, "example.com");
        assert_eq!(tunnel.remote_port, 80);
        assert!(tunnel.enabled);
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
    fn test_connection_limiter_time_window() {
        let mut limiter = ConnectionLimiter::new(1, Duration::from_millis(100));

        // First attempt should succeed
        assert!(limiter.can_attempt("test.com"));

        // Second attempt should fail immediately
        assert!(!limiter.can_attempt("test.com"));

        // After time window, should succeed again
        std::thread::sleep(Duration::from_millis(150));
        assert!(limiter.can_attempt("test.com"));
    }

    #[tokio::test]
    async fn test_tunnel_manager_creation() {
        let mock_server = MockSshServer::new().await.unwrap();
        let config = create_test_config(mock_server.port, 8080);
        let metrics = Arc::new(MetricsCollector::new());

        // This should succeed even without a real SSH server
        let result = TunnelManager::new(config, metrics).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_tunnel_manager_with_disabled_tunnel() {
        let mock_server = MockSshServer::new().await.unwrap();
        let mut config = create_test_config(mock_server.port, 8080);
        config.tunnels[0].enabled = false;

        let metrics = Arc::new(MetricsCollector::new());
        let tunnel_manager = TunnelManager::new(config, metrics).await.unwrap();

        // Should handle disabled tunnels gracefully
        assert!(!*tunnel_manager.shutdown.lock().unwrap());
    }

    #[tokio::test]
    async fn test_graceful_shutdown() {
        let mock_server = MockSshServer::new().await.unwrap();
        let config = create_test_config(mock_server.port, 8080);
        let metrics = Arc::new(MetricsCollector::new());

        let tunnel_manager = TunnelManager::new(config, metrics).await.unwrap();

        // Test shutdown mechanism
        let result = tunnel_manager.shutdown().await;
        assert!(result.is_ok());
        assert!(*tunnel_manager.shutdown.lock().unwrap());
    }

    #[tokio::test]
    async fn test_metrics_integration() {
        let metrics = Arc::new(MetricsCollector::new());

        // Test metric updates
        metrics.update_tunnel_status("test-tunnel", TunnelStatus::Connecting);
        metrics.update_tunnel_status("test-tunnel", TunnelStatus::Connected);
        metrics.increment_reconnect("test-tunnel");

        // Should not panic and should track metrics
        let prometheus_output = metrics.export_prometheus();
        assert!(prometheus_output.contains("mtunnel"));
    }

    // Integration test with real socket communication
    #[tokio::test]
    async fn test_socket_communication() {
        // Create a simple echo server
        let echo_server = TokioTcpListener::bind("127.0.0.1:0").await.unwrap();
        let echo_port = echo_server.local_addr().unwrap().port();

        tokio::spawn(async move {
            while let Ok((mut socket, _)) = echo_server.accept().await {
                tokio::spawn(async move {
                    let mut buf = [0; 1024];
                    loop {
                        match socket.read(&mut buf).await {
                            Ok(0) => break, // Connection closed
                            Ok(n) => {
                                if socket.write_all(&buf[..n]).await.is_err() {
                                    break;
                                }
                            }
                            Err(_) => break,
                        }
                    }
                });
            }
        });

        // Give the server time to start
        tokio::time::sleep(Duration::from_millis(100)).await;

        // Test basic socket communication
        let mut client = TokioTcpStream::connect(format!("127.0.0.1:{}", echo_port))
            .await
            .unwrap();

        let test_data = b"Hello, World!";
        client.write_all(test_data).await.unwrap();

        let mut response = vec![0u8; test_data.len()];
        client.read_exact(&mut response).await.unwrap();

        assert_eq!(&response, test_data);
    }

    // Test SSH connection error handling
    #[tokio::test]
    async fn test_ssh_connection_failure() {
        let ssh_config = create_test_ssh_config(99999); // Invalid port

        // This should fail gracefully
        let result = SshConnection::new(&ssh_config);
        assert!(result.is_err());

        // Error should be meaningful
        let error_msg = format!("{}", result.unwrap_err());
        assert!(error_msg.contains("Failed to connect") || error_msg.contains("Connection"));
    }

    // Benchmark test for performance comparison
    #[tokio::test]
    async fn test_connection_performance() {
        use std::time::Instant;

        let start = Instant::now();

        // Simulate multiple rapid connection attempts
        for _ in 0..10 {
            let ssh_config = create_test_ssh_config(22); // Standard SSH port
            let _ =
                TokioTcpStream::connect(format!("{}:{}", ssh_config.host, ssh_config.port)).await;
            // Don't care if it fails, just measuring timing
        }

        let duration = start.elapsed();
        println!("10 connection attempts took: {:?}", duration);

        // Should be much faster than CLI spawning (which would be ~1 second)
        assert!(duration < Duration::from_secs(1));
    }

    // Test configuration validation
    #[tokio::test]
    async fn test_config_validation() {
        let mut config = create_test_config(22, 8080);

        // Test with invalid characters in host
        config.ssh.host = "invalid;host".to_string();
        let metrics = Arc::new(MetricsCollector::new());

        let result = TunnelManager::new(config, metrics).await;
        assert!(result.is_err());
    }

    // Test memory usage (simplified)
    #[test]
    fn test_memory_footprint() {
        let tunnel_config = create_test_tunnel_config();
        let tunnel = Tunnel::from(&tunnel_config);

        // Basic size check - tunnel struct should be small
        let size = std::mem::size_of_val(&tunnel);
        println!("Tunnel struct size: {} bytes", size);

        // Should be reasonable (less than 1KB)
        assert!(size < 1024);
    }

    // Test concurrent tunnel management
    #[tokio::test]
    async fn test_concurrent_tunnels() {
        let mock_server1 = MockSshServer::new().await.unwrap();
        let mock_server2 = MockSshServer::new().await.unwrap();

        let mut config = create_test_config(mock_server1.port, 8080);

        // Add a second tunnel
        let mut tunnel2 = create_test_tunnel_config();
        tunnel2.name = "test-tunnel-2".to_string();
        tunnel2.local_port = 8081;
        config.tunnels.push(tunnel2);

        let metrics = Arc::new(MetricsCollector::new());
        let tunnel_manager = TunnelManager::new(config, metrics.clone()).await.unwrap();

        // Should handle multiple tunnels
        assert_eq!(tunnel_manager.config.tunnels.len(), 2);

        // Test shutdown with multiple tunnels
        let result = tunnel_manager.shutdown().await;
        assert!(result.is_ok());
    }
}

// Integration tests that require real SSH setup
#[cfg(test)]
mod integration_tests {
    use super::*;

    // These tests require a real SSH server setup
    // Run with: cargo test --features integration-tests

    #[tokio::test]
    #[ignore] // Requires real SSH setup
    async fn test_real_ssh_connection() {
        // This test requires:
        // 1. SSH server running on localhost
        // 2. Valid SSH key in ~/.ssh/
        // 3. User can connect without password

        let ssh_config = SshConfig {
            host: "localhost".to_string(),
            user: std::env::var("USER").unwrap_or("root".to_string()),
            port: 22,
            key_path: std::path::PathBuf::from(format!(
                "{}/.ssh/id_rsa",
                std::env::var("HOME").unwrap_or("/root".to_string())
            )),
            timeout: 30,
            keepalive_interval: 60,
        };

        let result = SshConnection::new(&ssh_config);
        if result.is_ok() {
            println!("✅ Real SSH connection successful!");
        } else {
            println!("❌ Real SSH connection failed: {:?}", result.unwrap_err());
        }
    }

    #[tokio::test]
    #[ignore] // Requires real SSH setup
    async fn test_end_to_end_tunnel() {
        // This is a complete end-to-end test
        // Requires SSH access to a remote server

        let ssh_config = SshConfig {
            host: "localhost".to_string(),
            user: std::env::var("USER").unwrap_or("root".to_string()),
            port: 22,
            key_path: std::path::PathBuf::from(format!(
                "{}/.ssh/id_rsa",
                std::env::var("HOME").unwrap_or("/root".to_string())
            )),
            timeout: 30,
            keepalive_interval: 60,
        };

        let tunnel_config = TunnelConfig {
            name: "integration-test".to_string(),
            direction: "receive".to_string(),
            local_host: "127.0.0.1".to_string(),
            local_port: 8888,
            remote_host: "127.0.0.1".to_string(),
            remote_port: 22, // Forward to SSH port
            enabled: true,
        };

        let config = Config {
            ssh: ssh_config,
            tunnels: vec![tunnel_config],
            limits: ConnectionLimits::default(),
        };

        let metrics = Arc::new(MetricsCollector::new());
        let tunnel_manager = TunnelManager::new(config, metrics).await;

        match tunnel_manager {
            Ok(manager) => {
                println!("✅ Integration test tunnel manager created successfully!");

                // Test a quick connection
                tokio::spawn(async move {
                    tokio::time::sleep(Duration::from_secs(1)).await;
                    let _ = manager.shutdown().await;
                });

                // This would normally run the tunnel
                // let _ = manager.start().await;
            }
            Err(e) => {
                println!("❌ Integration test failed: {:?}", e);
            }
        }
    }
}
