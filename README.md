# Load Balancer Template

This project demonstrates how to build a simple HTTP load balancer in Rust using the latest `hyper`, `tokio`, and `tower` crates. It also includes a lightweight worker binary that can be used to spin up mock backend servers for manual testing.

## Prerequisites

- Rust toolchain (stable) – install via [rustup](https://rustup.rs/)

## Running the Worker Servers

Build and run a worker on port `3000`:

```bash
cargo run --bin worker -- 3000
```

Start another worker on a different port, e.g. `3001`:

```bash
cargo run --bin worker -- 3001
```

Each worker echoes the HTTP method and path it receives.

## Running the Load Balancer

With two workers running, start the load balancer:

```bash
cargo run
```

By default the balancer listens on `http://127.0.0.1:1337` and forwards requests to the configured workers in a round-robin rotation.

Send any HTTP requests to the balancer port to watch them cycle between workers:

```bash
curl http://127.0.0.1:1337/test
```

## Customising Worker Hosts

Edit `src/main.rs` if you want to change the list of worker endpoints or the listening address. The balancer currently targets `http://localhost:3000` and `http://localhost:3001`.
