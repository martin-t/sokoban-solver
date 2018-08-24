// https://github.com/rust-lang/rust/issues/31844
#![feature(specialization)]
// Opt in to warnings about new 2018 idioms
#![warn(rust_2018_idioms)]
// Additional warnings that are allow by default (`rustc -W help`)
#![warn(missing_copy_implementations)]
#![warn(missing_debug_implementations)]
#![warn(trivial_casts)]
#![warn(trivial_numeric_casts)]
#![warn(unreachable_pub)]
#![warn(unused)]
// Clippy
#![allow(unknown_lints)] // necessary because rustc doesn't know about clippy
#![warn(clippy)]
// Stuff for testing
#![cfg_attr(test, feature(duration_as_u128))]
#![cfg_attr(test, feature(test))]

pub mod config;
pub mod level;
pub mod map_formatter;
pub mod moves;
pub mod solution_formatter;
pub mod solver;

mod data;
mod map;
mod parser;
mod state;
mod vec2d;

use std::error::Error;

use crate::config::Method;
use crate::level::Level;
use crate::solver::{SolverErr, SolverOk};

pub trait LoadLevel {
    fn load_level(&self) -> Result<Level, Box<dyn Error>>;
}

pub trait Solve {
    fn solve(&self, method: Method, print_status: bool) -> Result<SolverOk, SolverErr>;
}

#[cfg(test)]
mod tests {
    use test::{self, Bencher};

    use std::fs;
    use std::path::Path;

    use separator::Separatable;

    use crate::config::Method::{self, MoveOptimal, PushOptimal};

    use super::*;

