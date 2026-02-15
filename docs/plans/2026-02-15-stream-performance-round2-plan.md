# Stream Performance Round 2: Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Eliminate remaining allocations, clones, and data structure inefficiencies across the streaming pipeline.

**Architecture:** Replace heap-allocating `String` with inline `CompactString`, heap-allocating `Vec` with inline `SmallVec`, tree-pointer `BTreeMap` with cache-friendly sorted `Vec`, and wrap cold cache fields in `Arc` to cheapen deep clones.

**Tech Stack:** compact_str 0.9, smallvec 1.13, simd-json 0.14 (optional feature), criterion benchmarks.

---

### Task 1: Add workspace dependencies

**Files:**
- Modify: `Cargo.toml` (workspace root, `[workspace.dependencies]` section)
- Modify: `crates/betfair-stream-types/Cargo.toml`
- Modify: `crates/betfair-stream-api/Cargo.toml`

**Step 1: Add dependencies to workspace root**

In `Cargo.toml` under `[workspace.dependencies]`, add:

```toml
compact_str = { version = "0.9", features = ["serde"] }
smallvec = { version = "1.13", features = ["serde"] }
```

**Step 2: Add dependencies to betfair-stream-types**

In `crates/betfair-stream-types/Cargo.toml` under `[dependencies]`, add:

```toml
compact_str.workspace = true
smallvec.workspace = true
```

**Step 3: Add simd-json to betfair-stream-api**

In `crates/betfair-stream-api/Cargo.toml`, add:

```toml
[features]
simd = ["dep:simd-json"]

[dependencies]
simd-json = { version = "0.14", optional = true }
```

**Step 4: Verify compilation**

Run: `cargo check --workspace`
Expected: Compiles with no errors (new deps unused but available).

**Step 5: Commit**

```
chore: add compact_str, smallvec, simd-json dependencies
```

---

### Task 2: CompactString for stream-types string fields

**Files:**
- Modify: `crates/betfair-stream-types/src/response.rs` (lines 143, 146)
- Modify: `crates/betfair-stream-types/src/response/market_change_message.rs` (lines 67, 70, 73, 75, 85, 88, 99, 136, 157, 172, 179, 183, 362)
- Modify: `crates/betfair-stream-types/src/response/order_change_message.rs` (lines 110, 122, 132)
- Modify: `crates/betfair-stream-types/src/response/connection_message.rs` (line 11)
- Modify: `crates/betfair-stream-types/src/response/status_message.rs` (lines 45, 62, 67)

**Step 1: Add import to each file**

Add `use compact_str::CompactString;` to each response module file that has String fields.

**Step 2: Replace String fields in response.rs**

```rust
// Line 143: was String
pub struct Clock(pub CompactString);

// Line 146: was String
pub struct InitialClock(pub CompactString);
```

**Step 3: Update the custom deserializer in response.rs**

In the `DatasetChangeMessage` deserializer (~lines 170-190), update clock construction:

```rust
// Was: Clock(clk.to_owned())
// Now:
Clock(CompactString::from(clk))

// Was: InitialClock(ic.to_owned())
// Now:
InitialClock(CompactString::from(ic))
```

**Step 4: Replace String fields in market_change_message.rs**

All `String` → `CompactString` and `Option<String>` → `Option<CompactString>` and `Vec<String>` → `Vec<CompactString>`:

- MarketDefinition: `venue`, `race_type`, `settled_time`, `timezone`, `regulators`, `market_type`, `country_code`, `event_id`, `suspend_time`, `event_type_id`, `open_date`, `market_time` (12 fields)
- RunnerDefinition: `removal_date` (1 field)

**Step 5: Replace String fields in order_change_message.rs**

- Order: `lapse_status_reason_code`, `regulator_code`, `regulator_auth_code` (3 fields)

**Step 6: Replace String fields in connection_message.rs and status_message.rs**

- ConnectionMessage: `connection_id` (1 field)
- StatusSuccess: `connection_id` (1 field)
- StatusError: `error_message`, `connection_id` (2 fields)

