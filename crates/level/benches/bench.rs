use criterion::{criterion_group, criterion_main, BatchSize, Criterion};
// use std::hint::black_box;

use dnd_rs_level::{Level, Solver};

// fn run_level_generation_benchmark(c: &mut Criterion) {
//     let mut seed = 0;
//     c.bench_function("generate 50x50 level", |b| {
//         b.iter(|| {
//             seed += 1;
//             fastrand::seed(seed);

//             Level::random(black_box(50), black_box(50))
//         })
//     });
// }

fn run_solver_benchmarks(c: &mut Criterion) {
    fastrand::seed(1337);
    c.bench_function("solve 8x8 level", |b| {
        b.iter_batched_ref(
            || Level::random(8, 8),
            |level| Solver::from_level(level).first_solution(),
            BatchSize::SmallInput,
        );
    });
}

criterion_group!(
    benches,
    // run_level_generation_benchmark,
    run_solver_benchmarks,
);
criterion_main!(benches);
