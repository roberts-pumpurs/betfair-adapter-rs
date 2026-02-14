# Stream Performance: Benchmarks & Optimizations

**Date:** 2026-02-14
**Goal:** Minimize per-message latency in the streaming pipeline (deserialize → cache update → output).
**Approach:** Benchmarks first, then data-driven optimizations. Breaking API changes allowed (semver major).
**Framework:** criterion with HTML reports.

## Analysis Summary

The streaming hot path processes line-delimited JSON over TLS into an in-memory cache. Five critical bottlenecks were identified:

| Priority | Issue | Location | Impact |
|----------|-------|----------|--------|
| P0 | Deep clone of entire cache state on every update | `lib.rs:112,117` | ~50KB cloned per market update |
| P1 | Custom deserializer parses JSON twice via `Value` intermediate | `response.rs:148-197` | 30-40% deser overhead |
| P2 | No HashMap capacity hints for runners | `market_book_cache.rs:37` | 4-5 reallocations per market |
| P3 | RunnerDefinition cloned per runner during definition update | `market_book_cache.rs:144,146` | 1-2KB per runner |
| P4 | Order struct fully cloned into unmatched orders HashMap | `orderbook_runner_cache.rs:51` | Full Order clone per update |

## Benchmark Infrastructure

- **Location:** `crates/betfair-stream-api/benches/`
- **xtask:** `cargo xtask bench` command
- **Fixtures:** Existing JSON fixtures + production stream recordings (1.9MB, 11K lines)

## Benchmark Suites

### `deserialize.rs` — JSON bytes to Rust types

| Benchmark | Input | Measures |
|-----------|-------|---------|
| `deser_market_change_delta` | Inline ~200B JSON | Per-tick deser cost |
| `deser_market_change_image` | `streaming_mcm_SUB_IMAGE.json` | Full image parse |
| `deser_market_change_large_image` | `streaming_mcm_SUB_IMAGE_no_market_def.json` (182KB) | Worst-case deser |
| `deser_order_change` | `streaming_ocm_FULL_IMAGE.json` | Order message parse |

### `cache_update.rs` — Parsed message to cache mutation

| Benchmark | Setup | Measures |
|-----------|-------|---------|
| `cache_update_delta` | Pre-populated cache + single runner delta | Steady-state update |
| `cache_update_full_image` | Empty cache + full market image | Cold-start cost |
| `cache_update_market_definition` | Pre-populated cache + definition change | RunnerDefinition clone cost |

### `process_message.rs` — Full pipeline including output

| Benchmark | Setup | Measures |
|-----------|-------|---------|
| `process_message_delta` | Warm Cache processor + delta | End-to-end per-tick latency |
| `process_message_image` | Cache processor + full image | Worst-case output cost |
| `cache_clone_isolated` | 10 runners, 5 levels each | Isolated deep-clone cost |

## Optimization Plan (post-baseline)

### P0: Arc-wrap CachedMessage output
- Store `Arc<MarketBookCache>` in tracker, emit `Vec<Arc<MarketBookCache>>` in `CachedMessage`
- Eliminates ~50KB deep clone per update → ~8 bytes (Arc increment)
- Breaking API change

### P1: Rewrite DatasetChangeMessage deserializer
- Replace `Value::deserialize()` + clone + re-parse with proper `serde::de::Visitor`
- Eliminates intermediate Value tree and 3 `.clone()` calls per message

### P2: HashMap capacity hints
- `HashMap::with_capacity(16)` for runner maps
- Use `runners.len()` when market definition available

### P3: Reduce RunnerDefinition cloning
- Store `Arc<RunnerDefinition>` shared between market definition and runners
- Or iterate by index to avoid clone

### P4: Order update ownership transfer
- Change `update_unmatched` to take owned `Order` instead of `&Order`
- Move into HashMap instead of cloning

## Out of Scope

- Zero-copy deser (`Cow<str>`, `#[serde(borrow)]`) — pervasive lifetime changes, high risk
- `BTreeMap` → sorted `Vec` for `Available<T>` — measure first
- `simd-json` — measure serde_json baseline first
