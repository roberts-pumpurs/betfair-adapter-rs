# Stream Performance Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Establish criterion benchmarks for the streaming hot path, then optimize based on measured data.

**Architecture:** Benchmarks live in `crates/betfair-stream-api/benches/` using criterion. Three benchmark files cover the pipeline stages: deserialization, cache update, and full process_message. Existing fixture files provide realistic test data. After baselines, optimizations target the five identified bottlenecks (P0-P4) in priority order.

**Tech Stack:** criterion (benchmarks), serde_json (deser), std collections (cache)

---

### Task 1: Add criterion dependency and benchmark scaffold

**Files:**
- Modify: `Cargo.toml` (workspace root, add criterion to workspace deps)
- Modify: `crates/betfair-stream-api/Cargo.toml` (add criterion dev-dependency + `[[bench]]` sections)

**Step 1: Add criterion to workspace dependencies**

In `Cargo.toml` (workspace root), add under `[workspace.dependencies]` after the `# Testnig` section:

```toml
criterion = { version = "0.5", features = ["html_reports"] }
```

**Step 2: Add criterion and bench targets to betfair-stream-api**

In `crates/betfair-stream-api/Cargo.toml`, add:

```toml
[dev-dependencies]
pretty_assertions.workspace = true
criterion.workspace = true
betfair-stream-types.workspace = true

[[bench]]
name = "deserialize"
harness = false

[[bench]]
name = "cache_update"
harness = false

[[bench]]
name = "process_message"
harness = false
```

Note: `betfair-stream-types` is already a regular dependency, but adding it to dev-deps explicitly makes intent clear for bench files that import test fixtures. If the compiler complains, omit it.

**Step 3: Create empty benchmark files**

Create three files that compile but have no benchmarks yet:

`crates/betfair-stream-api/benches/deserialize.rs`:
```rust
use criterion::{criterion_group, criterion_main};

fn deserialize_benchmarks(_c: &mut criterion::Criterion) {
    // Will be filled in Task 2
}

criterion_group!(benches, deserialize_benchmarks);
criterion_main!(benches);
```

`crates/betfair-stream-api/benches/cache_update.rs`:
```rust
use criterion::{criterion_group, criterion_main};

fn cache_update_benchmarks(_c: &mut criterion::Criterion) {
    // Will be filled in Task 3
}

criterion_group!(benches, cache_update_benchmarks);
criterion_main!(benches);
```

`crates/betfair-stream-api/benches/process_message.rs`:
```rust
use criterion::{criterion_group, criterion_main};

fn process_message_benchmarks(_c: &mut criterion::Criterion) {
    // Will be filled in Task 4
}

criterion_group!(benches, process_message_benchmarks);
criterion_main!(benches);
```

**Step 4: Verify it compiles**

Run: `cargo bench -p betfair-stream-api --no-run`
Expected: Compiles successfully with no errors.

**Step 5: Commit**

```bash
git add Cargo.toml crates/betfair-stream-api/Cargo.toml crates/betfair-stream-api/benches/
git commit -m "feat: add criterion benchmark scaffold for stream performance"
```

---

### Task 2: Add xtask bench command

**Files:**
- Modify: `xtask/src/main.rs`

**Step 1: Add Bench variant to Args enum**

Add after the existing `Typos` variant (around line 20):

```rust
Bench {
    #[clap(last = true)]
    args: Vec<String>,
},
```

**Step 2: Add match arm for Bench**

Add in the `match args` block (before the closing brace, around line 135):

```rust
Args::Bench { args } => {
    println!("cargo bench");
    cmd!(sh, "cargo bench -p betfair-stream-api {args...}").run()?;
}
```

**Step 3: Verify xtask compiles**

Run: `cargo build -p xtask`
Expected: Compiles successfully.

**Step 4: Commit**

```bash
git add xtask/src/main.rs
git commit -m "feat: add cargo xtask bench command"
```

---

### Task 3: Write deserialization benchmarks

**Files:**
- Modify: `crates/betfair-stream-api/benches/deserialize.rs`

