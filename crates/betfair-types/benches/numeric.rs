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
