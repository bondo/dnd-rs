use criterion::{criterion_group, criterion_main, Criterion};
use std::hint::black_box;

use dnd_rs_level::{IterativeSolver, Level, RecursiveSolver};

fn run_level_generation_benchmark(c: &mut Criterion) {
    c.bench_function("generate 50x50 level", |b| {
        b.iter(|| Level::random(black_box(50), black_box(50)))
    });
}

fn run_recursive_solver_benchmark(c: &mut Criterion) {
    c.bench_function("solve 8x8 level (recursive)", |b| {
        let level = Level::random(8, 8);
        b.iter(|| RecursiveSolver::from_level(&level).first_solution())
    });
}

fn run_iterative_solver_benchmark(c: &mut Criterion) {
    c.bench_function("solve 8x8 level (iterative)", |b| {
        let level = Level::random(8, 8);
        b.iter(|| IterativeSolver::from_level(&level).first_solution())
    });
}

criterion_group!(
    benches,
    run_level_generation_benchmark,
    run_recursive_solver_benchmark,
    run_iterative_solver_benchmark
);
criterion_main!(benches);