    #[test]
    fn test_levels() {
        const UNSOLVED: i32 = 5;
        const REMOVER: i32 = 5;
        const VERY_SLOW: i32 = 4;
        const SLOW: i32 = 3;
        const VERY_SLOW_IN_DEBUG: i32 = 2;
        const SLOW_IN_DEBUG: i32 = 1;
        const OK: i32 = 0;

        #[cfg(debug_assertions)]
        const MAX_DIFFICULTY: i32 = 0;

        #[cfg(not(debug_assertions))]
        const MAX_DIFFICULTY: i32 = 2; // set to 4 to update all levels

        let levels = [
            (PushOptimal, "custom", "00-solved.txt", OK),
            (PushOptimal, "custom", "01-simplest-custom.txt", OK),
            (PushOptimal, "custom", "01-simplest-xsb.txt", OK),
            (PushOptimal, "custom", "02-one-way-xsb.txt", OK),
            (PushOptimal, "custom", "02-one-way.txt", OK),
            (PushOptimal, "custom", "03-long-way.txt", OK),
            (PushOptimal, "custom", "04-two-boxes-no-packing.txt", OK),
            (PushOptimal, "custom", "04-two-boxes-remover.txt", REMOVER),
            (PushOptimal, "custom", "04-two-boxes.txt", OK),
            (PushOptimal, "custom", "no-solution-parking.txt", OK),
            (PushOptimal, "custom", "original-01-remover.txt", REMOVER),
            (PushOptimal, "custom", "supaplex.txt", REMOVER),
            (PushOptimal, "custom", "supaplex-goals.txt", VERY_SLOW),
            (PushOptimal, "boxxle1", "1.txt", OK),
            (PushOptimal, "boxxle1", "2.txt", OK),
            (PushOptimal, "boxxle1", "3.txt", OK),
            (PushOptimal, "boxxle1", "4.txt", OK),
            (PushOptimal, "boxxle1", "5.txt", OK),
            (PushOptimal, "boxxle1", "6.txt", VERY_SLOW_IN_DEBUG),
            (PushOptimal, "boxxle1", "7.txt", OK),
            (PushOptimal, "boxxle1", "8.txt", OK),
            (PushOptimal, "boxxle1", "9.txt", SLOW_IN_DEBUG),
            (PushOptimal, "boxxle1", "10.txt", OK),
            (PushOptimal, "boxxle1", "11.txt", OK),
            (PushOptimal, "boxxle1", "12.txt", VERY_SLOW),
            (PushOptimal, "boxxle1", "13.txt", OK),
            (PushOptimal, "boxxle1", "14.txt", UNSOLVED),
            (PushOptimal, "boxxle1", "15.txt", OK),
            (PushOptimal, "boxxle1", "16.txt", UNSOLVED),
            (PushOptimal, "boxxle1", "17.txt", VERY_SLOW_IN_DEBUG),
            (PushOptimal, "boxxle1", "18.txt", SLOW_IN_DEBUG),
            (PushOptimal, "boxxle1", "19.txt", OK),
            (PushOptimal, "boxxle1", "20.txt", OK),
            (PushOptimal, "boxxle1", "21.txt", OK),
            (PushOptimal, "boxxle1", "22.txt", UNSOLVED),
            (PushOptimal, "boxxle1", "23.txt", SLOW_IN_DEBUG),
            (PushOptimal, "boxxle1", "24.txt", UNSOLVED),
            (PushOptimal, "boxxle1", "25.txt", SLOW),
            (PushOptimal, "boxxle1", "26.txt", UNSOLVED),
            (PushOptimal, "boxxle1", "27.txt", OK),
            (PushOptimal, "boxxle1", "28.txt", SLOW_IN_DEBUG),
            (PushOptimal, "boxxle1", "29.txt", SLOW),
            (PushOptimal, "boxxle1", "30.txt", UNSOLVED),
            (PushOptimal, "boxxle1", "108.txt", OK),
            (PushOptimal, "boxxle2", "1.txt", OK),
            (PushOptimal, "boxxle2", "2.txt", OK),
            (PushOptimal, "boxxle2", "3.txt", OK),
            (PushOptimal, "boxxle2", "4.txt", OK),
            (PushOptimal, "boxxle2", "5.txt", UNSOLVED),
            (PushOptimal, "boxxle2", "6.txt", VERY_SLOW_IN_DEBUG),
            (PushOptimal, "boxxle2", "7.txt", SLOW),
            (PushOptimal, "boxxle2", "8.txt", UNSOLVED),
            (PushOptimal, "boxxle2", "9.txt", UNSOLVED),
            (PushOptimal, "boxxle2", "10.txt", UNSOLVED),
            (PushOptimal, "original", "1.txt", VERY_SLOW),
            (MoveOptimal, "custom", "00-solved.txt", OK),
            (MoveOptimal, "custom", "01-simplest-custom.txt", OK),
            (MoveOptimal, "custom", "01-simplest-xsb.txt", OK),
            (MoveOptimal, "custom", "02-one-way-xsb.txt", OK),
            (MoveOptimal, "custom", "02-one-way.txt", OK),
            (MoveOptimal, "custom", "03-long-way.txt", OK),
            (MoveOptimal, "custom", "04-two-boxes-no-packing.txt", OK),
            (MoveOptimal, "custom", "04-two-boxes-remover.txt", REMOVER),
            (MoveOptimal, "custom", "04-two-boxes.txt", OK),
            (MoveOptimal, "custom", "no-solution-parking.txt", OK),
            (MoveOptimal, "custom", "original-01-remover.txt", REMOVER),
            (MoveOptimal, "custom", "supaplex.txt", REMOVER),
            (MoveOptimal, "custom", "supaplex-goals.txt", UNSOLVED),
            (MoveOptimal, "boxxle1", "1.txt", OK),
            (MoveOptimal, "boxxle1", "2.txt", SLOW_IN_DEBUG),
            (MoveOptimal, "boxxle1", "3.txt", OK),
            (MoveOptimal, "boxxle1", "4.txt", OK),
            (MoveOptimal, "boxxle1", "5.txt", OK),
            (MoveOptimal, "boxxle1", "6.txt", SLOW),
            (MoveOptimal, "boxxle1", "7.txt", SLOW_IN_DEBUG),
            (MoveOptimal, "boxxle1", "8.txt", OK),
            (MoveOptimal, "boxxle1", "9.txt", SLOW),
            (MoveOptimal, "boxxle1", "10.txt", OK),
        ];

        let levels: Vec<_> = levels
            .iter()
            .filter(|&&(_, _, _, difficulty)| difficulty <= MAX_DIFFICULTY)
            .collect();
        let succeeded = levels
            .iter()
            .filter(|&(method, level_pack, level_name, _)| {
                test_level(*method, level_pack, level_name)
            }).count();
        assert_eq!(succeeded, levels.len());
    }

