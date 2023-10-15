# Betfari Adapter

[![check](https://github.com/roberts-ivanovs/betfair-adapter-rs/actions/workflows/check.yaml/badge.svg)](https://github.com/roberts-ivanovs/betfair-adapter-rs/actions/workflows/check.yaml)
[![docs](https://github.com/roberts-ivanovs/betfair-adapter-rs/actions/workflows/doc.yaml/badge.svg)](https://github.com/roberts-ivanovs/betfair-adapter-rs/actions/workflows/doc.yaml)
[![msrv](https://github.com/roberts-ivanovs/betfair-adapter-rs/actions/workflows/msrv.yaml/badge.svg)](https://github.com/roberts-ivanovs/betfair-adapter-rs/actions/workflows/msrv.yaml)
[![test](https://github.com/roberts-ivanovs/betfair-adapter-rs/actions/workflows/test.yaml/badge.svg)](https://github.com/roberts-ivanovs/betfair-adapter-rs/actions/workflows/test.yaml)
[![unused-deps](https://github.com/roberts-ivanovs/betfair-adapter-rs/actions/workflows/unused-deps.yaml/badge.svg)](https://github.com/roberts-ivanovs/betfair-adapter-rs/actions/workflows/unused-deps.yaml)
[![deny](https://github.com/roberts-ivanovs/betfair-adapter-rs/actions/workflows/deny.yaml/badge.svg)](https://github.com/roberts-ivanovs/betfair-adapter-rs/actions/workflows/deny.yaml)
[![audit](https://github.com/roberts-ivanovs/betfair-adapter-rs/actions/workflows/audit.yaml/badge.svg)](https://github.com/roberts-ivanovs/betfair-adapter-rs/actions/workflows/audit.yaml)

https://docs.developer.betfair.com/display/1smk3cen4v3lu3yomq5qye0ni/Sample+Code

## Development setup

```bash
# Install rustup
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Install cargo-make
cargo install cargo-make

# Test the code
cargo make test

# Format the code
cargo make format

# Check the code
cargo make check

# Perform all of the CI checks
cargo make local-ci
```

## Betfair API-NG interface

<!-- TODO docs go here -->

## Betfair Stream API interface

<!-- TODO docs go here -->