**Step 7: Fix any compilation errors in stream-types**

Run: `cargo check -p betfair-stream-types`
Expected: Compiles. CompactString implements `Serialize`/`Deserialize` via the `serde` feature, so serde derives should work unchanged.

**Step 8: Fix compilation errors in downstream crates**

Run: `cargo check --workspace`
Fix any type mismatches where code constructs or pattern-matches on String. Common fixes:
- `"literal".to_owned()` → `CompactString::from("literal")` or `"literal".into()`
- `.as_str()` works on CompactString just like String
- `String::new()` → `CompactString::new("")` or `CompactString::default()`

**Step 9: Run tests**

Run: `cargo xtask test`
Expected: All tests pass.

**Step 10: Commit**

```
perf: replace String with CompactString in stream types (O1)
```

---

### Task 3: SmallVec for RunnerChange and OrderRunnerChange Vec fields

**Files:**
- Modify: `crates/betfair-stream-types/src/response/market_change_message.rs` (RunnerChange struct, ~lines 265-350; MarketChange struct, ~line 37)
- Modify: `crates/betfair-stream-types/src/response/order_change_message.rs` (OrderRunnerChange struct)

**Step 1: Add import**

```rust
use smallvec::SmallVec;
```

**Step 2: Replace Vec fields in RunnerChange**

All `Option<Vec<UpdateSet2>>` → `Option<SmallVec<[UpdateSet2; 4]>>` and
all `Option<Vec<UpdateSet3>>` → `Option<SmallVec<[UpdateSet3; 4]>>`:

- `best_available_to_back`: `Option<SmallVec<[UpdateSet3; 4]>>`
- `starting_price_back`: `Option<SmallVec<[UpdateSet2; 4]>>`
- `best_display_available_to_lay`: `Option<SmallVec<[UpdateSet3; 4]>>`
- `traded`: `Option<SmallVec<[UpdateSet2; 4]>>`
- `available_to_back`: `Option<SmallVec<[UpdateSet2; 4]>>`
- `starting_price_lay`: `Option<SmallVec<[UpdateSet2; 4]>>`
- `available_to_lay`: `Option<SmallVec<[UpdateSet2; 4]>>`
- `best_available_to_lay`: `Option<SmallVec<[UpdateSet3; 4]>>`
- `best_display_available_to_back`: `Option<SmallVec<[UpdateSet3; 4]>>`

**Step 3: Replace Vec field in MarketChange**

```rust
// runner_change field: was Option<Vec<RunnerChange>>
pub runner_change: Option<SmallVec<[RunnerChange; 4]>>,
```

**Step 4: Replace Vec fields in OrderRunnerChange**

- `matched_backs`: `Option<SmallVec<[UpdateSet2; 4]>>`
- `matched_lays`: `Option<SmallVec<[UpdateSet2; 4]>>`
- `unmatched_orders`: `Option<SmallVec<[Order; 2]>>`

**Step 5: Fix downstream compilation**

Run: `cargo check --workspace`

The cache module calls `.as_slice()` and uses `AsRef<[T]>` generics on the SmallVec contents. SmallVec implements `AsRef<[T]>` and `Deref<Target = [T]>`, so most call sites should work. Fix any that don't, typically:
- `vec.as_slice()` works on SmallVec
- `for item in vec` / `for item in &vec` works on SmallVec
- `vec.len()` works on SmallVec
- `vec.iter()` works on SmallVec

**Step 6: Run tests**

Run: `cargo xtask test`
Expected: All tests pass.

**Step 7: Commit**

```
perf: replace Vec with SmallVec for RunnerChange price fields (O3)
```

---

### Task 4: Remaining HashMap capacity hints

**Files:**
- Modify: `crates/betfair-stream-api/src/cache/primitives/orderbook_runner_cache.rs` (~lines 39, 41)
- Modify: `crates/betfair-stream-api/src/cache/tracker/market_stream_tracker.rs` (~line 19)
- Modify: `crates/betfair-stream-api/src/cache/tracker/order_stream_tracker.rs` (~line 19)