**Context for implementer:**
- Fixtures are at `crates/betfair-stream-types/tests/resources/` (relative to workspace root)
- The benchmark binary runs from the workspace root, so use paths relative to workspace root
- `ResponseMessage` is in `betfair_stream_types::response::ResponseMessage`
- The codec `StreamAPIClientCodec` is public from `betfair_stream_api`
- For the delta benchmark, use inline JSON matching the format in the fixture file `29788105` first line

**Step 1: Write the full deserialize benchmark file**

```rust
use std::path::Path;

use betfair_stream_types::response::ResponseMessage;
use criterion::{Criterion, black_box, criterion_group, criterion_main};

fn fixture(name: &str) -> String {
    let path = Path::new("crates/betfair-stream-types/tests/resources").join(name);
    std::fs::read_to_string(&path)
        .unwrap_or_else(|e| panic!("failed to read fixture {}: {e}", path.display()))
}

fn deser_market_change_delta(c: &mut Criterion) {
    // Minimal delta: single runner price update (~200 bytes)
    let json = r#"{"op":"mcm","id":2,"clk":"AHMAcArtjjje","pt":1471370160471,"mc":[{"id":"1.126235656","rc":[{"atb":[[1.01,200]],"id":13536143}]}]}"#;

    c.bench_function("deser_market_change_delta", |b| {
        b.iter(|| {
            black_box(serde_json::from_str::<ResponseMessage>(black_box(json)).unwrap());
        });
    });
}

fn deser_market_change_image(c: &mut Criterion) {
    let json = fixture("streaming_mcm_SUB_IMAGE.json");

    c.bench_function("deser_market_change_image", |b| {
        b.iter(|| {
            black_box(serde_json::from_str::<ResponseMessage>(black_box(&json)).unwrap());
        });
    });
}

fn deser_market_change_large_image(c: &mut Criterion) {
    let json = fixture("streaming_mcm_SUB_IMAGE_no_market_def.json");

    c.bench_function("deser_market_change_large_image", |b| {
        b.iter(|| {
            black_box(serde_json::from_str::<ResponseMessage>(black_box(&json)).unwrap());
        });
    });
}

fn deser_order_change(c: &mut Criterion) {
    let json = fixture("streaming_ocm_FULL_IMAGE.json");

    c.bench_function("deser_order_change", |b| {
        b.iter(|| {
            black_box(serde_json::from_str::<ResponseMessage>(black_box(&json)).unwrap());
        });
    });
}

criterion_group!(
    benches,
    deser_market_change_delta,
    deser_market_change_image,
    deser_market_change_large_image,
    deser_order_change,
);
criterion_main!(benches);
```

**Step 2: Verify benchmarks run**

Run: `cargo bench -p betfair-stream-api --bench deserialize`
Expected: All 4 benchmarks run and produce timing data. No panics.

**Step 3: Commit**

```bash
git add crates/betfair-stream-api/benches/deserialize.rs
git commit -m "feat: add deserialization benchmarks for streaming messages"
```

---

### Task 4: Write cache update benchmarks

**Files:**
- Modify: `crates/betfair-stream-api/benches/cache_update.rs`

**Context for implementer:**
- `StreamState` is at `betfair_stream_api::cache::tracker::StreamState`
- `MarketBookCache` is at `betfair_stream_api::cache::primitives::MarketBookCache`
- `MarketChangeMessage` is `betfair_stream_types::response::market_change_message::MarketChangeMessage`
- `ResponseMessage::MarketChange(msg)` gives you the `MarketChangeMessage`
- The `streaming_mcm_SUB_IMAGE.json` fixture parses to a `ResponseMessage::MarketChange` with a full market image
- For delta setup: first apply a full image to populate the cache, then benchmark applying a delta
- Fixtures containing market definitions: `streaming_mcm_SUB_IMAGE.json` has a full image with market definition and runners

**Step 1: Write the full cache_update benchmark file**

