# Benchmark Expansion Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Add 16 benchmarks across 4 crates covering numeric primitives, request serialization, codec encoding, and async RPC round-trips.

**Architecture:** Per-crate benchmark files following the existing pattern (criterion 0.5, `harness = false`, `black_box`, `iter`/`iter_batched`). RPC benchmarks use `tokio::runtime::Runtime::block_on` for async. All wired into `cargo xtask bench`.

**Tech Stack:** criterion 0.5 (html_reports), serde_json, tokio, wiremock (via betfair-rpc-server-mock), bytes/tokio-util for codec.

---

### Task 1: Add `betfair-types` numeric benchmarks

**Files:**
- Modify: `crates/betfair-types/Cargo.toml`
- Create: `crates/betfair-types/benches/numeric.rs`

**Step 1: Add criterion dev-dependency and bench config to Cargo.toml**

Append to `crates/betfair-types/Cargo.toml`:

```toml
# under [dev-dependencies]
criterion.workspace = true

[[bench]]
name = "numeric"
harness = false
```

**Step 2: Write the benchmark file**

Create `crates/betfair-types/benches/numeric.rs`:

```rust
use std::collections::BTreeMap;
use std::hash::{Hash, Hasher};

use betfair_types::numeric::F64Ord;
use betfair_types::price::Price;
use betfair_types::size::Size;
use criterion::{Criterion, black_box, criterion_group, criterion_main};

fn price_new_low_range(c: &mut Criterion) {
    // 1.01-2.0 range, 0.01 increment — finest granularity
    let prices = [1.01, 1.25, 1.50, 1.75, 1.99];
    c.bench_function("price_new_low_range", |b| {
        b.iter(|| {
            for &p in &prices {
                black_box(Price::new(black_box(p)).unwrap());
            }
        });
    });
}

fn price_new_mid_range(c: &mut Criterion) {
    // 6.0-10.0 range, 0.2 increment
    let prices = [6.0, 7.2, 8.4, 9.6];
    c.bench_function("price_new_mid_range", |b| {
        b.iter(|| {
            for &p in &prices {
                black_box(Price::new(black_box(p)).unwrap());
            }
        });
    });
}

fn price_new_high_range(c: &mut Criterion) {
    // 100-1000 range, 10.0 increment — last match arm
    let prices = [100.0, 300.0, 500.0, 750.0, 1000.0];
    c.bench_function("price_new_high_range", |b| {
        b.iter(|| {
            for &p in &prices {
                black_box(Price::new(black_box(p)).unwrap());
            }
        });
    });
}

fn size_new(c: &mut Criterion) {
    let sizes = [1.234567, 99.999, 0.01, 42.555, 1000.005];
    c.bench_function("size_new", |b| {
        b.iter(|| {
            for &s in &sizes {
                black_box(Size::new(black_box(s)));
            }
        });
    });
}

fn f64ord_hash(c: &mut Criterion) {
    let values: Vec<F64Ord> = (0..100).map(|i| F64Ord::new(i as f64 * 0.5 + 1.0)).collect();
    c.bench_function("f64ord_hash", |b| {
        b.iter(|| {
            for v in &values {
                let mut hasher = std::collections::hash_map::DefaultHasher::new();
                black_box(v).hash(&mut hasher);
                black_box(hasher.finish());
            }
        });
    });
}

fn f64ord_btreemap_lookup(c: &mut Criterion) {
    // Simulate Available cache: ~50-entry BTreeMap keyed by F64Ord
    let mut map = BTreeMap::new();
    for i in 0..50 {
        let key = F64Ord::new(1.0 + i as f64 * 0.02);
        map.insert(key, i as f64 * 10.0);
    }
    let lookup_keys: Vec<F64Ord> = (0..50).map(|i| F64Ord::new(1.0 + i as f64 * 0.02)).collect();

    c.bench_function("f64ord_btreemap_lookup", |b| {
        b.iter(|| {
            for key in &lookup_keys {
                black_box(map.get(black_box(key)));
            }
        });
    });
}

criterion_group!(
    benches,
    price_new_low_range,
    price_new_mid_range,
    price_new_high_range,
    size_new,
    f64ord_hash,
    f64ord_btreemap_lookup,
);
criterion_main!(benches);
```

