// https://github.com/rust-lang/rust/issues/31844
#![feature(specialization)]
// Opt in to unstable features expected for Rust 2018
#![feature(rust_2018_preview)]
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

#[cfg(test)]
extern crate test;

pub mod config;
pub mod data;
pub mod formatter;
pub mod level;
pub mod map;
pub mod solver;
pub mod state;

mod fs;
mod parser;
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
    use test::Bencher;

    use crate::config::Method;

    use super::*;

    #[test]
    fn test_levels() {
        let levels = [
            (Method::Pushes, "custom", "01-simplest-xsb.txt"),
            (Method::Pushes, "custom", "01-simplest-custom.txt"),
            (Method::Pushes, "custom", "02-one-way.txt"),
            (Method::Pushes, "custom", "03-long-way.txt"),
            (Method::Pushes, "custom", "04-two-boxes-no-packing.txt"),
            //(Method::Pushes, "custom", "04-two-boxes-remover.txt"),
            (Method::Pushes, "custom", "04-two-boxes.txt"),
            (Method::Pushes, "custom", "no-solution-parking.txt"),
            //(Method::Pushes, "custom", "original-sokoban-01-remover.txt"),
            //(Method::Pushes, "custom", "supaplex.txt"),
            //(Method::Pushes, "custom", "supaplex-goals.txt"),
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
            //(Method::Pushes, "boxxle1", "12.txt"),
            (Method::Pushes, "boxxle1", "13.txt"),
            //(Method::Pushes, "boxxle1", "14.txt"),
            (Method::Pushes, "boxxle1", "15.txt"),
            //(Method::Pushes, "boxxle1", "16.txt"),
            (Method::Pushes, "boxxle1", "17.txt"),
            (Method::Pushes, "boxxle1", "18.txt"),
            (Method::Pushes, "boxxle1", "19.txt"),
            (Method::Pushes, "boxxle1", "20.txt"),
            (Method::Moves, "custom", "01-simplest-xsb.txt"),
            (Method::Moves, "custom", "01-simplest-custom.txt"),
            (Method::Moves, "custom", "02-one-way.txt"),
            (Method::Moves, "custom", "03-long-way.txt"),
            (Method::Moves, "custom", "04-two-boxes-no-packing.txt"),
            //(Method::Moves, "custom", "04-two-boxes-remover.txt"),
            (Method::Moves, "custom", "04-two-boxes.txt"),
            (Method::Moves, "custom", "no-solution-parking.txt"),
            //(Method::Moves, "custom", "original-sokoban-01-remover.txt"),
            //(Method::Moves, "custom", "supaplex.txt"),
            //(Method::Moves, "custom", "supaplex-goals.txt"),
            (Method::Moves, "boxxle1", "1.txt"),
            (Method::Moves, "boxxle1", "2.txt"),
            (Method::Moves, "boxxle1", "3.txt"),
            (Method::Moves, "boxxle1", "4.txt"),
            (Method::Moves, "boxxle1", "5.txt"),
            //(Method::Moves, "boxxle1", "6.txt"),
            (Method::Moves, "boxxle1", "7.txt"),
            (Method::Moves, "boxxle1", "8.txt"),
            //(Method::Moves, "boxxle1", "9.txt"),
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
        let result_file = format!("solutions/{}-{}/{}", level_pack, method_name, level_name);

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
            started.elapsed().as_millis(),
        );

        let mut out = String::new();
        write!(out, "{:?}", solution).unwrap();

        // uncomment to add new files, directory needs to exist, don't update this way - see below
        //fs::write_file(&result_file, &out).unwrap();

        let expected = fs::read_file(&result_file).unwrap();
        if out != expected {
            print!("Expected:\n{}", expected);
            print!("Got:\n{}", out);

            // other stats can go up with a better solution
            let (out_len, out_created, out_visited) = parse_stats(&out);
            let (expected_len, expected_created, expected_visited) = parse_stats(&expected);
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
                //fs::write_file(&result_file, &out).unwrap();
            }

            false
        } else {
            true
        }
    }

    fn parse_stats(stats: &str) -> (i32, i32, i32) {
        let mut lines = stats.lines();

        // no solution or length
        let length = lines
            .next()
            .unwrap()
            .split_whitespace()
            .last()
            .unwrap()
            .split(',')
            .collect::<Vec<_>>()
            .join("")
            .parse()
            .unwrap_or(0);

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

        (length, created, visited)
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
