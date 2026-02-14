use criterion::{criterion_group, criterion_main};

fn deserialize_benchmarks(_c: &mut criterion::Criterion) {}

criterion_group!(benches, deserialize_benchmarks);
criterion_main!(benches);