**Step 3: Verify it compiles and runs**

Run: `cargo bench -p betfair-types --bench numeric -- --test`
Expected: All 6 benchmarks listed, each runs a single iteration (--test mode).

**Step 4: Run full benchmark**

Run: `cargo bench -p betfair-types --bench numeric`
Expected: Criterion output with timing for all 6 benchmarks.

**Step 5: Commit**

```
feat(bench): add numeric type benchmarks for Price, Size, F64Ord
```

---

### Task 2: Add `betfair-stream-types` serialization benchmarks

**Files:**
- Modify: `crates/betfair-stream-types/Cargo.toml`
- Create: `crates/betfair-stream-types/benches/serialize.rs`

**Step 1: Add criterion dev-dependency and bench config to Cargo.toml**

Append to `crates/betfair-stream-types/Cargo.toml`:

```toml
# under [dev-dependencies]
criterion.workspace = true

[[bench]]
name = "serialize"
harness = false
```

**Step 2: Write the benchmark file**

Create `crates/betfair-stream-types/benches/serialize.rs`:

```rust
use betfair_stream_types::request::RequestMessage;
use betfair_stream_types::request::authentication_message::AuthenticationMessage;
use betfair_stream_types::request::heartbeat_message::HeartbeatMessage;
use betfair_stream_types::request::market_subscription_message::{
    Fields, MarketDataFilter, MarketFilter, MarketSubscriptionMessage,
};
use betfair_stream_types::request::order_subscription_message::{
    OrderFilter, OrderSubscriptionMessage,
};
use criterion::{Criterion, black_box, criterion_group, criterion_main};

fn ser_market_subscription(c: &mut Criterion) {
    let msg = RequestMessage::MarketSubscription(MarketSubscriptionMessage {
        id: Some(1),
        segmentation_enabled: Some(true),
        clk: None,
        heartbeat_ms: Some(500),
        initial_clk: None,
        market_filter: Some(Box::new(MarketFilter::builder()
            .market_ids(vec!["1.206502771".into(), "1.206502772".into()])
            .event_type_ids(vec!["7".into()])
            .turn_in_play_enabled(true)
            .build())),
        conflate_ms: Some(0),
        market_data_filter: Some(Box::new(MarketDataFilter::builder()
            .fields(vec![
                Fields::ExBestOffers,
                Fields::ExTradedVol,
                Fields::ExLtp,
                Fields::ExMarketDef,
            ])
            .ladder_levels(
                betfair_stream_types::request::market_subscription_message::LadderLevel::new(3)
                    .unwrap(),
            )
            .build())),
    });

    c.bench_function("ser_market_subscription", |b| {
        b.iter(|| {
            black_box(serde_json::to_string(black_box(&msg)).unwrap());
        });
    });
}

fn ser_order_subscription(c: &mut Criterion) {
    let msg = RequestMessage::OrderSubscription(OrderSubscriptionMessage {
        id: Some(2),
        segmentation_enabled: Some(true),
        order_filter: Some(Box::new(OrderFilter::builder()
            .include_overall_position(true)
            .partition_matched_by_strategy_ref(false)
            .build())),
        clk: None,
        heartbeat_ms: Some(500),
        initial_clk: None,
        conflate_ms: Some(0),
    });

    c.bench_function("ser_order_subscription", |b| {
        b.iter(|| {
            black_box(serde_json::to_string(black_box(&msg)).unwrap());
        });
    });
}

fn ser_authentication(c: &mut Criterion) {
    let msg = RequestMessage::Authentication(AuthenticationMessage {
        id: Some(1),
        session: "xxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx".to_owned(),
        app_key: "qa{n}pCPTV]EYTLGVO".to_owned(),
    });

    c.bench_function("ser_authentication", |b| {
        b.iter(|| {
            black_box(serde_json::to_string(black_box(&msg)).unwrap());
        });
    });
}

fn ser_heartbeat(c: &mut Criterion) {
    let msg = RequestMessage::Heartbeat(HeartbeatMessage { id: Some(1) });

    c.bench_function("ser_heartbeat", |b| {
        b.iter(|| {
            black_box(serde_json::to_string(black_box(&msg)).unwrap());
        });
    });
}

criterion_group!(
    benches,
    ser_market_subscription,
    ser_order_subscription,
    ser_authentication,
    ser_heartbeat,
);
criterion_main!(benches);
```