```rust
use std::path::Path;

use betfair_stream_api::cache::tracker::StreamState;
use betfair_stream_types::response::ResponseMessage;
use betfair_stream_types::response::market_change_message::{
    MarketChange, MarketChangeMessage, MarketDefinition, RunnerDefinition,
    StreamRunnerDefinitionStatus,
};
use betfair_stream_types::response::{DatasetChangeMessage, ChangeType};
use betfair_adapter::betfair_types::types::sports_aping::{MarketId, SelectionId};
use betfair_adapter::betfair_types::price::Price;
use betfair_adapter::betfair_types::size::Size;
use betfair_adapter::betfair_types::num;
use betfair_stream_types::response::UpdateSet2;
use criterion::{Criterion, black_box, criterion_group, criterion_main};

fn fixture(name: &str) -> String {
    let path = Path::new("crates/betfair-stream-types/tests/resources").join(name);
    std::fs::read_to_string(&path)
        .unwrap_or_else(|e| panic!("failed to read fixture {}: {e}", path.display()))
}

fn parse_market_change(json: &str) -> MarketChangeMessage {
    match serde_json::from_str::<ResponseMessage>(json).unwrap() {
        ResponseMessage::MarketChange(msg) => msg,
        other => panic!("expected MarketChange, got {other:?}"),
    }
}

/// Benchmark: apply a single runner delta to a pre-populated cache
fn cache_update_delta(c: &mut Criterion) {
    // First populate the cache with a full image
    let image_json = fixture("streaming_mcm_SUB_IMAGE.json");
    let image_msg = parse_market_change(&image_json);

    let mut state = StreamState::new();
    state.market_change_update(image_msg);

    // Delta: single runner price update
    let delta_json = r#"{"op":"mcm","id":2,"clk":"AHMAcArtjjje","pt":1471370160471,"mc":[{"id":"1.128149474","rc":[{"atb":[[1.01,200]],"id":4520808}]}]}"#;
    let delta_msg = parse_market_change(delta_json);

    c.bench_function("cache_update_delta", |b| {
        b.iter_batched(
            || (state.clone(), delta_msg.clone()),
            |(mut s, msg)| {
                black_box(s.market_change_update(msg));
            },
            criterion::BatchSize::SmallInput,
        );
    });
}

/// Benchmark: apply a full market image to an empty cache
fn cache_update_full_image(c: &mut Criterion) {
    let json = fixture("streaming_mcm_SUB_IMAGE.json");
    let msg = parse_market_change(&json);

    c.bench_function("cache_update_full_image", |b| {
        b.iter_batched(
            || (StreamState::new(), msg.clone()),
            |(mut s, msg)| {
                black_box(s.market_change_update(msg));
            },
            criterion::BatchSize::SmallInput,
        );
    });
}

/// Benchmark: update market definition (triggers RunnerDefinition cloning)
fn cache_update_market_definition(c: &mut Criterion) {
    // Populate cache first
    let image_json = fixture("streaming_mcm_SUB_IMAGE.json");
    let image_msg = parse_market_change(&image_json);
    let mut state = StreamState::new();
    state.market_change_update(image_msg);

    // Build a market definition update message
    // Re-parse the same image to get the market definition for re-application
    let def_json = fixture("streaming_mcm_SUB_IMAGE.json");
    let def_msg = parse_market_change(&def_json);

    c.bench_function("cache_update_market_definition", |b| {
        b.iter_batched(
            || (state.clone(), def_msg.clone()),
            |(mut s, msg)| {
                black_box(s.market_change_update(msg));
            },
            criterion::BatchSize::SmallInput,
        );
    });
}

criterion_group!(
    benches,
    cache_update_delta,
    cache_update_full_image,
    cache_update_market_definition,
);
criterion_main!(benches);
```

**Step 2: Verify benchmarks run**

Run: `cargo bench -p betfair-stream-api --bench cache_update`
Expected: All 3 benchmarks run and produce timing data.

**Step 3: Commit**

```bash
git add crates/betfair-stream-api/benches/cache_update.rs
git commit -m "feat: add cache update benchmarks"
```

---

### Task 5: Write process_message benchmarks

