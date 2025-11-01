# betfair-adapter-rs

[![Crates.io](https://img.shields.io/crates/v/betfair-adapter.svg)](https://crates.io/crates/betfair-adapter) [![docs.rs](https://docs.rs/betfair-adapter/badge.svg)](https://docs.rs/betfair-adapter)
[![check](https://github.com/roberts-pumpurs/betfair-adapter-rs/actions/workflows/check.yaml/badge.svg)](https://github.com/roberts-pumpurs/betfair-adapter-rs/actions/workflows/check.yaml) [![docs](https://github.com/roberts-pumpurs/betfair-adapter-rs/actions/workflows/doc.yaml/badge.svg)](https://github.com/roberts-pumpurs/betfair-adapter-rs/actions/workflows/doc.yaml) [![test](https://github.com/roberts-pumpurs/betfair-adapter-rs/actions/workflows/test.yaml/badge.svg)](https://github.com/roberts-pumpurs/betfair-adapter-rs/actions/workflows/test.yaml) [![unused-deps](https://github.com/roberts-pumpurs/betfair-adapter-rs/actions/workflows/unused-deps.yaml/badge.svg)](https://github.com/roberts-pumpurs/betfair-adapter-rs/actions/workflows/unused-deps.yaml) [![deny](https://github.com/roberts-pumpurs/betfair-adapter-rs/actions/workflows/deny.yaml/badge.svg)](https://github.com/roberts-pumpurs/betfair-adapter-rs/actions/workflows/deny.yaml)

Utilities for interacting with the Betfair API (API‑NG and Streaming) from Rust.

## Features

- High‑level API‑NG client (`betfair-adapter`) with typed requests and responses
- API‑NG models in `betfair-types`
- Streaming API client with internal caching (`betfair-stream-api` and `betfair-stream-types`)
- Low‑level XML parser for Betfair messages (`betfair-xml-parser`)
- Mock RPC server for testing (`betfair-rpc-server-mock`)
- Code generation for API definitions (`betfair-typegen`)
- TLS certificate generation for streaming (`betfair-cert-gen`)
- Examples and `xtask` automation for development workflows

## Crate Overview

- **betfair-adapter**: High‑level async client for API‑NG
- **betfair-types**: API‑NG request/response types
- **betfair-stream-types**: Streaming API message types
- **betfair-stream-api**: Streaming client, cache, and trackers
- **betfair-xml-parser**: XML parser utilities
- **betfair-rpc-server-mock**: Mock Betfair RPC server for integration tests
- **betfair-typegen**: Code generator for API definitions
- **betfair-cert-gen**: Generate TLS certificates for streaming connections

See the [`examples/`](./examples) directory for complete guides.

## Feature Flags

### `decimal-primitives`

Enabling this feature uses `rust_decimal::Decimal` for many numeric values including `Price`, `Size`, and some other
fields (handicaps, market rates, etc.).
Using `Decimal` provides precise arithmetic, however it can be slower compared to f64 floating-point operations.

#### Creating values from literals:

Use the `num!` macro for cross-compatibility when creating numeric literals. The following will compile
whether the `decimal-primitives` feature is enabled or disabled.

```rust
use betfair_types::num;

let price = Price::new(num!(1.5))?;
let size = Size::from(num!(100.0));
```

There is also a `num_ord!` macro for creating either `Decimal` or `F64Ord`, and `num_u8!` for creating either `Decimal` or `u8`,
depending on whether the `decimal-primitives` feature is enabled or disabled.

## Development

Clone the repository and explore available tasks:

```bash
git clone https://github.com/roberts-pumpurs/betfair-adapter-rs.git
cd betfair-adapter-rs
cargo xtask --help
```

Run tests, lint checks, and formatters:

```bash
cargo xtask test
cargo xtask clippy
cargo xtask fmt
```

## Contributing

Please read [CONTRIBUTING.md](CONTRIBUTING.md) and [CODE_OF_CONDUCT.md](CODE_OF_CONDUCT.md) for guidelines.

## License

Licensed under the MIT OR Apache-2.0 license at your choice:

- [LICENSE-MIT](./LILICENSE-MIT)
- [LICENSE-APACHE](./LICENSE-APACHE)