**Step 3: Verify it compiles and runs**

Run: `cargo bench -p betfair-stream-types --bench serialize -- --test`
Expected: All 4 benchmarks listed.

**Step 4: Run full benchmark**

Run: `cargo bench -p betfair-stream-types --bench serialize`
Expected: Criterion output with timing for all 4 benchmarks.

**Step 5: Commit**

```
feat(bench): add request serialization benchmarks for stream types
```

---

### Task 3: Add `betfair-stream-api` codec encode benchmarks

**Files:**
- Modify: `crates/betfair-stream-api/Cargo.toml` (add `[[bench]]` entry)
- Create: `crates/betfair-stream-api/benches/codec.rs`

**Step 1: Add bench entry to Cargo.toml**

Append to the existing `[[bench]]` entries in `crates/betfair-stream-api/Cargo.toml`:

```toml
[[bench]]
name = "codec"
harness = false
```

**Step 2: Write the benchmark file**

Create `crates/betfair-stream-api/benches/codec.rs`:

```rust
use betfair_stream_api::StreamAPIClientCodec;
use betfair_stream_types::request::RequestMessage;
use betfair_stream_types::request::heartbeat_message::HeartbeatMessage;
use betfair_stream_types::request::market_subscription_message::{
    Fields, MarketDataFilter, MarketFilter, MarketSubscriptionMessage,
};
use bytes::BytesMut;
use criterion::{BatchSize, Criterion, black_box, criterion_group, criterion_main};
use tokio_util::codec::Encoder;

fn codec_encode_market_subscription(c: &mut Criterion) {
    let msg = RequestMessage::MarketSubscription(MarketSubscriptionMessage {
        id: Some(1),
        segmentation_enabled: Some(true),
        clk: None,
        heartbeat_ms: Some(500),
        initial_clk: None,
        market_filter: Some(Box::new(MarketFilter::builder()
            .market_ids(vec!["1.206502771".into(), "1.206502772".into()])
            .event_type_ids(vec!["7".into()])
            .turn_in_play_enabled(true)
            .build())),
        conflate_ms: Some(0),
        market_data_filter: Some(Box::new(MarketDataFilter::builder()
            .fields(vec![
                Fields::ExBestOffers,
                Fields::ExTradedVol,
                Fields::ExLtp,
                Fields::ExMarketDef,
            ])
            .build())),
    });

    c.bench_function("codec_encode_market_subscription", |b| {
        b.iter_batched(
            || (StreamAPIClientCodec, BytesMut::with_capacity(512), msg.clone()),
            |(mut codec, mut buf, msg)| {
                codec.encode(msg, &mut buf).unwrap();
                black_box(buf);
            },
            BatchSize::SmallInput,
        );
    });
}

fn codec_encode_heartbeat(c: &mut Criterion) {
    let msg = RequestMessage::Heartbeat(HeartbeatMessage { id: Some(1) });

    c.bench_function("codec_encode_heartbeat", |b| {
        b.iter_batched(
            || (StreamAPIClientCodec, BytesMut::with_capacity(64), msg.clone()),
            |(mut codec, mut buf, msg)| {
                codec.encode(msg, &mut buf).unwrap();
                black_box(buf);
            },
            BatchSize::SmallInput,
        );
    });
}

criterion_group!(benches, codec_encode_market_subscription, codec_encode_heartbeat);
criterion_main!(benches);
```