**Files:**
- Modify: `crates/betfair-stream-api/benches/process_message.rs`

**Context for implementer:**
- `MessageProcessor` trait is at `betfair_stream_api::MessageProcessor`
- `Cache` struct is at `betfair_stream_api::Cache` — but it's constructed via `BetfairStreamBuilder::new()` which requires an `Unauthenticated` client. For direct benchmark use, we need to construct `Cache` directly.
- Looking at `lib.rs:74-76`, `Cache` has field `state: StreamState` — it's a simple wrapper. We need `Cache` to be constructible for benchmarks.
- `Cache` currently has no public constructor — only created inside `BetfairStreamBuilder::new()`.
- **Solution:** Add a `Cache::new()` public constructor. This is a minor API addition, not a breaking change.

**Step 1: Add `Cache::new()` constructor**

In `crates/betfair-stream-api/src/lib.rs`, add after line 76 (after the `Cache` struct definition), before the `CachedMessage` enum:

```rust
impl Cache {
    /// Creates a new `Cache` with an empty `StreamState`.
    #[must_use]
    pub fn new() -> Self {
        Self {
            state: StreamState::new(),
        }
    }
}

impl Default for Cache {
    fn default() -> Self {
        Self::new()
    }
}
```

**Step 2: Write the full process_message benchmark file**

```rust
use std::path::Path;

use betfair_stream_api::cache::primitives::MarketBookCache;
use betfair_stream_api::cache::tracker::StreamState;
use betfair_stream_api::{Cache, MessageProcessor};
use betfair_stream_types::response::ResponseMessage;
use betfair_stream_types::response::market_change_message::MarketChangeMessage;
use criterion::{Criterion, black_box, criterion_group, criterion_main};

fn fixture(name: &str) -> String {
    let path = Path::new("crates/betfair-stream-types/tests/resources").join(name);
    std::fs::read_to_string(&path)
        .unwrap_or_else(|e| panic!("failed to read fixture {}: {e}", path.display()))
}

fn parse_market_change(json: &str) -> MarketChangeMessage {
    match serde_json::from_str::<ResponseMessage>(json).unwrap() {
        ResponseMessage::MarketChange(msg) => msg,
        other => panic!("expected MarketChange, got {other:?}"),
    }
}

/// Benchmark: full process_message pipeline with a delta (the most common hot path)
fn process_message_delta(c: &mut Criterion) {
    let image_json = fixture("streaming_mcm_SUB_IMAGE.json");
    let image_msg: ResponseMessage = serde_json::from_str(&image_json).unwrap();

    // Warm up cache with the full image
    let mut cache = Cache::new();
    cache.process_message(image_msg);

    // Delta message
    let delta_json = r#"{"op":"mcm","id":2,"clk":"AHMAcArtjjje","pt":1471370160471,"mc":[{"id":"1.128149474","rc":[{"atb":[[1.01,200]],"id":4520808}]}]}"#;
    let delta_msg: ResponseMessage = serde_json::from_str(delta_json).unwrap();

    c.bench_function("process_message_delta", |b| {
        b.iter_batched(
            || (cache.clone(), delta_msg.clone()),
            |(mut c, msg)| {
                black_box(c.process_message(msg));
            },
            criterion::BatchSize::SmallInput,
        );
    });
}

/// Benchmark: full process_message pipeline with a full image
fn process_message_image(c: &mut Criterion) {
    let json = fixture("streaming_mcm_SUB_IMAGE.json");
    let msg: ResponseMessage = serde_json::from_str(&json).unwrap();

    c.bench_function("process_message_image", |b| {
        b.iter_batched(
            || (Cache::new(), msg.clone()),
            |(mut c, msg)| {
                black_box(c.process_message(msg));
            },
            criterion::BatchSize::SmallInput,
        );
    });
}

/// Benchmark: isolate the cost of cloning a populated MarketBookCache
/// This directly measures the P0 bottleneck
fn cache_clone_isolated(c: &mut Criterion) {
    // Build a populated cache via the full image
    let image_json = fixture("streaming_mcm_SUB_IMAGE.json");
    let image_msg = parse_market_change(&image_json);

    let mut state = StreamState::new();
    state.market_change_update(image_msg);

    // Get references to cached markets
    let caches: Vec<&MarketBookCache> = state
        .market_stream_tracker
        .states();

    // Pre-clone one to have an owned version for repeated cloning
    let owned: Vec<MarketBookCache> = caches.into_iter().cloned().collect();

    c.bench_function("cache_clone_isolated", |b| {
        b.iter(|| {
            let cloned: Vec<MarketBookCache> = black_box(&owned).iter().cloned().collect();
            black_box(cloned);
        });
    });
}

criterion_group!(
    benches,
    process_message_delta,
    process_message_image,
    cache_clone_isolated,
);
criterion_main!(benches);
```

