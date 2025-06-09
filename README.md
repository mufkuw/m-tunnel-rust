# m-tunnel

A lightweight and production-ready SSH tunnel utility in Rust.

## Features

- Forward (local → remote) and reverse (remote → local) tunnels
- Configurable via `.env` and `m-tunnel.conf`
- Auto-reconnect on SSH drop
- Graceful shutdown on Ctrl+C
- Secure by default (local-only bindings)

## Setup

1. Copy your SSH private key
2. Create `.env`:

SSH_HOST=example.com
SSH_PORT=22
SSH_USER=root
SSH_PRIVATE_KEY=path/to/key.pem

markdown
Copy
Edit

3. Create `m-tunnel.conf`:

send -- 0.0.0.0:2222 to 127.0.0.1:22
receive -- 127.0.0.1:8080 to 10.0.0.5:80

arduino
Copy
Edit

4. Run:

```bash
cargo run --release