**Step 3: Verify `StreamAPIClientCodec` is publicly accessible**

The codec is defined in `betfair-stream-api/src/lib.rs` as `pub struct StreamAPIClientCodec`. The `Encoder` impl uses `RequestMessage` which is re-exported from `betfair-stream-types`. Verify the bench can import `StreamAPIClientCodec` — if it's not publicly exported, add `pub use` in `lib.rs`.

Run: `cargo bench -p betfair-stream-api --bench codec -- --test`
Expected: Both benchmarks listed. If compile error about visibility, fix the export.

**Step 4: Run full benchmark**

Run: `cargo bench -p betfair-stream-api --bench codec`
Expected: Criterion output for both benchmarks.

**Step 5: Commit**

```
feat(bench): add codec encode benchmarks for streaming API
```

---

### Task 4: Add `betfair-adapter` async RPC benchmarks

**Files:**
- Modify: `crates/betfair-adapter/Cargo.toml`
- Create: `crates/betfair-adapter/benches/rpc.rs`

**Step 1: Add dev-dependencies and bench config to Cargo.toml**

Add to `crates/betfair-adapter/Cargo.toml`:

```toml
# under [dev-dependencies]
criterion.workspace = true
betfair-rpc-server-mock.workspace = true
serde_json.workspace = true

[[bench]]
name = "rpc"
harness = false
```

Note: `tokio` is already a dev-dependency. `betfair-rpc-server-mock` needs a workspace entry — check if it exists, if not add `betfair-rpc-server-mock = { path = "crates/betfair-rpc-server-mock" }` to workspace dependencies.

**Step 2: Ensure `betfair-rpc-server-mock` is in workspace deps**

Check the root `Cargo.toml` `[workspace.dependencies]` section. If `betfair-rpc-server-mock` is not listed, add:

```toml
betfair-rpc-server-mock = { path = "crates/betfair-rpc-server-mock" }
```

**Step 3: Write the benchmark file**

Create `crates/betfair-adapter/benches/rpc.rs`:

```rust
use betfair_rpc_server_mock::Server;
use betfair_types::types::sports_aping::list_market_book;
use criterion::{BatchSize, Criterion, black_box, criterion_group, criterion_main};
use serde_json::json;

fn make_runtime() -> tokio::runtime::Runtime {
    tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .unwrap()
}

fn rpc_send_request_list_market_book(c: &mut Criterion) {
    let rt = make_runtime();

    // Setup: start mock server, authenticate client, mount mock response
    let (client, server) = rt.block_on(async {
        let server = Server::new().await;
        let response = json!([
            {
                "marketId": "1.206502771",
                "isMarketDataDelayed": false,
                "status": "OPEN",
                "betDelay": 0,
                "bspReconciled": false,
                "complete": true,
                "inplay": false,
                "numberOfWinners": 1,
                "numberOfRunners": 2,
                "numberOfActiveRunners": 2,
                "totalMatched": 0.0,
                "totalAvailable": 153.6,
                "crossMatching": true,
                "runnersVoidable": false,
                "version": 4910,
                "runners": [
                    {
                        "selectionId": 12062411,
                        "handicap": 0.0,
                        "status": "ACTIVE",
                        "totalMatched": 0.0,
                        "ex": {
                            "availableToBack": [
                                { "price": 1.19, "size": 57.52 },
                                { "price": 1.02, "size": 6.48 },
                                { "price": 1.01, "size": 21.14 }
                            ],
                            "availableToLay": [
                                { "price": 1.43, "size": 31.11 }
                            ],
                            "tradedVolume": []
                        }
                    },
                    {
                        "selectionId": 50310375,
                        "handicap": 0.0,
                        "status": "ACTIVE",
                        "totalMatched": 0.0,
                        "ex": {
                            "availableToBack": [
                                { "price": 3.75, "size": 5.21 }
                            ],
                            "availableToLay": [
                                { "price": 4.8, "size": 4.37 }
                            ],
                            "tradedVolume": []
                        }
                    }
                ]
            }
        ]);
        server
            .mock_authenticated_rpc_from_json::<list_market_book::Parameters>(response)
            .mount(&server.bf_api_mock_server)
            .await;

        let unauth = server.client().await;
        let (client, _keep_alive) = unauth.authenticate().await.unwrap();
        (client, server)
    });

    c.bench_function("rpc_send_request_list_market_book", |b| {
        b.iter(|| {
            rt.block_on(async {
                let result = client
                    .send_request(
                        list_market_book::Parameters::builder()
                            .market_ids(vec!["1.206502771".into()])
                            .build(),
                    )
                    .await
                    .unwrap();
                black_box(result);
            });
        });
    });

    // Keep server alive for the benchmark duration
    drop(server);
}

fn rpc_bot_login(c: &mut Criterion) {
    let rt = make_runtime();

    let server = rt.block_on(async { Server::new().await });

    c.bench_function("rpc_bot_login", |b| {
        b.iter_batched(
            || rt.block_on(async { server.client().await }),
            |unauth_client| {
                rt.block_on(async {
                    let result = unauth_client.authenticate().await.unwrap();
                    black_box(result);
                });
            },
            BatchSize::SmallInput,
        );
    });

    drop(server);
}

criterion_group!(benches, rpc_send_request_list_market_book, rpc_bot_login);
criterion_main!(benches);
```

