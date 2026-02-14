use std::path::Path;
use std::sync::Arc;

use betfair_stream_api::cache::primitives::MarketBookCache;
use betfair_stream_api::cache::tracker::StreamState;
use betfair_stream_api::{Cache, MessageProcessor};
use betfair_stream_types::response::ResponseMessage;
use betfair_stream_types::response::market_change_message::MarketChangeMessage;
use criterion::{Criterion, black_box, criterion_group, criterion_main};

fn fixture(name: &str) -> String {
    let path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../betfair-stream-types/tests/resources")
        .join(name);
    std::fs::read_to_string(&path)
        .unwrap_or_else(|e| panic!("failed to read fixture {}: {e}", path.display()))
}

fn parse_market_change(json: &str) -> MarketChangeMessage {
    match serde_json::from_str::<ResponseMessage>(json).unwrap() {
        ResponseMessage::MarketChange(msg) => msg,
        other => panic!("expected MarketChange, got {other:?}"),
    }
}

/// Full process_message pipeline with a delta (the most common hot path)
fn process_message_delta(c: &mut Criterion) {
    let image_json = fixture("streaming_mcm_SUB_IMAGE.json");
    let image_msg: ResponseMessage = serde_json::from_str(&image_json).unwrap();

    // Warm up cache with full image
    let mut cache = Cache::new();
    cache.process_message(image_msg);

    // Delta message
    let delta_json = r#"{"op":"mcm","id":2,"clk":"AHMAcArtjjje","pt":1471370160471,"mc":[{"id":"1.126235656","rc":[{"atb":[[1.01,200]],"id":11131804}]}]}"#;
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

/// Full process_message pipeline with a full image
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

/// Isolate the cost of cloning Arc-wrapped MarketBookCache (measures P0 optimization)
fn cache_clone_isolated(c: &mut Criterion) {
    let image_json = fixture("streaming_mcm_SUB_IMAGE.json");
    let image_msg = parse_market_change(&image_json);

    let mut state = StreamState::new();
    let owned = state.market_change_update(image_msg).unwrap();

    c.bench_function("cache_clone_isolated", |b| {
        b.iter(|| {
            let cloned: Vec<Arc<MarketBookCache>> =
                black_box(&owned).iter().map(Arc::clone).collect();
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
