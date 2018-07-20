#[macro_use]
extern crate criterion;

extern crate sokoban_solver;

use criterion::{Benchmark, Criterion};

use sokoban_solver::config::Method;
use sokoban_solver::{LoadLevel, Solve};

// allowing unused so i can bench just one or few
// and still notice other warnings if there are any
#[allow(unused)]
fn benech_boxxle_1(c: &mut Criterion) {
    // 3 goals in a row
    bench_level(c, Method::Pushes, "levels/boxxle1/1.txt", 100);
}

#[allow(unused)]
fn benech_boxxle_5(c: &mut Criterion) {
    // 4 boxes goal room
    bench_level(c, Method::Pushes, "levels/boxxle1/5.txt", 100);
}

#[allow(unused)]
fn benech_boxxle_18(c: &mut Criterion) {
    // 6 boxes - tiny goalroom
    bench_level(c, Method::Pushes, "levels/boxxle1/18.txt", 25);
}

#[allow(unused)]
fn benech_boxxle_108(c: &mut Criterion) {
    // 6 boxes in the middle
    bench_level(c, Method::Pushes, "levels/boxxle1/108.txt", 50);
}

#[allow(unused)]
fn benech_boxxle_1_moves(c: &mut Criterion) {
    // 3 goals in a row
    bench_level(c, Method::Moves, "levels/boxxle1/1.txt", 100);
}

fn bench_level(c: &mut Criterion, method: Method, level_path: &str, samples: usize) {
    let level = level_path.load_level().unwrap();

    c.bench(
        &format!("{}", method),
        Benchmark::new(level_path, move |b| {
            b.iter(|| {
                criterion::black_box(
                    level.solve(criterion::black_box(method), criterion::black_box(false)),
                )
            })
        }).sample_size(samples),
    );
}

criterion_group!(
    benches,
    benech_boxxle_1,
    //benech_boxxle_5,
    //benech_boxxle_18,
    //benech_boxxle_108,
    //benech_boxxle_1_moves,
);
criterion_main!(benches);
