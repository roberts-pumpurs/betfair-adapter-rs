use criterion::{criterion_group, criterion_main};

fn cache_update_benchmarks(_c: &mut criterion::Criterion) {}

criterion_group!(benches, cache_update_benchmarks);
criterion_main!(benches);
