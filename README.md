# libp2p-quic-multi-runtime

TLS-based QUIC transport for [libp2p](https://libp2p.io/), implemented on top
of [quinn](https://github.com/quinn-rs/quinn).

## Fork notice

This repository is a **standalone fork** of the `libp2p-quic` crate extracted from the
upstream [rust-libp2p](https://github.com/libp2p/rust-libp2p) monorepo (`transports/quic`).

Upstream source:

- https://github.com/libp2p/rust-libp2p/tree/master/transports/quic

This fork is maintained independently for easier publishing and development outside the workspace. It includes
additional runtime support (for example `smol`) that may differ from upstream.

Original code is licensed under the MIT License (see `LICENSE`).

## Features

| Feature | Description                                        |
|---------|----------------------------------------------------|
| `tokio` | Use the Tokio runtime via `quic::tokio::Transport` |
| `smol`  | Use the smol runtime via `quic::smol::Transport`   |

## Usage

Add to `Cargo.toml`:

```toml
[dependencies]
libp2p-quic-multi-runtime = { version = "0.14", features = ["tokio"] }
```

Tokio example:

```rust
use libp2p_core::{Multiaddr, Transport, transport::ListenerId};
use libp2p_quic_multi_runtime as quic;

#[tokio::main]
async fn main() -> std::io::Result<()> {
    let keypair = libp2p_identity::Keypair::generate_ed25519();
    let config = quic::Config::new(&keypair);
    let mut transport = quic::tokio::Transport::new(config);

    transport.listen_on(
        ListenerId::next(),
        "/ip4/127.0.0.1/udp/0/quic-v1".parse().unwrap(),
    )?;

    Ok(())
}
```

Smol example:

```toml
libp2p-quic-multi-runtime = { version = "0.14", features = ["smol"] }
```

```rust
use libp2p_core::{Transport, transport::ListenerId};
use libp2p_quic_multi_runtime as quic;

fn main() -> std::io::Result<()> {
    smol::block_on(async {
        let keypair = libp2p_identity::Keypair::generate_ed25519();
        let config = quic::Config::new(&keypair);
        let mut transport = quic::smol::Transport::new(config);

        transport.listen_on(
            ListenerId::next(),
            "/ip4/127.0.0.1/udp/0/quic-v1".parse().unwrap(),
        )?;

        Ok(())
    })
}
```

## QUIC in libp2p

QUIC combines transport, encryption, and multiplexing. Connections do not need a separate security or muxer upgrade;
configure TLS via `quic::Config::new`.

## Development

```bash
# Tokio
cargo test --features tokio

# smol
cargo test --features smol
```

## Publishing

This crate is intended to be published independently to crates.io.

1. Update `repository` (and optionally `homepage`) in `Cargo.toml`.
2. Commit `Cargo.lock` for reproducible builds.
3. Run `cargo publish`.

Ensure the `name` field in `Cargo.toml` matches the published crate name (`libp2p-quic-multi-runtime`).