**Step 1: Add capacity to OrderBookRunner**

```rust
// Was: HashMap::new()
unmatched_orders: HashMap::with_capacity(16),
strategy_matches: HashMap::with_capacity(4),
```

**Step 2: Add capacity to tracker HashMaps**

```rust
// MarketStreamTracker::new() and OrderStreamTracker::new()
// Was: HashMap::new()
market_state: HashMap::with_capacity(64),
```

**Step 3: Run tests**

Run: `cargo xtask test`
Expected: All tests pass.

**Step 4: Commit**

```
perf: add HashMap capacity hints for order runner and trackers (O5)
```

---

### Task 5: MarketId clone reduction in tracker entry pattern

**Files:**
- Modify: `crates/betfair-stream-api/src/cache/tracker/market_stream_tracker.rs` (~lines 34-56)
- Modify: `crates/betfair-stream-api/src/cache/tracker/order_stream_tracker.rs` (~lines 34-56)

**Step 1: Restructure MarketStreamTracker::process to reduce clones**

Current pattern clones MarketId 3+ times. Restructure to clone once:

```rust
for market_change in data {
    // Take ownership instead of cloning from Option
    let Some(market_id) = market_change.market_id.take() else {
        continue;
    };

    let is_new = !self.market_state.contains_key(&market_id);

    if is_new {
        img = HasFullImage(true);
        self.market_state.insert(
            market_id.clone(),
            Arc::new(MarketBookCache::new(market_id.clone(), publish_time)),
        );
    }

    let market = self.market_state.get_mut(&market_id).unwrap();

    // ... rest of update logic (Arc::make_mut, etc.) ...

    market_ids.push(market_id); // Move, no clone
}
```

Note: `market_change.market_id` is `Option<MarketId>`. Using `.take()` instead of `.clone()` avoids one clone. The `entry()` API requires the key to be cloned for the insert case — using `contains_key` + `insert` avoids cloning in the common case (market already exists).

**Step 2: Apply same pattern to OrderStreamTracker::process**

Same restructuring for the order tracker.

**Step 3: Run tests**

Run: `cargo xtask test`
Expected: All tests pass.

**Step 4: Commit**

```
perf: reduce MarketId clones in tracker entry pattern (O6)
```

---

### Task 6: Arc-wrap MarketDefinition in MarketBookCache

**Files:**
- Modify: `crates/betfair-stream-api/src/cache/primitives/market_book_cache.rs`

**Step 1: Change market_definition field type**

```rust
// Was: market_definition: Option<Box<MarketDefinition>>
market_definition: Option<Arc<MarketDefinition>>,
```

Add `use std::sync::Arc;` import.

**Step 2: Update update_market_definition method**

```rust
pub fn update_market_definition(&mut self, market_definition: Box<MarketDefinition>) {
    // ... runner updates stay the same ...

    // Store as Arc instead of Box
    self.market_definition = Some(Arc::from(*market_definition));
}
```

Note: `Arc::from(*market_definition)` unboxes then wraps in Arc. This is a single allocation swap (Box → Arc). The `Box<MarketDefinition>` comes from the deserialized message, so we can't avoid the initial allocation — but we can make subsequent clones (triggered by `Arc::make_mut` on the parent `MarketBookCache`) essentially free.

**Step 3: Update any code that accesses market_definition mutably**

If any code does `self.market_definition.as_mut()`, it needs to use `Arc::make_mut()` or restructure. Check all access patterns. Most should be read-only (`.as_ref()`).

**Step 4: Update the getter/public API**

If `market_definition` is exposed publicly, update the return type from `Option<&Box<MarketDefinition>>` to `Option<&Arc<MarketDefinition>>` or just `Option<&MarketDefinition>` (deref through Arc).

**Step 5: Run tests**

Run: `cargo xtask test`
Expected: All tests pass.

**Step 6: Commit**

```
perf: Arc-wrap MarketDefinition to cheapen deep clones (O2a)
```

