use criterion::{criterion_group, criterion_main};

fn process_message_benchmarks(_c: &mut criterion::Criterion) {}

criterion_group!(benches, process_message_benchmarks);
criterion_main!(benches);
