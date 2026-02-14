use std::path::Path;

use betfair_stream_api::cache::tracker::StreamState;
use betfair_stream_types::response::ResponseMessage;
use betfair_stream_types::response::market_change_message::MarketChangeMessage;
use criterion::{BatchSize, Criterion, black_box, criterion_group, criterion_main};

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

fn cache_update_delta(c: &mut Criterion) {
    // Populate cache with full image first
    let image_json = fixture("streaming_mcm_SUB_IMAGE.json");
    let image_msg = parse_market_change(&image_json);

    let mut state = StreamState::new();
    state.market_change_update(image_msg);

    // Delta: single runner price update (market ID must match the image fixture: "1.126235656")
    let delta_json = r#"{"op":"mcm","id":2,"clk":"AAAAAAAB","pt":1471370160471,"mc":[{"id":"1.126235656","rc":[{"batb":[[0,2.56,8.00]],"id":11131804}]}]}"#;
    let delta_msg = parse_market_change(delta_json);

    c.bench_function("cache_update_delta", |b| {
        b.iter_batched(
            || (state.clone(), delta_msg.clone()),
            |(mut s, msg)| {
                black_box(s.market_change_update(msg));
            },
            BatchSize::SmallInput,
        );
    });
}

fn cache_update_full_image(c: &mut Criterion) {
    let json = fixture("streaming_mcm_SUB_IMAGE.json");
    let msg = parse_market_change(&json);

    c.bench_function("cache_update_full_image", |b| {
        b.iter_batched(
            || (StreamState::new(), msg.clone()),
            |(mut s, msg)| {
                black_box(s.market_change_update(msg));
            },
            BatchSize::SmallInput,
        );
    });
}

fn cache_update_market_definition(c: &mut Criterion) {
    // Populate cache first
    let image_json = fixture("streaming_mcm_SUB_IMAGE.json");
    let image_msg = parse_market_change(&image_json);
    let mut state = StreamState::new();
    state.market_change_update(image_msg);

    // Re-use the same image as definition update (it contains market definition with runners)
    let def_json = fixture("streaming_mcm_SUB_IMAGE.json");
    let def_msg = parse_market_change(&def_json);

    c.bench_function("cache_update_market_definition", |b| {
        b.iter_batched(
            || (state.clone(), def_msg.clone()),
            |(mut s, msg)| {
                black_box(s.market_change_update(msg));
            },
            BatchSize::SmallInput,
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
