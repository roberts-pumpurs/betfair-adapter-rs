# Benchmark Expansion Design

## Context

The workspace currently has 10 benchmarks, all in `betfair-stream-api`, covering deserialization (4), cache updates (3), and message processing pipeline (3). These miss several performance-critical areas: numeric type construction, request serialization, codec encoding, and RPC round-trip latency.

## Approach

**Approach A — per-crate benchmarks.** Each crate owns its benchmarks next to the code it tests. This follows Rust conventions and keeps crate boundaries clean.

## New Benchmarks

### 1. `betfair-types/benches/numeric.rs` — Numeric primitives

| Benchmark | Operation | Why it matters |
|---|---|---|
| `price_new_low_range` | `Price::new()` with prices in 1.01–2.0 | Finest granularity (0.01 increment), most branch logic |
| `price_new_mid_range` | `Price::new()` with prices in 6.0–10.0 | Mid-range boundary adjustment (0.2 increment) |
| `price_new_high_range` | `Price::new()` with prices in 100–1000 | Coarsest range (10.0 increment), last match arm |
| `size_new` | `Size::new()` construction | `round_2dp()` called on every Size creation |
| `f64ord_hash` | `F64Ord` hashing via `to_bits()` | HashMap key in cache — called on every runner lookup |
| `f64ord_btreemap_lookup` | BTreeMap lookup with F64Ord keys | Simulates `Available` cache ladder lookups (~50 entries) |

**Implementation notes:**
- Price benches use arrays of representative prices per range to avoid branch-prediction bias.
- F64Ord btreemap bench pre-populates with ~50 entries (typical ladder depth).
- Add `criterion` as dev-dependency to `betfair-types/Cargo.toml`.

### 2. `betfair-stream-types/benches/serialize.rs` — Request serialization

| Benchmark | Operation | Why it matters |
|---|---|---|
| `ser_market_subscription` | `serde_json::to_string` for MarketSubscription | Largest request type (filter, data fields, conflation) |
| `ser_order_subscription` | `serde_json::to_string` for OrderSubscription | Order-specific subscription |
| `ser_authentication` | `serde_json::to_string` for Authentication | Auth request with session/app key |
| `ser_heartbeat` | `serde_json::to_string` for Heartbeat | Minimal message — baseline serialization cost |

**Implementation notes:**
- Construct each `RequestMessage` variant once in setup with realistic fields.
- Use `black_box()` on output to prevent dead-code elimination.
- Add `criterion` as dev-dependency to `betfair-stream-types/Cargo.toml`.

### 3. `betfair-stream-api/benches/codec.rs` — Codec encode path

| Benchmark | Operation | Why it matters |
|---|---|---|
| `codec_encode_market_subscription` | `StreamAPIClientCodec::encode()` | Full encode: serde serialization + CRLF framing into BytesMut |
| `codec_encode_heartbeat` | `StreamAPIClientCodec::encode()` | Minimal message — shows framing overhead baseline |

**Implementation notes:**
- Use `iter_batched` with `SmallInput` to provide fresh `BytesMut` each iteration (encode appends).
- Add new `[[bench]]` entry to existing `betfair-stream-api/Cargo.toml`.

### 4. `betfair-adapter/benches/rpc.rs` — RPC round-trip with mock server

| Benchmark | Operation | Why it matters |
|---|---|---|
| `rpc_send_request_list_market_book` | Full `send_request()` with WireMock | End-to-end: JSON serialize → HTTP POST → deserialize response |
| `rpc_send_request_list_market_catalogue` | Full `send_request()` with WireMock | Different response shape/size |
| `rpc_bot_login` | `bot_log_in()` authentication flow | Auth round-trip from unauthenticated state |

**Implementation notes:**
- Use tokio runtime with `criterion::async_executor::AsyncExecutor` or `Runtime::block_on`.
- Spin up one `Server` instance per benchmark group; mock server stays alive for all iterations.
- Mount realistic mock responses using `Server::mock_authenticated_rpc` helpers.
- `bot_log_in` creates a fresh `Unauthenticated` client per iteration.
- Intentionally includes localhost TCP overhead — measures full stack including reqwest connection pool.
- Add `criterion`, `tokio`, `betfair-rpc-server-mock`, `serde_json` as dev-dependencies to `betfair-adapter/Cargo.toml`.

## xtask Update

Update `Args::Bench` handler to iterate over all crates with benchmarks:

```rust
let benches = [
    ("betfair-types", "numeric"),
    ("betfair-stream-types", "serialize"),
    ("betfair-stream-api", "deserialize"),
    ("betfair-stream-api", "cache_update"),
    ("betfair-stream-api", "process_message"),
    ("betfair-stream-api", "codec"),
    ("betfair-adapter", "rpc"),
];
```

CI workflow (`bench.yaml`) requires no changes — it calls `cargo xtask bench`.

## Summary

This adds 16 new benchmarks across 4 crates, expanding coverage from streaming-only to the full hot path: numeric construction, request serialization, codec framing, and async RPC round-trips.