Note: We dropped `list_market_catalogue` and kept 2 RPC benchmarks (one authenticated call + bot login). The catalogue response is structurally similar — if you want it added later it's trivial to clone the pattern.

**Step 4: Verify it compiles**

Run: `cargo bench -p betfair-adapter --bench rpc -- --test`
Expected: Both benchmarks listed. May need to adjust imports if types aren't re-exported.

**Step 5: Run full benchmark**

Run: `cargo bench -p betfair-adapter --bench rpc`
Expected: Criterion output. RPC benchmarks will be slower (ms range) due to HTTP overhead.

**Step 6: Commit**

```
feat(bench): add async RPC round-trip benchmarks with mock server
```

---

### Task 5: Update xtask bench command

**Files:**
- Modify: `xtask/src/main.rs`

**Step 1: Update the bench handler**

Replace the bench loop in `xtask/src/main.rs` (lines 141-154) with:

```rust
Args::Bench { save_baseline } => {
    println!("cargo bench");
    let benches = [
        ("betfair-types", "numeric"),
        ("betfair-stream-types", "serialize"),
        ("betfair-stream-api", "deserialize"),
        ("betfair-stream-api", "cache_update"),
        ("betfair-stream-api", "process_message"),
        ("betfair-stream-api", "codec"),
        ("betfair-adapter", "rpc"),
    ];
    for (crate_name, bench_name) in benches {
        if let Some(ref baseline) = save_baseline {
            cmd!(
                sh,
                "cargo bench -p {crate_name} --bench {bench_name} -- --save-baseline {baseline}"
            )
            .run()?;
        } else {
            cmd!(sh, "cargo bench -p {crate_name} --bench {bench_name}").run()?;
        }
    }
}
```

**Step 2: Verify xtask runs all benchmarks**

Run: `cargo xtask bench`
Expected: All 7 bench suites run sequentially. 26 total benchmarks (10 existing + 16 new).

**Step 3: Commit**

```
feat(xtask): update bench command to run all workspace benchmarks
```

---

### Task 6: Run `cargo xtask check` and fix any issues

**Step 1: Run clippy + fmt**

Run: `cargo xtask check`
Expected: Clean. If clippy or fmt issues, fix and re-run.

**Step 2: Run the full bench suite end-to-end**

Run: `cargo xtask bench`
Expected: All benchmarks complete successfully.

**Step 3: Final commit (if any fixes needed)**

```
style: fix clippy/fmt issues in benchmark files
```
