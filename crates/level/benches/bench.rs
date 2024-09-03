use criterion::{criterion_group, criterion_main, BatchSize, Criterion};
use std::hint::black_box;

use dnd_rs_level::{Level, Solver};

fn run_level_generation_benchmark(c: &mut Criterion) {
    let mut seed = 0;
    c.bench_function("generate 50x50 level", |b| {
        b.iter(|| {
            seed += 1;
            fastrand::seed(seed);

            Level::random(black_box(50), black_box(50)).unwrap()
        })
    });
}

fn run_solver_benchmarks(c: &mut Criterion) {
    fastrand::seed(1337);
    c.bench_function("solve 10x10 level", |b| {
        b.iter_batched_ref(
            || Level::random(10, 10).unwrap(),
            |level| Solver::from_level(level).first_solution(),
            BatchSize::SmallInput,
        );
    });

    // Compare to https://github.com/MischaU8/dungeons_diagrams/tree/bf29a0454aec28476ac80286e130feeaa4081dec?tab=readme-ov-file#usage
    c.bench_function("solve nimble example", |b| {
        b.iter(|| {
            Solver::try_from(black_box(
                r#"
  1 4 2 7 0 4 4 4
3 ? ? ? ? ? ? ? ?
2 ? ? ? ? ? ? ? M
5 ? ? M ? ? ? ? ?
3 ? ? ? ? ? ? ? M
4 ? ? ? ? ? ? ? ?
1 ? T ? ? ? ? ? M
4 ? ? ? ? ? ? ? ?
4 ? ? ? ? ? ? ? M
"#,
            ))
            .unwrap()
            .first_solution()
            .unwrap()
        });
    });
}

criterion_group!(
    benches,
    run_level_generation_benchmark,
    run_solver_benchmarks,
);
criterion_main!(benches);
