# Stream Performance Round 2: Deep Optimizations

**Date:** 2026-02-15
**Goal:** Eliminate remaining allocations and clones across the entire streaming pipeline.
**Approach:** Data-driven, benchmarked. Breaking API changes allowed.
**Builds on:** Round 1 (Arc-wrap, deserializer rewrite, capacity hints, order ownership).

## Current Baseline (post Round 1)

| Benchmark | Time |
|-----------|------|
| `deser_market_change_delta` | 1.16 µs |
| `deser_market_change_image` | 11.38 µs |
| `deser_market_change_large_image` | 1.09 ms |
| `deser_order_change` | 12.64 µs |
| `cache_update_delta` | 818 ns |
| `cache_update_full_image` | 1.16 µs |
| `cache_update_market_definition` | 1.11 µs |
| `process_message_delta` | 837 ns |
| `process_message_image` | 1.17 µs |
| `cache_clone_isolated` | 10.9 ns |

## String Type Evaluation

Six alternatives were evaluated for replacing `String` in high-frequency deserialization paths:

| Type | Stack | Inline capacity | Clone | serde | Allocates <20 chars |
|------|-------|-----------------|-------|-------|---------------------|
| `String` (current) | 24B | 0 | O(n) | native | Yes |
| `Arc<str>` | 16B | 0 | O(1) | `serde/rc` feature | Yes |
| `Cow<'a, str>` | 24B | borrowed | O(1) borrowed | `#[serde(borrow)]` | Maybe (zero-copy) |
| `SmolStr` | 24B | 23B | O(1) | `serde` feature | No |
| `CompactString` | 24B | 24B | O(1) | `serde` feature | No |
| `EcoString` | 16B | 15B | O(1) | yes | 16-20 chars: Yes |

**Decision: `CompactString`** for all stream-types string fields.

Rationale:
- 24 bytes inline covers all clock tokens (10-20 chars), country codes (2-3 chars), timezone names (~20 chars), event IDs, market types, etc. without heap allocation.
- O(1) clone for both inline and heap cases (uses Arc internally for heap).
- Same stack size as `String` so no layout changes.
- `serde` feature provides drop-in `Deserialize` support.
- `Cow<'a, str>` was rejected: requires lifetime parameter on all structs (pervasive change), breaks the existing cache storage pattern, and falls back to allocation on JSON escape sequences.
- `SmolStr` was close but 23B vs 24B inline is slightly worse and CompactString has a larger ecosystem.
- `EcoString` was rejected: only 15B inline means clock tokens (often 12-20 chars) would sometimes spill.

## Optimization Plan

### O1: CompactString for stream types (Deserialization)

**Impact:** ~10-15% deserialization improvement, eliminates heap allocations for all strings <24 chars.

Replace `String` with `CompactString` across all `betfair-stream-types` response types:
- `Clock(String)` and `InitialClock(String)` newtypes
- `MarketDefinition`: `venue`, `timezone`, `country_code`, `race_type`, `market_type`, `event_id`, `event_type_id`, `settled_time`, `suspend_time`, `open_date`, `market_time`
- `MarketDefinition.regulators: Vec<String>`
- `RunnerDefinition.removal_date: Option<String>`
- `Order`: `regulator_code`, `regulator_auth_code`, `lapse_status_reason_code`