**Step 3: Verify benchmarks compile and run**

Run: `cargo bench -p betfair-stream-api --bench process_message`
Expected: All 3 benchmarks run. If `market_stream_tracker` field isn't public, access via `StreamState` methods — check if `states()` or a similar accessor exists.

**Step 4: Commit**

```bash
git add crates/betfair-stream-api/src/lib.rs crates/betfair-stream-api/benches/process_message.rs
git commit -m "feat: add process_message benchmarks and Cache::new() constructor"
```

---

### Task 6: Run full baseline and verify all benchmarks

**Files:** None (verification only)

**Step 1: Run all benchmarks**

Run: `cargo xtask bench`
Expected: All 10 benchmarks across 3 files run successfully. Note the timing numbers — these are the baselines.

**Step 2: Verify HTML reports generated**

Check: `target/criterion/` directory should contain HTML reports for each benchmark.

**Step 3: Commit any fixups needed**

If any benchmark needed adjustments to compile/run (field visibility, import paths), commit those fixes:

```bash
git add -A
git commit -m "fix: benchmark adjustments for field access"
```

---

### Task 7: Fix compilation issues and field access

**Context for implementer:**
This task exists as a catch-all for compilation issues discovered in Task 6. Common issues:

1. `Cache` struct fields may not be public enough — add `pub fn new()` if not done in Task 5
2. `market_stream_tracker` field on `StreamState` is `pub` (confirmed in source), but `states()` method returns `Vec<&MarketBookCache>` — use this
3. `MarketChangeMessage` is a newtype wrapper — check if it needs `.0` access for inner `DatasetChangeMessage`
4. Import paths may need adjustment — `betfair_stream_api::cache::tracker::StreamState` etc.

**Step 1: Fix any compilation errors from `cargo bench -p betfair-stream-api --no-run`**

Run the compile check, fix errors one at a time. Common fixes:
- Add missing `use` imports
- Access private fields through public methods
- Adjust fixture paths if running from a different working directory

**Step 2: Ensure `cargo xtask check` still passes**

Run: `cargo xtask check`
Expected: No clippy warnings, no format issues.

**Step 3: Commit fixes**

```bash
git add -A
git commit -m "fix: resolve benchmark compilation issues"
```

---

## Post-Baseline: Optimization Tasks

> These tasks should only be started AFTER Task 6 baselines are recorded.
> Each optimization should be benchmarked before AND after to measure impact.

### Task 8: P0 — Arc-wrap CachedMessage output (biggest win)

**Files:**
- Modify: `crates/betfair-stream-api/src/lib.rs` (CachedMessage enum + Cache::process_message)
- Modify: `crates/betfair-stream-api/src/cache/tracker/market_stream_tracker.rs` (store Arc internally)
- Modify: `crates/betfair-stream-api/src/cache/tracker/order_stream_tracker.rs` (store Arc internally)
- Modify: `crates/betfair-stream-api/src/cache/tracker/mod.rs` (return types)

**Step 1: Change CachedMessage to use Arc**

In `crates/betfair-stream-api/src/lib.rs`, change:

