# M-Tunnel Migration Guide

## Overview

M-Tunnel now supports two configuration formats:

1. **Legacy format**: `.env` + `m-tunnel.conf` (backward compatible)
2. **New TOML format**: `config.toml` (recommended)

## New Features

- Structured configuration with validation
- Individual tunnel enable/disable
- Connection limits and timeouts
- Better error handling and logging
- Optional metrics endpoint
- Comprehensive testing

## Migration Steps

### 1. Keep Using Legacy Format (No Changes Required)

Your existing `.env` and `m-tunnel.conf` files will continue to work without any changes.

### 2. Migrate to TOML Format (Recommended)

#### Current `.env` file:

```bash
SSH_HOST=example.com
SSH_PORT=22
SSH_USER=root
SSH_PRIVATE_KEY=m-tunnel.key
```

#### Current `m-tunnel.conf` file:

```
send -- 127.0.0.1:22 to 192.168.80.12:20001
receive -- 0.0.0.0:8080 from 10.0.0.5:80
```

#### New `config.toml` file:

```toml
[ssh]
host = "example.com"
user = "root"
port = 22
key_path = "m-tunnel.key"
timeout = 30
keepalive_interval = 60

[limits]
max_attempts = 5
retry_window_secs = 300
max_backoff_secs = 60

[[tunnels]]
name = "ssh-forward"
direction = "send"
local_host = "127.0.0.1"
local_port = 22
remote_host = "192.168.80.12"
remote_port = 20001
enabled = true

[[tunnels]]
name = "web-reverse"
direction = "receive"
local_host = "0.0.0.0"
local_port = 8080
remote_host = "10.0.0.5"
remote_port = 80
enabled = true
```

## New Environment Variables

### Metrics (Optional)

```bash
export METRICS_PORT=9090
```

Access metrics at: `http://localhost:9090/metrics`

### Log Level

```bash
export RUST_LOG=info  # debug, info, warn, error
```

## Security Improvements

1. **SSH Key Validation**: Checks file permissions (600/400)
2. **Input Sanitization**: Prevents command injection
3. **Connection Limits**: Rate limiting for failed connections
4. **Structured Logging**: Better audit trail

## Benefits of TOML Format

1. **Individual Control**: Enable/disable tunnels separately
2. **Named Tunnels**: Better identification in logs and metrics
3. **Validation**: Configuration errors caught at startup
4. **Extensibility**: Easy to add new features
5. **Type Safety**: Proper validation of ports, timeouts, etc.

## Testing Your Configuration

```bash
# Test with current config
cargo run --release

# Test specific features
METRICS_PORT=9090 cargo run --release
RUST_LOG=debug cargo run --release
```

## Backward Compatibility

- All existing configurations continue to work
- No breaking changes to command line interface
- Same systemd service file works
- Same installation paths supported

## Troubleshooting

### SSH Key Issues

```
Error: SSH key has insecure permissions: 644. Should be 600 or 400
```

**Solution**: `chmod 600 /path/to/key`

### Configuration Errors

```
Error: Failed to parse TOML configuration
```

**Solution**: Check TOML syntax, validate with online TOML validator

### Migration Questions

- Keep both formats during transition
- Test new format with `config.toml` alongside existing files
- The system automatically prefers TOML if available

## Support

For issues or questions:

1. Check the logs with `RUST_LOG=debug`
2. Validate configuration syntax
3. Test SSH connectivity manually
4. Review file permissions