    fn test_level(method: Method, level_pack: &str, level_name: &str) -> bool {
        // for updating results more easily
        // (need to update when equal too because the file includes individual depths)
        #![allow(collapsible_if)]

        use std::fmt::Write;
        use std::time::Instant;

        let method_name = method.to_string();
        let level_path = format!("levels/{}/{}", level_pack, level_name);
        let result_dir = format!("solutions/{}/{}", method_name, level_pack);
        let result_file = format!("{}/{}", result_dir, level_name);

        println!("Solving {} using {}", level_path, method_name);
        let started = Instant::now();

        let level = level_path.load_level().unwrap();
        let solution = level.solve(method, false).unwrap();

        // innacurate, only useful to quickly see which levels are difficult
        println!(
            "Solved {} using {} in approximately {} ms",
            level_path,
            method_name,
            (started.elapsed().as_millis() as u64).separated_string(), // separator doesn't support u128
        );

        let mut out = String::new();
        match solution.moves {
            None => writeln!(out, "No solution").unwrap(),
            Some(ref moves) => {
                writeln!(out, "{}", moves);
                writeln!(out, "Moves: {}", moves.move_cnt()).unwrap();
                writeln!(out, "Pushes: {}", moves.push_cnt()).unwrap();
            }
        }
        writeln!(out, "{}", solution.stats).unwrap();
        if let Some(ref moves) = solution.moves {
            let include_steps = method == Method::MoveOptimal;
            write!(out, "{}", level.xsb_solution(moves, include_steps)).unwrap();
        }

        if !Path::new(&result_dir).exists() {
            fs::create_dir_all(&result_dir).unwrap();
        }

        if !Path::new(&result_file).exists() {
            fs::write(&result_file, &out).unwrap();
            print!("Solution:\n{}", out);
            println!("         >>> SAVED NEW SOLUTION <<<");
        }

        let expected = fs::read_to_string(&result_file).unwrap();
        if out != expected {
            print!("Expected:\n{}", expected);
            print!("Got:\n{}", out);

            // other stats can go up with a better solution
            let (maybe_out_lens, out_created, out_visited) = parse_stats(&out);
            let (maybe_expected_lens, expected_created, expected_visited) = parse_stats(&expected);
            if maybe_out_lens.is_some() != maybe_expected_lens.is_some() {
                println!("         >>> SOLVABILITY CHANGED <<<\n\n");
            } else {
                let (out_moves, out_pushes) = maybe_out_lens.unwrap_or((-1, -1));
                let (expected_moves, expected_pushes) = maybe_expected_lens.unwrap_or((-1, -1));
                if out_moves > expected_moves
                    || out_pushes > expected_pushes
                    || out_created > expected_created
                    || out_visited > expected_visited
                {
                    println!("         >>> WORSE <<<\n\n");
                } else {
                    if out_moves == expected_moves
                        && out_pushes == expected_pushes
                        && out_created == expected_created
                        && out_visited == expected_visited
                    {
                        println!("         >>> EQUAL <<<\n\n");
                    } else {
                        println!("         >>> BETTER <<<\n\n");
                    }

                    // uncomment to update results - here to avoid accidentally accepting worse
                    //fs::write(&result_file, &out).unwrap();
                }
            }

            false
        } else {
            true
        }
    }

    fn parse_stats(stats: &str) -> (Option<(i32, i32)>, i32, i32) {
        let mut lines = stats.lines();

        let first = lines.next().unwrap();
        let maybe_lengths = if first == "No solution" {
            None
        } else {
            let moves = parse_number_from_line(lines.next().unwrap());
            let pushes = parse_number_from_line(lines.next().unwrap());
            Some((moves, pushes))
        };

        // created and visited
        let created = parse_number_from_line(lines.next().unwrap());
        let visited = parse_number_from_line(lines.next().unwrap());

        (maybe_lengths, created, visited)
    }

    fn parse_number_from_line(line: &str) -> i32 {
        line.split_whitespace()
            .last()
            .unwrap()
            .split(',')
            .collect::<Vec<_>>()
            .join("")
            .parse()
            .unwrap()
    }

    // old benches using the default bencher - all ignored since moving to criterion

    #[bench]
    #[ignore]
    fn bench_boxxle1_001(b: &mut Bencher) {
        // 3 goals in a row
        bench_level("levels/boxxle1/1.txt", Method::PushOptimal, b);
    }

    #[bench]
    #[ignore]
    fn bench_boxxle1_005(b: &mut Bencher) {
        // 4 boxes goal room
        bench_level("levels/boxxle1/5.txt", Method::PushOptimal, b);
    }

    #[bench]
    #[ignore]
    fn bench_boxxle1_018(b: &mut Bencher) {
        // 6 boxes - tiny goalroom
        bench_level("levels/boxxle1/18.txt", Method::PushOptimal, b);
    }

    #[bench]
    #[ignore]
    fn bench_boxxle1_108(b: &mut Bencher) {
        // 6 boxes in the middle
        bench_level("levels/boxxle1/108.txt", Method::PushOptimal, b);
    }

    #[bench]
    #[ignore]
    fn bench_boxxle1_001_moves(b: &mut Bencher) {
        bench_level("levels/boxxle1/1.txt", Method::MoveOptimal, b);
    }

    fn bench_level(level_path: &str, method: Method, b: &mut Bencher) {
        let level = level_path.load_level().unwrap();

        b.iter(|| test::black_box(level.solve(test::black_box(method), test::black_box(false))));
    }
}