```rust
// Before (line 82-98)
pub enum CachedMessage {
    Connection(ConnectionMessage),
    MarketChange(Vec<MarketBookCache>),
    OrderChange(Vec<OrderBookCache>),
    Status(StatusMessage),
}

// After
pub enum CachedMessage {
    Connection(ConnectionMessage),
    MarketChange(Vec<Arc<MarketBookCache>>),
    OrderChange(Vec<Arc<OrderBookCache>>),
    Status(StatusMessage),
}
```

**Step 2: Change MarketStreamTracker to store Arc<MarketBookCache>**

In `crates/betfair-stream-api/src/cache/tracker/market_stream_tracker.rs`:

```rust
// Change the HashMap value type
pub struct MarketStreamTracker {
    market_state: HashMap<MarketId, Arc<MarketBookCache>>,
    updates_processed: u64,
}
```

Update `process()` method:
- When creating new cache entries, wrap in `Arc::new()`
- When updating, use `Arc::make_mut()` to get mutable access (this clones only if there are other references)
- Return `Vec<Arc<MarketBookCache>>` instead of `Vec<&MarketBookCache>`

**Step 3: Change OrderStreamTracker similarly**

Apply same pattern to `order_stream_tracker.rs`.

**Step 4: Update Cache::process_message**

In `lib.rs`, the `.cloned().collect()` lines become simple `collect()` since Arc::clone is cheap:

```rust
// Before (lines 109-118)
ResponseMessage::MarketChange(market_change_message) => self
    .state
    .market_change_update(market_change_message)
    .map(|markets| markets.into_iter().cloned().collect::<Vec<_>>())
    .map(CachedMessage::MarketChange),

// After
ResponseMessage::MarketChange(market_change_message) => self
    .state
    .market_change_update(market_change_message)
    .map(CachedMessage::MarketChange),
```

**Step 5: Update StreamState return types**

In `mod.rs`, change `market_change_update` and `order_change_update` to return `Vec<Arc<...>>`.

**Step 6: Fix downstream compilation**

Run: `cargo build -p betfair-stream-api`
Fix any type mismatches in tests, examples, and the benchmarks themselves.

**Step 7: Run tests**

Run: `cargo xtask test`
Expected: All tests pass.

**Step 8: Run benchmarks and compare**

Run: `cargo xtask bench`
Compare `process_message_delta` and `cache_clone_isolated` with baselines. Expected: significant improvement (10-100x for clone_isolated).

**Step 9: Commit**

```bash
git add -A
git commit -m "perf: Arc-wrap cache output to eliminate deep clones (P0)"
```

---

### Task 9: P1 — Rewrite DatasetChangeMessage deserializer

**Files:**
- Modify: `crates/betfair-stream-types/src/response.rs` (lines 148-197)

**Step 1: Replace Value-based deserializer with field-by-field approach**

The current deserializer parses into `serde_json::Value` then clones fields. Replace with a `#[derive(Deserialize)]` helper struct that maps the JSON fields directly, then converts to `DatasetChangeMessage`.

In `crates/betfair-stream-types/src/response.rs`, replace the manual `Deserialize` impl (lines 148-197):

```rust
impl<'de, T> Deserialize<'de> for DatasetChangeMessage<T>
where
    T: DeserializeOwned + DataChange<T>,
{
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        // Use a serde_json::Map to avoid full Value clone overhead
        use serde_json::Map;

        let mut map = Map::deserialize(deserializer)?;

        let id = map
            .get("id")
            .and_then(serde_json::Value::as_i64)
            .map(|id| id as i32);

        // Extract the data field using the type's key, removing it to take ownership
        let data = map.remove(T::key()).and_then(|data| {
            serde_json::from_value(data).expect("data item should be deserialized")
        });

        let change_type = map.remove("ct").and_then(|ct| {
            serde_json::from_value(ct).expect("ct should be deserialized")
        });

        let clock = map
            .get("clk")
            .and_then(|clk| clk.as_str())
            .map(|clk| Clock(clk.to_owned()));

        let heartbeat_ms = map.get("heartbeatMs").and_then(serde_json::Value::as_i64);

        let publish_time = map
            .get("pt")
            .and_then(serde_json::Value::as_i64)
            .and_then(|pt| chrono::TimeZone::timestamp_millis_opt(&chrono::Utc, pt).latest());

        let initial_clock = map
            .get("initialClk")
            .and_then(|ic| ic.as_str())
            .map(|ic| InitialClock(ic.to_owned()));

        let conflate_ms = map.get("conflateMs").and_then(serde_json::Value::as_i64);

        let segment_type = map.remove("segmentType").and_then(|st| {
            serde_json::from_value(st).expect("segmentType should be deserialized")
        });

        let status = map
            .get("status")
            .and_then(serde_json::Value::as_i64)
            .map(|s| s as i32);

        Ok(Self {
            id,
            change_type,
            clock,
            heartbeat_ms,
            publish_time,
            initial_clock,
            data,
            conflate_ms,
            segment_type,
            status,
        })
    }
}
```