Also propagate to cache types that store these values (MarketId, BetId, etc. if they're String-based).

### O2: Arc-wrap cold fields in MarketBookCache (Cache clone mitigation)

**Impact:** ~30-50% reduction in Arc::make_mut deep clone cost.

When `Arc::make_mut` triggers (consumer holds Arc refs from previous messages), the **entire** MarketBookCache is deep-cloned. Structure analysis:

| Field | Size (20 runners) | Update frequency |
|-------|-------------------|------------------|
| `market_definition` | ~3.6KB | Rare (SUB_IMAGE only) |
| `runners` HashMap | ~16-42KB | Every delta |
| `runner.definition` | ~100B each | Rare |
| `runner.price_ladders` (7 BTreeMaps) | ~1-4KB each | Every delta |
| Metadata (id, time, active) | ~50B | Every delta |

Wrap cold fields in inner `Arc`:
- `market_definition: Option<Arc<MarketDefinition>>` (was `Option<Box<MarketDefinition>>`)
- `runner.definition: Option<Arc<RunnerDefinition>>` (was `Option<RunnerDefinition>`)

When `Arc::make_mut` triggers a deep clone, these inner Arcs are cloned as 8-byte pointer increments instead of copying ~3.6KB of definition data. The hot runner data (price ladders, metadata) still needs to be cloned, but the cold definition data (~4KB per market, ~100B per runner) is shared.

### O3: SmallVec for RunnerChange price level vectors (Deserialization)

**Impact:** ~15-25% deserialization improvement for delta messages.

`RunnerChange` has 10 `Option<Vec<UpdateSet2/3>>` fields. In typical deltas, only 1-3 are populated with 1-5 entries each. Each populated field currently heap-allocates a `Vec`.

Replace with `SmallVec<[T; 4]>`:
- `best_available_to_back: Option<SmallVec<[UpdateSet3; 4]>>`
- `available_to_back: Option<SmallVec<[UpdateSet2; 4]>>`
- etc. for all 10 vector fields

4 inline entries covers 80%+ of delta messages without heap allocation. SmallVec has serde support via the `serde` feature.

Similarly for `OrderRunnerChange`:
- `matched_backs: Option<SmallVec<[UpdateSet2; 4]>>`
- `matched_lays: Option<SmallVec<[UpdateSet2; 4]>>`
- `unmatched_orders: Option<SmallVec<[Order; 2]>>`

### O4: Sorted Vec for Available\<T\> price ladders (Cache update)

**Impact:** ~10-20% cache update improvement for price level operations.

`Available<T>` currently uses `BTreeMap<K, V>` for storing price levels. Typical depth is 5-20 entries. For small sorted collections, a sorted `Vec` has much better cache locality.

Replace `BTreeMap` with a sorted `Vec<(K, V)>` using binary search for lookups. Use `SmallVec<[(K, V); 16]>` to keep typical ladders inline.

The `UpdateSet` trait's `update` method already receives sorted data from Betfair. The update pattern (insert or replace at key, remove if value is zero) maps well to binary search + insert.

### O5: Remaining HashMap capacity hints (Cache update)

**Impact:** ~5-10% reduction in HashMap rehashing.

Missing capacity hints:
- `OrderBookRunner.unmatched_orders: HashMap::with_capacity(16)`
- `OrderBookRunner.strategy_matches: HashMap::with_capacity(4)`
- `MarketStreamTracker.market_state: HashMap::with_capacity(64)`
- `OrderStreamTracker.market_state: HashMap::with_capacity(64)`

### O6: MarketId clone reduction in entry pattern (Cache update)

**Impact:** ~5% per market update.

Current pattern clones MarketId 2-3 times per market in the tracker entry loop. Reduce to 1 clone by restructuring the entry API usage.

### O7: simd-json for JSON parsing (Deserialization)

**Impact:** ~15-30% deserialization improvement on large messages.

`simd-json` provides SIMD-accelerated JSON parsing that is serde-compatible. It requires a **mutable** input buffer (`&mut [u8]`), which works with our codec since we already have `BytesMut`.

Integration approach:
- Add `simd-json` as optional dependency behind a `simd` feature flag
- In the codec `Decoder`, use `simd_json::from_slice` when the feature is enabled
- Falls back to `serde_json::from_slice` without the feature
- No API changes needed

### O8: RunnerDefinition clone elimination (Cache update)

**Impact:** ~5-10% on definition updates.

`market_book_cache.rs` clones `RunnerDefinition` per runner during `update_market_definition`. With O2 (Arc-wrapping definitions), this becomes `Arc::clone` instead. This optimization is a direct consequence of O2.

## Out of Scope

- **Zero-copy deserialization with lifetimes** (`Cow<'a, str>`, `#[serde(borrow)]`): Requires lifetime parameters on all structs. Pervasive change with high risk, marginal benefit over CompactString for strings <24 chars.
- **Custom allocator**: Global allocator changes (jemalloc, mimalloc) affect the entire binary. Worth benchmarking separately.
- **Encoder optimization** (`serde_json::to_writer` instead of `to_string`): Send path is not hot (subscriptions/heartbeats only).
- **Channel backpressure monitoring**: Current mpsc design is sound.

## Dependency Additions

```toml
[workspace.dependencies]
compact_str = { version = "0.9", features = ["serde"] }
smallvec = { version = "1.13", features = ["serde"] }
simd-json = { version = "0.14", optional = true }
```
