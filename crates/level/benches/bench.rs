use criterion::{criterion_group, criterion_main, Criterion};
use std::hint::black_box;

use dnd_rs_level::Level;

fn criterion_benchmark(c: &mut Criterion) {
    c.bench_function("generate 20x20 level", |b| {
        b.iter(|| Level::random(black_box(20), black_box(20)))
    });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