Key change: use `map.remove()` instead of `map.get() + clone()` for fields that need `from_value()`. This takes ownership instead of cloning.

**Step 2: Run existing tests**

Run: `cargo nextest run -p betfair-stream-types`
Expected: All 16 fixture tests pass.

**Step 3: Run deserialization benchmarks**

Run: `cargo bench -p betfair-stream-api --bench deserialize`
Compare with baselines.

**Step 4: Commit**

```bash
git add crates/betfair-stream-types/src/response.rs
git commit -m "perf: eliminate Value cloning in DatasetChangeMessage deserializer (P1)"
```

---

### Task 10: P2 — HashMap capacity hints

**Files:**
- Modify: `crates/betfair-stream-api/src/cache/primitives/market_book_cache.rs` (line 37)
- Modify: `crates/betfair-stream-api/src/cache/primitives/orderbook_cache.rs` (line 34)

**Step 1: Add capacity to MarketBookCache::new()**

In `market_book_cache.rs` line 37, change:
```rust
// Before
runners: HashMap::new(),
// After
runners: HashMap::with_capacity(16),
```

**Step 2: Use runner count from market definition when available**

In `update_market_definition()` (line 134), when receiving a market definition with runners, if the runners HashMap is empty, reserve capacity:

```rust
pub fn update_market_definition(&mut self, market_definition: Box<MarketDefinition>) {
    if self.runners.is_empty() {
        self.runners.reserve(market_definition.runners.len());
    }
    // ... rest unchanged
```

**Step 3: Add capacity to OrderBookCache::new()**

In `orderbook_cache.rs` line 34, change:
```rust
// Before
runners: HashMap::new(),
// After
runners: HashMap::with_capacity(8),
```

**Step 4: Run tests**

Run: `cargo xtask test`
Expected: All tests pass.

**Step 5: Run cache_update benchmarks**

Run: `cargo bench -p betfair-stream-api --bench cache_update`
Compare `cache_update_full_image` with baseline.

**Step 6: Commit**

```bash
git add crates/betfair-stream-api/src/cache/primitives/market_book_cache.rs crates/betfair-stream-api/src/cache/primitives/orderbook_cache.rs
git commit -m "perf: add HashMap capacity hints for runner maps (P2)"
```

---

### Task 11: P3 — Reduce RunnerDefinition cloning

**Files:**
- Modify: `crates/betfair-stream-api/src/cache/primitives/market_book_cache.rs` (update_market_definition)

**Step 1: Use index-based iteration to avoid clone**

In `update_market_definition` (line 134-151), iterate with ownership by consuming the Vec:

```rust
pub fn update_market_definition(&mut self, market_definition: Box<MarketDefinition>) {
    if self.runners.is_empty() {
        self.runners.reserve(market_definition.runners.len());
    }

    for runner_definition in &market_definition.runners {
        let Some(selection_id) = runner_definition.id else {
            continue;
        };
        let key = (selection_id, runner_definition.handicap);
        if let Some(runner) = self.runners.get_mut(&key) {
            runner.set_definition(runner_definition.clone());
        }
        // Don't add new runners from definition alone — they'll come from runner changes
        // The old code added runners from definitions, but this creates empty runners
        // that may never receive data. Only add if runner already exists.
    }

    // Add runners that exist in definition but not yet in cache
    for runner_definition in &market_definition.runners {
        let Some(selection_id) = runner_definition.id else {
            continue;
        };
        let key = (selection_id, runner_definition.handicap);
        if !self.runners.contains_key(&key) {
            self.add_runner_from_definition(runner_definition.clone());
        }
    }

    self.market_definition = Some(market_definition);
}
```

