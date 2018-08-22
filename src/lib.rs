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
pub mod data;
pub mod formatter;
pub mod level;
pub mod moves;
pub mod solver;

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

    use crate::config::Method;

    use super::*;

    #[test]
    fn test_levels() {
        // TODO add LURD
        let levels = [
            (Method::Pushes, "custom", "01-simplest-custom.txt"),
            (Method::Pushes, "custom", "01-simplest-xsb.txt"),
            (Method::Pushes, "custom", "02-one-way-xsb.txt"),
            (Method::Pushes, "custom", "02-one-way.txt"),
            (Method::Pushes, "custom", "03-long-way.txt"),
            (Method::Pushes, "custom", "04-two-boxes-no-packing.txt"),
            //(Method::Pushes, "custom", "04-two-boxes-remover.txt"), // remover
            (Method::Pushes, "custom", "04-two-boxes.txt"),
            (Method::Pushes, "custom", "no-solution-parking.txt"),
            //(Method::Pushes, "custom", "original-sokoban-01-remover.txt"), // remover
            //(Method::Pushes, "custom", "supaplex.txt"), // remover
            //(Method::Pushes, "custom", "supaplex-goals.txt"), // very slow
            (Method::Pushes, "boxxle1", "1.txt"),
            (Method::Pushes, "boxxle1", "2.txt"),
            (Method::Pushes, "boxxle1", "3.txt"),
            (Method::Pushes, "boxxle1", "4.txt"),
            (Method::Pushes, "boxxle1", "5.txt"),
            (Method::Pushes, "boxxle1", "6.txt"),
            (Method::Pushes, "boxxle1", "7.txt"),
            (Method::Pushes, "boxxle1", "8.txt"),
            (Method::Pushes, "boxxle1", "9.txt"),
            (Method::Pushes, "boxxle1", "10.txt"),
            (Method::Pushes, "boxxle1", "11.txt"),
            //(Method::Pushes, "boxxle1", "12.txt"), // very slow
            (Method::Pushes, "boxxle1", "13.txt"),
            //(Method::Pushes, "boxxle1", "14.txt"), // never solved
            (Method::Pushes, "boxxle1", "15.txt"),
            //(Method::Pushes, "boxxle1", "16.txt"), // never solved
            (Method::Pushes, "boxxle1", "17.txt"),
            (Method::Pushes, "boxxle1", "18.txt"),
            (Method::Pushes, "boxxle1", "19.txt"),
            (Method::Pushes, "boxxle1", "20.txt"),
            (Method::Pushes, "boxxle1", "21.txt"),
            //(Method::Pushes, "boxxle1", "22.txt"), // never solved
            (Method::Pushes, "boxxle1", "23.txt"),
            //(Method::Pushes, "boxxle1", "24.txt"), // never solved
            //(Method::Pushes, "boxxle1", "25.txt"), // slow
            //(Method::Pushes, "boxxle1", "26.txt"), // never solved
            (Method::Pushes, "boxxle1", "27.txt"),
            (Method::Pushes, "boxxle1", "28.txt"),
            //(Method::Pushes, "boxxle1", "29.txt"), // slow
            //(Method::Pushes, "boxxle1", "30.txt"), // never solved
            (Method::Pushes, "boxxle2", "1.txt"),
            (Method::Pushes, "boxxle2", "2.txt"),
            (Method::Pushes, "boxxle2", "3.txt"),
            (Method::Pushes, "boxxle2", "4.txt"),
            //(Method::Pushes, "boxxle2", "5.txt"), // very slow
            (Method::Pushes, "boxxle2", "6.txt"),
            //(Method::Pushes, "boxxle2", "7.txt"), // never solved
            //(Method::Pushes, "boxxle2", "8.txt"), // never solved
            //(Method::Pushes, "boxxle2", "9.txt"), // never solved
            //(Method::Pushes, "boxxle2", "10.txt"), // never solved
            //(Method::Pushes, "original", "1.txt"), // very slow
            (Method::Moves, "custom", "01-simplest-custom.txt"),
            (Method::Moves, "custom", "01-simplest-xsb.txt"),
            (Method::Moves, "custom", "02-one-way-xsb.txt"),
            (Method::Moves, "custom", "02-one-way.txt"),
            (Method::Moves, "custom", "03-long-way.txt"),
            (Method::Moves, "custom", "04-two-boxes-no-packing.txt"),
            //(Method::Moves, "custom", "04-two-boxes-remover.txt"), // remover
            (Method::Moves, "custom", "04-two-boxes.txt"),
            (Method::Moves, "custom", "no-solution-parking.txt"),
            //(Method::Moves, "custom", "original-sokoban-01-remover.txt"), // remover
            //(Method::Moves, "custom", "supaplex.txt"), // remover
            //(Method::Moves, "custom", "supaplex-goals.txt"), // never solved
            (Method::Moves, "boxxle1", "1.txt"),
            (Method::Moves, "boxxle1", "2.txt"),
            (Method::Moves, "boxxle1", "3.txt"),
            (Method::Moves, "boxxle1", "4.txt"),
            (Method::Moves, "boxxle1", "5.txt"),
            //(Method::Moves, "boxxle1", "6.txt"), // slow
            (Method::Moves, "boxxle1", "7.txt"),
            (Method::Moves, "boxxle1", "8.txt"),
            //(Method::Moves, "boxxle1", "9.txt"), // slow
            (Method::Moves, "boxxle1", "10.txt"),
        ];

        // for some reason rayon makes this actually slower
        let succeeded = levels
            .iter()
            .filter(|&&(method, level_pack, level_name)| test_level(method, level_pack, level_name))
            .count();
        assert_eq!(succeeded, levels.len());
    }

    fn test_level(method: Method, level_pack: &str, level_name: &str) -> bool {
        // for updating results more easily
        // (need to update when equal too because the file includes individual depths)
        #![allow(collapsible_if)]

        use std::fmt::Write;
        use std::time::Instant;

        let method_name = method.to_string().to_lowercase();
        let level_path = format!("levels/{}/{}", level_pack, level_name);
        let result_dir = format!("solutions/{}-{}", level_pack, method_name);
        let result_file = format!("{}/{}", result_dir, level_name);

        println!("Solving {} using {}", level_path, method_name);
        let started = Instant::now();

        let solution = level_path
            .load_level()
            .unwrap()
            .solve(method, false)
            .unwrap();

        // innacurate, only useful to quickly see which levels are difficult
        println!(
            "Solved {} using {} in approximately {} ms",
            level_path,
            method_name,
            (started.elapsed().as_millis() as u64).separated_string(), // separator doesn't support u128
        );

        let mut out = String::new();
        write!(out, "{:?}", solution).unwrap();

        if !Path::new(&result_file).exists() {
            if !Path::new(&result_dir).exists() {
                fs::create_dir(&result_dir).unwrap();
            }
            fs::write(&result_file, &out).unwrap();
            print!("Solution:\n{}", out);
            println!("         >>> SAVED NEW SOLUTION <<<");
        }

        let expected = fs::read_to_string(&result_file).unwrap();
        if out != expected {
            print!("Expected:\n{}", expected);
            print!("Got:\n{}", out);

            // other stats can go up with a better solution
            let (maybe_out_len, out_created, out_visited) = parse_stats(&out);
            let (maybe_expected_len, expected_created, expected_visited) = parse_stats(&expected);
            if maybe_out_len.is_some() != maybe_expected_len.is_some() {
                println!("         >>> SOLVABILITY CHANGED <<<\n\n");
            } else {
                let out_len = maybe_out_len.unwrap_or(-1);
                let expected_len = maybe_expected_len.unwrap_or(-1);
                if out_len > expected_len
                    || out_created > expected_created
                    || out_visited > expected_visited
                {
                    println!("         >>> WORSE <<<\n\n");
                } else {
                    if out_len == expected_len
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

    fn parse_stats(stats: &str) -> (Option<i32>, i32, i32) {
        let mut lines = stats.lines();

        // no solution or length
        let maybe_length = lines
            .next()
            .unwrap()
            .split_whitespace()
            .last()
            .unwrap()
            .split(',')
            .collect::<Vec<_>>()
            .join("")
            .parse()
            .ok();

        // created and visited
        let created = lines
            .next()
            .unwrap()
            .split_whitespace()
            .last()
            .unwrap()
            .split(',')
            .collect::<Vec<_>>()
            .join("")
            .parse()
            .unwrap();
        let visited = lines
            .next()
            .unwrap()
            .split_whitespace()
            .last()
            .unwrap()
            .split(',')
            .collect::<Vec<_>>()
            .join("")
            .parse()
            .unwrap();

        (maybe_length, created, visited)
    }

    // old benches using the default bencher - all ignored since moving to criterion

    #[bench]
    #[ignore]
    fn bench_boxxle1_001(b: &mut Bencher) {
        // 3 goals in a row
        bench_level("levels/boxxle1/1.txt", Method::Pushes, b);
    }

    #[bench]
    #[ignore]
    fn bench_boxxle1_005(b: &mut Bencher) {
        // 4 boxes goal room
        bench_level("levels/boxxle1/5.txt", Method::Pushes, b);
    }

    #[bench]
    #[ignore]
    fn bench_boxxle1_018(b: &mut Bencher) {
        // 6 boxes - tiny goalroom
        bench_level("levels/boxxle1/18.txt", Method::Pushes, b);
    }

    #[bench]
    #[ignore]
    fn bench_boxxle1_108(b: &mut Bencher) {
        // 6 boxes in the middle
        bench_level("levels/boxxle1/108.txt", Method::Pushes, b);
    }

    #[bench]
    #[ignore]
    fn bench_boxxle1_001_moves(b: &mut Bencher) {
        bench_level("levels/boxxle1/1.txt", Method::Moves, b);
    }

    fn bench_level(level_path: &str, method: Method, b: &mut Bencher) {
        let level = level_path.load_level().unwrap();

        b.iter(|| test::black_box(level.solve(test::black_box(method), test::black_box(false))));
    }
}
