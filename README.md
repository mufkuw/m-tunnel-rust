# M-Tunnel Rust — Simple, fast SSH tunneling

M-Tunnel Rust is a small, focused SSH tunnel manager written in Rust. It supports both a native SSH2-based implementation and a traditional SSH CLI mode. The project is designed for reliability, secure defaults, and easy integration into automation or devops workflows.

If you find this project useful, a star or a fork helps a lot. Contributions and bug reports are very welcome.

## Quick highlights

- Native SSH2 implementation (no external ssh binary required)
- Multiple tunnels and directions (forward/reverse)
- TOML configuration with sensible defaults and SSH key checks
- Async runtime (Tokio) for performance and responsiveness
- Built-in retry/backoff and connection throttling

## Quick start

Clone, build, and run a dry-run to verify your configuration:

```bash
git clone https://github.com/mufkuw/m-tunnel-rust.git
cd m-tunnel-rust
cargo build --release

# Dry-run (no network connections) - recommended for first run
./target/release/m-tunnel-rust --ssh2 --config configs/real_ssh_test.toml --dry-run
```

Install (optional):

```bash
sudo cp target/release/m-tunnel-rust /usr/local/bin/
```

## Configuration

Configure tunnels using a TOML file. A minimal example:

```toml
[ssh]
host = "example.com"
user = "username"
port = 22
key_path = "~/.ssh/id_rsa"

[[tunnels]]
name = "web"
direction = "receive"
local_host = "127.0.0.1"
local_port = 8080
remote_host = "internal.web"
remote_port = 80
enabled = true
```

See `configs/` and `docs/` for example files and a full explanation of available options.

## Tests & quick verification

There are a set of helper scripts under `tests/` to validate the build and basic runtime behaviour.

Run the quick test suite (takes about 30s):

```bash
cd tests
chmod +x ./test_quick.sh
./test_quick.sh
```

For development, run unit tests with:

```bash
cargo test
```

And check for lints and warnings:

```bash
cargo clippy --all-targets --all-features -- -D warnings
cargo fmt --all -- --check
```

## Development notes

- The codebase targets the 2021 Rust edition and uses Tokio for async.
- Feature flags in `Cargo.toml` enable optional components; for example `--features ssh2` enables the native SSH2 library.
- The `tests/` directory contains several helper scripts used during development and CI.

## Contributing

Contributions are welcome. If you'd like to help:

1. Star or fork the repo
2. Open an issue to discuss larger changes
3. Create a branch for your work (`git checkout -b feature/your-change`)
4. Run tests and linters before opening a PR
5. Open a pull request with a clear description and small, focused changes

## License

This project is now distributed under the GNU Affero General Public License v3 (AGPLv3). See the `LICENSE` file for full details.

## Support / Buy me a coffee

If you'd like to support ongoing development, here are simple ways to accept donations or 'buy a coffee':

- GitHub Sponsors — create a sponsor profile and add a badge/link to your README
- Ko-fi / Buy Me a Coffee — create an account and add a link or badge
- PayPal.me — add a PayPal.me link for direct tips (example: `https://paypal.me/mufkuw`)
- Open Collective — for recurring donations and transparent budgeting

Example donation blurb you can add to your README:

> If you find M-Tunnel Rust useful, consider buying me a coffee: https://paypal.me/mufkuw (PayPal handle: `@mufkuw`) — any support helps keep maintenance and improvements going.

Optional PayPal button (markdown):

```markdown
[![Donate](https://img.shields.io/badge/Donate-PayPal-blue?logo=paypal)](https://paypal.me/mufkuw)
```

If you want, I can add a badge and a short instructions block (Ko-fi / PayPal / GitHub Sponsors) to the README and show how to include it in project metadata.

---

Thanks for checking out M-Tunnel Rust. If you have questions or want help integrating the tool into your environment, open an issue or a PR — happy to help.