---

### Task 7: Arc-wrap RunnerDefinition in RunnerBookCache

**Files:**
- Modify: `crates/betfair-stream-api/src/cache/primitives/runner_book_cache.rs`
- Modify: `crates/betfair-stream-api/src/cache/primitives/market_book_cache.rs` (the loop in update_market_definition)

**Step 1: Change definition field type in RunnerBookCache**

```rust
// Was: definition: Option<RunnerDefinition>
definition: Option<Arc<RunnerDefinition>>,
```

**Step 2: Update set_definition to accept Arc**

```rust
pub fn set_definition(&mut self, definition: Arc<RunnerDefinition>) {
    self.definition = Some(definition);
}
```

**Step 3: Update update_market_definition loop in MarketBookCache**

Pre-wrap definitions in Arc, then share:

```rust
pub fn update_market_definition(&mut self, market_definition: Box<MarketDefinition>) {
    // ...
    for runner_definition in &market_definition.runners {
        let arc_def = Arc::new(runner_definition.clone());
        // ...
        if let Some(runner) = runner {
            runner.set_definition(Arc::clone(&arc_def));
        } else {
            self.add_runner_from_definition(arc_def);
        }
    }
    self.market_definition = Some(Arc::from(*market_definition));
}
```

Note: We still clone `RunnerDefinition` once per runner (from the `&market_definition.runners` borrow), but the resulting `Arc` is then shared cheaply. This is unavoidable because `market_definition` is stored as a whole and we can't destructure it.

Alternative: If we can take ownership of the runners Vec before storing market_definition:

```rust
let mut market_definition = *market_definition; // Unbox
let runner_defs: Vec<RunnerDefinition> = std::mem::take(&mut market_definition.runners);
// Now we own each RunnerDefinition, no clone needed
for runner_definition in runner_defs {
    let arc_def = Arc::new(runner_definition);
    // ...
}
// Store market_definition (now with empty runners vec)
self.market_definition = Some(Arc::new(market_definition));
```

This eliminates ALL RunnerDefinition clones. The trade-off is that `market_definition.runners` is now empty in the stored copy. If the stored copy's runners are ever accessed, this would be a problem — check usage.

**Step 4: Update add_runner_from_definition to accept Arc**

```rust
fn add_runner_from_definition(&mut self, definition: Arc<RunnerDefinition>) -> eyre::Result<()> {
    // Extract selection_id and handicap from definition before storing
    let runner = RunnerBookCache::new_from_arc_definition(definition)?;
    // ...
}
```

**Step 5: Run tests**

Run: `cargo xtask test`
Expected: All tests pass.

**Step 6: Commit**

```
perf: Arc-wrap RunnerDefinition to eliminate per-runner clones (O2b/O8)
```

---

### Task 8: Replace BTreeMap with sorted Vec in Available\<T\>

**Files:**
- Modify: `crates/betfair-stream-api/src/cache/primitives/available_cache.rs`

**Step 1: Change the data structure**

```rust
pub struct Available<T: UpdateSet> {
    pub book: Vec<(T::Key, T::Value)>,
}
```

**Step 2: Rewrite Available::new**

```rust
pub fn new<A: AsRef<[T]>>(prices: A) -> Self {
    let mut book = Vec::with_capacity(prices.as_ref().len());
    for price in prices.as_ref() {
        if !price.should_be_deleted() {
            book.push((price.key(), price.value()));
        }
    }
    book.sort_unstable_by(|a, b| a.0.cmp(&b.0));
    book.dedup_by(|a, b| a.0 == b.0);
    Self { book }
}
```

**Step 3: Rewrite Available::update**

```rust
pub fn update<A: AsRef<[T]>>(&mut self, prices: A) {
    for price in prices.as_ref() {
        let key = price.key();
        match self.book.binary_search_by(|entry| entry.0.cmp(&key)) {
            Ok(idx) => {
                if price.should_be_deleted() {
                    self.book.remove(idx);
                } else {
                    self.book[idx].1 = price.value();
                }
            }
            Err(idx) => {
                if !price.should_be_deleted() {
                    self.book.insert(idx, (key, price.value()));
                }
            }
        }
    }
}
```

