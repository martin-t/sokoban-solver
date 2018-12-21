#[macro_use]
extern crate criterion;

extern crate sokoban_solver;

use criterion::{Benchmark, Criterion};

use sokoban_solver::config::Method;
use sokoban_solver::{LoadLevel, Solve};

// allowing unused so i can bench just one or a few
// and still notice other warnings if there are any
#[allow(unused)]
fn bench_pushes_boxxle1_1(c: &mut Criterion) {
    // 3 goals in a row
    bench_level(c, Method::Pushes, "levels/boxxle1/1.txt", 150);
}

#[allow(unused)]
fn bench_pushes_boxxle1_5(c: &mut Criterion) {
    // 4 box goal room
    bench_level(c, Method::Pushes, "levels/boxxle1/5.txt", 75);
}

#[allow(unused)]
fn bench_pushes_boxxle1_18(c: &mut Criterion) {
    // 6 boxes - tiny goalroom with 2 entrances
    bench_level(c, Method::Pushes, "levels/boxxle1/18.txt", 15);
}

#[allow(unused)]
fn bench_pushes_boxxle1_25(c: &mut Criterion) {
    // 7 box goal room
    bench_level(c, Method::Pushes, "levels/boxxle1/25.txt", 10);
}

#[allow(unused)]
fn bench_pushes_boxxle1_29(c: &mut Criterion) {
    // 8 boxes - goal area with 4 entrances
    bench_level(c, Method::Pushes, "levels/boxxle1/29.txt", 5);
}

#[allow(unused)]
fn bench_pushes_boxxle1_108(c: &mut Criterion) {
    // 6 boxes in the middle
    bench_level(c, Method::Pushes, "levels/boxxle1/108.txt", 25);
}

#[allow(unused)]
fn bench_pushes_boxxle2_3(c: &mut Criterion) {
    // 5 separate goals
    bench_level(c, Method::Pushes, "levels/boxxle2/3.txt", 75);
}

#[allow(unused)]
fn bench_pushes_boxxle2_4(c: &mut Criterion) {
    // 13 goals in a checkerboard
    bench_level(c, Method::Pushes, "levels/boxxle2/4.txt", 10);
}

#[allow(unused)]
fn bench_pushes_custom_remover_original_1(c: &mut Criterion) {
    let level = "levels/custom/remover-original-01.txt";
    bench_level(c, Method::Pushes, level, 10);
}

#[allow(unused)]
fn bench_moves_boxxle1_1(c: &mut Criterion) {
    // 3 goals in a row
    bench_level(c, Method::Moves, "levels/boxxle1/1.txt", 150);
}

fn bench_level(c: &mut Criterion, method: Method, level_path: &str, samples: usize) {
    let level = level_path.load_level().unwrap();

    c.bench(
        &format!("{}", method),
        Benchmark::new(level_path, move |b| {
            b.iter(|| {
                criterion::black_box(Solve::solve(
                    criterion::black_box(&level),
                    criterion::black_box(method),
                    criterion::black_box(false),
                ))
            })
        })
        .sample_size(samples),
    );
}

criterion_group!(
    benches,
    bench_pushes_boxxle1_1,
    bench_pushes_boxxle1_5,
    bench_pushes_boxxle1_18,
    bench_pushes_boxxle1_25,
    bench_pushes_boxxle1_29,
    bench_pushes_boxxle1_108,
    bench_pushes_boxxle2_3,
    bench_pushes_boxxle2_4,
    bench_pushes_custom_remover_original_1,
    bench_moves_boxxle1_1,
);
criterion_main!(benches);