Wait — this still clones. The real fix: since `market_definition` is moved into `self.market_definition` at the end, we can't consume the runners Vec. The cleanest approach is to keep the existing logic but note that the clone here is actually necessary (the runner stores its own copy, and the market definition stores the Vec). This is the correct design since runners can be updated independently.

**Alternative: skip this task** if benchmarks show the RunnerDefinition clone cost is low relative to P0/P1. The cloning only happens on market definition changes (infrequent) not on every delta.

**Step 2: Run tests**

Run: `cargo xtask test`
Expected: All pass.

**Step 3: Commit if changes were made**

```bash
git add crates/betfair-stream-api/src/cache/primitives/market_book_cache.rs
git commit -m "perf: reduce RunnerDefinition cloning in market definition updates (P3)"
```

---

### Task 12: P4 — Order update ownership transfer

**Files:**
- Modify: `crates/betfair-stream-api/src/cache/primitives/orderbook_runner_cache.rs` (update_unmatched)
- Modify: `crates/betfair-stream-api/src/cache/primitives/orderbook_cache.rs` (update_cache caller)

**Step 1: Change update_unmatched to take owned Orders**

In `orderbook_runner_cache.rs`, change:

```rust
// Before (lines 45-53)
pub(crate) fn update_unmatched<'o>(
    &mut self,
    unmatched_orders: impl IntoIterator<Item = &'o Order>,
) {
    for order in unmatched_orders {
        self.unmatched_orders
            .insert(order.id.clone(), order.clone());
    }
}

// After
pub(crate) fn update_unmatched(
    &mut self,
    unmatched_orders: impl IntoIterator<Item = Order>,
) {
    for order in unmatched_orders {
        let id = order.id.clone();
        self.unmatched_orders.insert(id, order);
    }
}
```

**Step 2: Update caller in orderbook_cache.rs**

In `orderbook_cache.rs` line 65-66, the caller currently passes `uo` which is `&Vec<Order>`. Change to pass owned:

```rust
// Before
if let Some(ref uo) = runner_change.unmatched_orders {
    runner.update_unmatched(uo);
}

// After
if let Some(uo) = runner_change.unmatched_orders {
    runner.update_unmatched(uo);
}
```

This changes from borrowing to moving the `Vec<Order>`. Since `runner_change` is consumed by the loop (it comes from `order_runner_change` Vec iteration), this should work. Verify that `runner_change` isn't used after this point.

Note: The other fields (`matched_lays`, `matched_backs`, `strategy_matches`) still borrow with `ref`. This is fine since those only need slice access.

**Step 3: Run tests**

Run: `cargo xtask test`
Expected: All pass.

**Step 4: Run benchmarks**

Run: `cargo bench -p betfair-stream-api --bench cache_update`
Compare order-related metrics if present.

**Step 5: Commit**

```bash
git add crates/betfair-stream-api/src/cache/primitives/orderbook_runner_cache.rs crates/betfair-stream-api/src/cache/primitives/orderbook_cache.rs
git commit -m "perf: take owned Order in update_unmatched to avoid cloning (P4)"
```

---

### Task 13: Final verification and cleanup

**Files:** None (verification only)

**Step 1: Run full test suite**

Run: `cargo xtask test`
Expected: All tests pass.

**Step 2: Run full check**

Run: `cargo xtask check`
Expected: No clippy warnings, no format issues.

**Step 3: Run all benchmarks for final numbers**

Run: `cargo xtask bench`
Record all timing data and compare with Task 6 baselines.

**Step 4: Run typos check**

Run: `cargo xtask typos`
Expected: No typos.