**Step 4: Rewrite Available::clear**

```rust
pub fn clear(&mut self) {
    self.book.clear();
}
```

**Step 5: Update PartialEq, Eq, Serialize, Debug derives**

`Vec<(K, V)>` derives all the same traits as `BTreeMap<K, V>`, so derives should work. But `Serialize` output format changes (array of tuples vs map). Check if serialization format matters for tests.

If serialization format matters, implement custom `Serialize`/`Deserialize` to match the BTreeMap format, or update test expectations.

**Step 6: Run the Available tests**

Run: `cargo nextest run -p betfair-stream-api available_cache`
Expected: All 8 tests pass. Tests validate insert, delete, update, clear, and initialization — the behavior should be identical.

**Step 7: Run full test suite**

Run: `cargo xtask test`
Expected: All tests pass.

**Step 8: Commit**

```
perf: replace BTreeMap with sorted Vec in Available<T> (O4)
```

---

### Task 9: simd-json behind feature flag

**Files:**
- Modify: `crates/betfair-stream-api/src/lib.rs` (StreamAPIClientCodec Decoder impl)

**Step 1: Update the Decoder to conditionally use simd-json**

In the `Decoder` impl for `StreamAPIClientCodec`, replace the parsing logic:

```rust
fn decode(&mut self, src: &mut bytes::BytesMut) -> Result<Option<Self::Item>, Self::Error> {
    if let Some(pos) = src.iter().position(|&byte| byte == b'\n') {
        let delimiter_size = if pos > 0 && src[pos - 1] == b'\r' { 2 } else { 1 };
        let mut line = src.split_to(pos + 1);
        line.truncate(line.len().saturating_sub(delimiter_size));

        #[cfg(feature = "simd")]
        let (raw, data) = {
            let raw = line.clone().freeze();
            let data = simd_json::from_slice::<ResponseMessage>(&mut line)?;
            (raw, data)
        };

        #[cfg(not(feature = "simd"))]
        let (raw, data) = {
            let raw = line.freeze();
            let data = serde_json::from_slice::<ResponseMessage>(&raw)?;
            (raw, data)
        };

        return Ok(Some((raw, data)));
    }
    Ok(None)
}
```

Note: simd-json requires `&mut [u8]` and modifies the buffer in place. We clone the line first to preserve clean raw bytes for the `on_message_received` callback. The clone is a cheap memcpy (~200 bytes for typical deltas), far less than the parsing savings.

**Step 2: Add conditional import**

At the top of lib.rs:

```rust
#[cfg(feature = "simd")]
use simd_json;
```

**Step 3: Update error handling if needed**

simd-json errors implement `std::error::Error`, so `?` should work with the existing error type. If not, add a `From` impl.

**Step 4: Verify compilation both ways**

Run: `cargo check -p betfair-stream-api`
Run: `cargo check -p betfair-stream-api --features simd`
Expected: Both compile.

**Step 5: Run tests both ways**

Run: `cargo nextest run -p betfair-stream-api`
Run: `cargo nextest run -p betfair-stream-api --features simd`
Expected: All tests pass in both configurations.

**Step 6: Commit**

```
perf: add simd-json behind optional feature flag (O7)
```

---

### Task 10: Run benchmarks and final verification

**Step 1: Run cargo xtask check**

Run: `cargo xtask check`
Expected: Clippy + fmt pass.

**Step 2: Run cargo xtask test**

Run: `cargo xtask test`
Expected: All tests pass.

**Step 3: Run benchmarks**

Run: `cargo xtask bench`
Record all numbers. Compare against Round 1 baseline.

**Step 4: Run benchmarks with simd feature**

Run: `cargo bench -p betfair-stream-api --features simd`
Record numbers for comparison.

**Step 5: Commit any formatting fixes**

```
style: apply rustfmt fixes
```
