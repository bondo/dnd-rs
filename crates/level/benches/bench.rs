use criterion::{criterion_group, criterion_main, BatchSize, Criterion};
use std::hint::black_box;

use dnd_rs_level::{IterativeSolver, Level, RecursiveSolver};

fn run_level_generation_benchmark(c: &mut Criterion) {
    let mut seed = 0;
    c.bench_function("generate 50x50 level", |b| {
        b.iter(|| {
            seed += 1;
            fastrand::seed(seed);

            Level::random(black_box(50), black_box(50))
        })
    });
}

fn run_solver_benchmarks(c: &mut Criterion) {
    let mut g = c.benchmark_group("solve 8x8 level");

    fastrand::seed(1337);
    g.bench_function("recursive solver", |b| {
        b.iter_batched_ref(
            || Level::random(8, 8),
            |level| RecursiveSolver::from_level(level).first_solution(),
            BatchSize::SmallInput,
        );
    });

    fastrand::seed(1337);
    g.bench_function("iterative solver", |b| {
        b.iter_batched_ref(
            || Level::random(8, 8),
            |level| IterativeSolver::from_level(level).first_solution(),
            BatchSize::SmallInput,
        );
    });
}

criterion_group!(
    benches,
    run_level_generation_benchmark,
    run_solver_benchmarks,
);
criterion_main!(benches);
