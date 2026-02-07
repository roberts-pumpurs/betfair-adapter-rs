# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Build & Development Commands

All development tasks are run through `cargo xtask`:

```bash
cargo xtask test              # Run tests with cargo-nextest
cargo xtask test --coverage   # Run tests with grcov coverage report
cargo xtask check             # Clippy (with -D warnings) + rustfmt verification
cargo xtask fmt               # Auto-format + clippy --fix
cargo xtask deny              # Dependency security/license audit (cargo-deny)
cargo xtask typos             # Spell check with typos-cli
cargo xtask typos --write     # Auto-fix typos
cargo xtask unused-deps       # Detect unused dependencies (cargo-machete)
cargo xtask doc               # Generate rustdoc
```

Run a single test: `cargo nextest run -p <crate-name> <test_name>`

Run tests for a specific crate: `cargo nextest run -p betfair-stream-types`

## Project Architecture

Rust workspace (edition 2024, MSRV 1.91) with 8 crates under `crates/`:

### Crate dependency graph

```
betfair-stream-api → betfair-adapter → betfair-types → (generated at build time)
                   → betfair-stream-types → betfair-types
                                                ↑
                                          betfair-typegen → betfair-xml-parser
```

- **betfair-adapter**: High-level async RPC client. `BetfairRpcClient<T>` is generic over auth state (`Unauthenticated`/`Authenticated`) — the type system prevents calling authenticated endpoints before login.
- **betfair-types**: API-NG request/response types. **Code-generated at build time** — `build.rs` reads XML files from `crates/betfair-typegen/assets/` (SportsAPING.xml, AccountAPING.xml, HeartbeatAPING.xml), parses them with `betfair-xml-parser`, generates Rust via `betfair-typegen`, and writes to `$OUT_DIR/mod.rs`. Do not edit the generated types directly.
- **betfair-stream-api**: Streaming API client with TLS, automatic reconnection (exponential backoff), and built-in caching. Uses `MessageProcessor` trait for pluggable message handling (`Cache` for stateful caching, `Forwarder` for raw passthrough). Protocol is line-delimited JSON (CRLF terminated) over TLS.
- **betfair-stream-types**: Streaming message types (request/response serialization).
- **betfair-xml-parser**: Deserializes Betfair WSDL XML into Rust structs consumed by typegen.
- **betfair-typegen**: Code generation engine that transforms parsed XML AST into Rust `TokenStream`.
- **betfair-cert-gen**: Generates self-signed TLS certificates for Betfair bot login.
- **betfair-rpc-server-mock**: WireMock-based mock server for integration testing without Betfair credentials.

### Key patterns

- **Numeric types**: `Price` and `Size` newtypes wrap f64. `F64Ord` provides Eq/Ord/Hash for f64. Use macros `num!()`, `num_ord!()`, `num_u8!()` to construct values.
- **Secrets**: Credentials use `redact::Secret<T>` for automatic redaction in Debug/Display output.
- **URLs**: `BetfairUrl<T>` uses marker types (`RestBase`, `Stream`, `BotLogin`, etc.) for type-safe URL handling.
- **Streaming cache**: `cache/` module maintains `MarketBookCache` and `OrderBookCache` with incremental updates from the streaming API, tracked by `StreamState`.

## Conventions

- Conventional commits required (enforced by CI).
- Workspace lints: `rust_2018_idioms = "deny"`, `unused_must_use = "deny"`.
- rustfmt config: `reorder_imports = true`, `use_field_init_shorthand = true`.
- CI runs: clippy, fmt check, nextest, cargo-deny, typos, cargo-machete, and doc generation.
