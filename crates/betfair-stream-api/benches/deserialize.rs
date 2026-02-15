use std::path::Path;

use betfair_stream_types::response::ResponseMessage;
use std::hint::black_box;

use criterion::{Criterion, criterion_group, criterion_main};

fn fixture(name: &str) -> String {
    let path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../betfair-stream-types/tests/resources")
        .join(name);
    std::fs::read_to_string(&path)
        .unwrap_or_else(|e| panic!("failed to read fixture {}: {e}", path.display()))
}

fn deser_market_change_delta(c: &mut Criterion) {
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
