// https://github.com/rust-lang/rust/issues/45388
#![feature(crate_visibility_modifier)]
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
#![warn(clippy::all)]
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
    #[allow(unused)] // needed for latest nightly
    extern crate test;
    use self::test::Bencher;

    use std::fmt::{Display, Write};
    use std::fs;
    use std::path::Path;
    use std::time::Instant;

    use difference::Changeset;
    use separator::Separatable;

    use crate::config::Method::{self, Moves, Pushes};

    use super::*;

    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    enum TestResult {
        Ok,
        Changed(Change, Change, Change),
        SolvabilityChanged,
    }

    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    enum Change {
        Worse,
        Equal,
        Better,
    }

    #[test]
    fn test_levels() {
        // Note: this test (and the other level tests) will likely break if implementation details of the containers used in the solver change.

        const UNSOLVED: i32 = 5;
        const VERY_SLOW: i32 = 4;
        const SLOW: i32 = 3;
        const VERY_SLOW_IN_DEBUG: i32 = 2;
        const SLOW_IN_DEBUG: i32 = 1;
        const OK: i32 = 0;

        #[cfg(debug_assertions)]
        const MAX_DIFFICULTY: i32 = 0;

        #[cfg(not(debug_assertions))]
        const MAX_DIFFICULTY: i32 = 2; // set to 4 to update all levels

        #[cfg_attr(rustfmt, rustfmt_skip)]
        let levels = [
            (Pushes, "custom", "00-empty.txt", OK),
            (Pushes, "custom", "00-solved.txt", OK),
            (Pushes, "custom", "01-simplest-custom.txt", OK),
            (Pushes, "custom", "01-simplest-xsb.txt", OK),
            (Pushes, "custom", "02-one-way-xsb.txt", OK),
            (Pushes, "custom", "02-one-way.txt", OK),
            (Pushes, "custom", "03-long-way.txt", OK),
            (Pushes, "custom", "03-two-ways.txt", OK),
            (Pushes, "custom", "04-two-boxes-no-packing.txt", OK),
            (Pushes, "custom", "04-two-boxes.txt", OK),
            (Pushes, "custom", "05-same-moves-diff-pushes.txt", OK),
            (Pushes, "custom", "05-same-pushes-diff-moves.txt", OK),
            (Pushes, "custom", "deadlock-cell-on-dead-end.txt", OK),
            (Pushes, "custom", "deadlock-original-28.txt", UNSOLVED),
            (Pushes, "custom", "no-solution-parking.txt", OK),
            (Pushes, "custom", "remover-00-solved.txt", OK),
            (Pushes, "custom", "remover-01-simplest-custom.txt", OK),
            (Pushes, "custom", "remover-01-simplest-xsb.txt", OK),
            (Pushes, "custom", "remover-02-one-way-xsb.txt", OK),
            (Pushes, "custom", "remover-02-one-way.txt", OK),
            (Pushes, "custom", "remover-03-long-way.txt", OK),
            (Pushes, "custom", "remover-04-two-boxes.txt", OK),
            (Pushes, "custom", "remover-original-01.txt", OK),
            (Pushes, "custom", "remover-original-02.txt", SLOW),
            (Pushes, "custom", "remover-original-03.txt", SLOW),
            (Pushes, "custom", "remover-original-04.txt", UNSOLVED),
            (Pushes, "custom", "supaplex-remover.txt", VERY_SLOW_IN_DEBUG),
            (Pushes, "custom", "supaplex-goals.txt", VERY_SLOW),
            (Pushes, "696", "1.txt", OK),
            (Pushes, "696", "2.txt", OK),
            (Pushes, "696", "3.txt", OK),
            (Pushes, "696", "4.txt", OK),
            (Pushes, "696", "5.txt", OK),
            (Pushes, "696", "6.txt", OK),
            (Pushes, "696", "7.txt", OK),
            (Pushes, "696", "8.txt", OK),
            (Pushes, "696", "9.txt", OK),
            (Pushes, "696", "10.txt", OK),
            (Pushes, "696", "11.txt", OK),
            (Pushes, "696", "12.txt", OK),
            (Pushes, "696", "13.txt", OK),
            (Pushes, "696", "14.txt", OK),
            (Pushes, "696", "15.txt", OK),
            (Pushes, "696", "16.txt", OK),
            (Pushes, "696", "17.txt", OK),
            (Pushes, "696", "18.txt", OK),
            (Pushes, "696", "19.txt", OK),
            (Pushes, "696", "20.txt", OK),
            (Pushes, "696", "21.txt", OK),
            (Pushes, "696", "22.txt", OK),
            (Pushes, "696", "23.txt", OK),
            (Pushes, "696", "24.txt", OK),
            (Pushes, "696", "25.txt", OK),
            (Pushes, "696", "26.txt", OK),
            (Pushes, "696", "27.txt", OK),
            (Pushes, "696", "28.txt", OK),
            (Pushes, "696", "29.txt", OK),
            (Pushes, "696", "30.txt", OK),
            (Pushes, "696", "31.txt", OK),
            (Pushes, "696", "32.txt", OK),
            (Pushes, "696", "33.txt", OK),
            (Pushes, "696", "34.txt", OK),
            (Pushes, "696", "35.txt", OK),
            (Pushes, "696", "36.txt", OK),
            (Pushes, "696", "37.txt", OK),
            (Pushes, "696", "38.txt", OK),
            (Pushes, "696", "39.txt", OK),
            (Pushes, "696", "40.txt", OK),
            (Pushes, "696", "41.txt", OK),
            (Pushes, "696", "42.txt", OK),
            (Pushes, "696", "43.txt", OK),
            (Pushes, "696", "44.txt", OK),
            (Pushes, "696", "45.txt", OK),
            (Pushes, "696", "46.txt", OK),
            (Pushes, "696", "47.txt", OK),
            (Pushes, "696", "48.txt", OK),
            (Pushes, "696", "49.txt", OK),
            (Pushes, "696", "50.txt", SLOW_IN_DEBUG),
            (Pushes, "696", "51.txt", SLOW_IN_DEBUG),
            (Pushes, "696", "52.txt", SLOW_IN_DEBUG),
            (Pushes, "696", "53.txt", SLOW_IN_DEBUG),
            (Pushes, "696", "54.txt", SLOW_IN_DEBUG),
            (Pushes, "696", "55.txt", SLOW_IN_DEBUG),
            (Pushes, "696", "56.txt", SLOW_IN_DEBUG),
            (Pushes, "696", "57.txt", SLOW_IN_DEBUG),
            (Pushes, "696", "58.txt", SLOW_IN_DEBUG),
            (Pushes, "696", "59.txt", SLOW_IN_DEBUG),
            (Pushes, "696", "60.txt", SLOW_IN_DEBUG),
            (Pushes, "696", "61.txt", SLOW_IN_DEBUG),
            (Pushes, "696", "62.txt", SLOW_IN_DEBUG),
            (Pushes, "696", "63.txt", SLOW_IN_DEBUG),
            (Pushes, "696", "64.txt", SLOW_IN_DEBUG),
            (Pushes, "696", "65.txt", SLOW_IN_DEBUG),
            (Pushes, "696", "66.txt", SLOW_IN_DEBUG),
            (Pushes, "696", "67.txt", SLOW_IN_DEBUG),
            (Pushes, "696", "68.txt", SLOW_IN_DEBUG),
            (Pushes, "696", "69.txt", SLOW_IN_DEBUG),
            (Pushes, "696", "70.txt", SLOW_IN_DEBUG),
            (Pushes, "696", "71.txt", SLOW_IN_DEBUG),
            (Pushes, "696", "72.txt", SLOW_IN_DEBUG),
            (Pushes, "696", "73.txt", SLOW_IN_DEBUG),
            (Pushes, "696", "74.txt", SLOW_IN_DEBUG),
            (Pushes, "696", "75.txt", SLOW_IN_DEBUG),
            (Pushes, "696", "76.txt", SLOW_IN_DEBUG),
            (Pushes, "696", "77.txt", SLOW_IN_DEBUG),
            (Pushes, "696", "78.txt", SLOW_IN_DEBUG),
            (Pushes, "696", "79.txt", SLOW_IN_DEBUG),
            (Pushes, "696", "80.txt", SLOW_IN_DEBUG),
            (Pushes, "696", "81.txt", SLOW_IN_DEBUG),
            (Pushes, "696", "82.txt", SLOW_IN_DEBUG),
            (Pushes, "696", "83.txt", SLOW_IN_DEBUG),
            (Pushes, "696", "84.txt", SLOW_IN_DEBUG),
            (Pushes, "696", "85.txt", SLOW_IN_DEBUG),
            (Pushes, "696", "86.txt", SLOW_IN_DEBUG),
            (Pushes, "696", "87.txt", SLOW_IN_DEBUG),
            (Pushes, "696", "88.txt", SLOW_IN_DEBUG),
            (Pushes, "696", "89.txt", SLOW_IN_DEBUG),
            (Pushes, "696", "90.txt", SLOW_IN_DEBUG),
            (Pushes, "696", "91.txt", SLOW_IN_DEBUG),
            (Pushes, "696", "92.txt", SLOW_IN_DEBUG),
            (Pushes, "696", "93.txt", SLOW_IN_DEBUG),
            (Pushes, "696", "94.txt", SLOW_IN_DEBUG),
            (Pushes, "696", "95.txt", SLOW_IN_DEBUG),
            (Pushes, "696", "96.txt", SLOW_IN_DEBUG),
            (Pushes, "696", "97.txt", SLOW_IN_DEBUG),
            (Pushes, "696", "98.txt", SLOW_IN_DEBUG),
            (Pushes, "696", "99.txt", SLOW_IN_DEBUG),
            (Pushes, "boxxle1", "1.txt", OK),
            (Pushes, "boxxle1", "2.txt", OK),
            (Pushes, "boxxle1", "3.txt", OK),
            (Pushes, "boxxle1", "4.txt", OK),
            (Pushes, "boxxle1", "5.txt", OK),
            (Pushes, "boxxle1", "6.txt", VERY_SLOW_IN_DEBUG),
            (Pushes, "boxxle1", "7.txt", OK),
            (Pushes, "boxxle1", "8.txt", OK),
            (Pushes, "boxxle1", "9.txt", SLOW_IN_DEBUG),
            (Pushes, "boxxle1", "10.txt", OK),
            (Pushes, "boxxle1", "11.txt", OK),
            (Pushes, "boxxle1", "12.txt", VERY_SLOW),
            (Pushes, "boxxle1", "13.txt", OK),
            (Pushes, "boxxle1", "14.txt", UNSOLVED),
            (Pushes, "boxxle1", "15.txt", OK),
            (Pushes, "boxxle1", "16.txt", UNSOLVED),
            (Pushes, "boxxle1", "17.txt", VERY_SLOW_IN_DEBUG),
            (Pushes, "boxxle1", "18.txt", SLOW_IN_DEBUG),
            (Pushes, "boxxle1", "19.txt", OK),
            (Pushes, "boxxle1", "20.txt", OK),
            (Pushes, "boxxle1", "21.txt", OK),
            (Pushes, "boxxle1", "22.txt", UNSOLVED),
            (Pushes, "boxxle1", "23.txt", SLOW_IN_DEBUG),
            (Pushes, "boxxle1", "24.txt", UNSOLVED),
            (Pushes, "boxxle1", "25.txt", SLOW),
            (Pushes, "boxxle1", "26.txt", UNSOLVED),
            (Pushes, "boxxle1", "27.txt", OK),
            (Pushes, "boxxle1", "28.txt", SLOW_IN_DEBUG),
            (Pushes, "boxxle1", "29.txt", SLOW),
            (Pushes, "boxxle1", "30.txt", UNSOLVED),
            (Pushes, "boxxle1", "108.txt", OK),
            (Pushes, "boxxle2", "1.txt", OK),
            (Pushes, "boxxle2", "2.txt", OK),
            (Pushes, "boxxle2", "3.txt", OK),
            (Pushes, "boxxle2", "4.txt", OK),
            (Pushes, "boxxle2", "5.txt", UNSOLVED),
            (Pushes, "boxxle2", "6.txt", VERY_SLOW_IN_DEBUG),
            (Pushes, "boxxle2", "7.txt", SLOW),
            (Pushes, "boxxle2", "8.txt", UNSOLVED),
            (Pushes, "boxxle2", "9.txt", UNSOLVED),
            (Pushes, "boxxle2", "10.txt", UNSOLVED),
            (Pushes, "original-and-extra", "1.txt", VERY_SLOW),
            (Moves, "custom", "00-empty.txt", OK),
            (Moves, "custom", "00-solved.txt", OK),
            (Moves, "custom", "01-simplest-custom.txt", OK),
            (Moves, "custom", "01-simplest-xsb.txt", OK),
            (Moves, "custom", "02-one-way-xsb.txt", OK),
            (Moves, "custom", "02-one-way.txt", OK),
            (Moves, "custom", "03-long-way.txt", OK),
            (Moves, "custom", "03-two-ways.txt", OK),
            (Moves, "custom", "04-two-boxes-no-packing.txt", OK),
            (Moves, "custom", "04-two-boxes.txt", OK),
            (Moves, "custom", "05-same-moves-diff-pushes.txt", OK),
            (Moves, "custom", "05-same-pushes-diff-moves.txt", OK),
            (Moves, "custom", "deadlock-cell-on-dead-end.txt", OK),
            (Moves, "custom", "deadlock-original-28.txt", UNSOLVED),
            (Moves, "custom", "no-solution-parking.txt", OK),
            (Moves, "custom", "remover-00-solved.txt", OK),
            (Moves, "custom", "remover-01-simplest-custom.txt", OK),
            (Moves, "custom", "remover-01-simplest-xsb.txt", OK),
            (Moves, "custom", "remover-02-one-way-xsb.txt", OK),
            (Moves, "custom", "remover-02-one-way.txt", OK),
            (Moves, "custom", "remover-03-long-way.txt", OK),
            (Moves, "custom", "remover-04-two-boxes.txt", OK),
            (Moves, "custom", "remover-original-01.txt", VERY_SLOW),
            (Moves, "custom", "remover-original-02.txt", UNSOLVED),
            (Moves, "custom", "remover-original-03.txt", UNSOLVED),
            (Moves, "custom", "remover-original-04.txt", UNSOLVED),
            (Moves, "custom", "supaplex-remover.txt", VERY_SLOW),
            (Moves, "custom", "supaplex-goals.txt", VERY_SLOW),
            (Moves, "boxxle1", "1.txt", OK),
            (Moves, "boxxle1", "2.txt", SLOW_IN_DEBUG),
            (Moves, "boxxle1", "3.txt", OK),
            (Moves, "boxxle1", "4.txt", OK),
            (Moves, "boxxle1", "5.txt", OK),
            (Moves, "boxxle1", "6.txt", SLOW),
            (Moves, "boxxle1", "7.txt", SLOW_IN_DEBUG),
            (Moves, "boxxle1", "8.txt", OK),
            // TODO jsoko says it's solvable in 170 moves and 41 pushes (not 43)
            // jsoko: ldldlluurDldRurrurrdLLLDlluullldRddDrdrRRdrruUUUddddlluRlllluluuuRurDurDlDRurrurrdLLLrrdddlddrUUUUdddllllluluuuurrrDrrurrdLddddllllldlUUUUdddrrrrruruuuLLLDuulDullldRRRurD
            // this: ldldlluurDDldRuurrurrdLLLLulDullldRddDrdrRRdrruUUUddddlluRlllluluuuRRurDlDRurrurrdLLLrrdddlddrUUUUddldlllluluuururrDrrurrdLdddldlllldlUUUUddrdrrrruruuuLLLDuulDlulldRRRurD
            // supaplex-remover can also be solved move optimally with fewer pushes
            (Moves, "boxxle1", "9.txt", SLOW),
            (Moves, "boxxle1", "10.txt", OK),
        ];

        let levels: Vec<_> = levels
            .iter()
            .filter(|&&(_, _, _, difficulty)| difficulty <= MAX_DIFFICULTY)
            .map(|&(method, pack, level, _)| (method, pack, level))
            .collect();
        test_and_time_levels(&levels);
    }

    #[test]
    #[ignore] // most are simple but there's so many of them that testing all of them takes too long
    fn test_696() {
        let levels: Vec<_> = (100..=696)
            .filter(|&i| i != 250 && i != 693) // currently can't solve these two
            .map(|num| (Method::Pushes, "696", format!("{}.txt", num)))
            .collect();
        test_and_time_levels(&levels);
    }

    #[test]
    #[ignore]
    fn test_aymeric() {
        let levels: Vec<_> = (1..=20)
            .map(|num| (Method::Pushes, "aymeric-cosmonotes", format!("{}.txt", num)))
            .chain((1..=40).map(|num| {
                (
                    Method::Pushes,
                    "aymeric-microcosmos",
                    format!("{}.txt", num),
                )
            }))
            .chain(
                (1..=40).map(|num| (Method::Pushes, "aymeric-minicosmos", format!("{}.txt", num))),
            )
            .chain(
                (1..=40).map(|num| (Method::Pushes, "aymeric-nabocosmos", format!("{}.txt", num))),
            )
            .chain(
                (1..=20).map(|num| (Method::Pushes, "aymeric-picocosmos", format!("{}.txt", num))),
            )
            .collect();
        test_and_time_levels(&levels);
    }

    #[test]
    #[ignore]
    fn test_microban1() {
        let levels: Vec<_> = (1..=155)
            .filter(|&num| num != 93 && num != 144 && num != 153)
            .map(|num| (Method::Pushes, "microban1", format!("{}.txt", num)))
            .collect();
        test_and_time_levels(&levels);
    }

    #[test]
    #[ignore]
    fn test_microban2() {
        let levels: Vec<_> = (1..=135)
            .filter(|&num| num != 66 && num != 102 && num != 104 && num < 100)
            .map(|num| (Method::Pushes, "microban2", format!("{}.txt", num)))
            .collect();
        test_and_time_levels(&levels);
    }

    fn test_and_time_levels<L: AsRef<str> + Display>(levels: &[(Method, &str, L)]) {
        let started = Instant::now();

        let results: Vec<_> = levels
            .iter()
            .map(|(method, pack, name)| (method, pack, name, test_level(*method, pack, name)))
            .collect();

        println!(
            "Tested {} levels in {} ms",
            levels.len(),
            (started.elapsed().as_millis() as u64).separated_string() // separator doesn't support u128
        );

        let succeeded = results
            .iter()
            .filter(|&(_, _, _, res)| *res == TestResult::Ok)
            .count();

        let print_bad = |msg, predicate: fn(TestResult) -> bool| {
            let bad_levels: Vec<_> = results
                .iter()
                .filter(|&(_, _, _, res)| predicate(*res))
                .collect();
            if !bad_levels.is_empty() {
                println!("{} ({}):", msg, bad_levels.len());
                for (method, pack, name, _) in bad_levels {
                    println!("\t{} {}/{}", method, pack, name);
                }
            }
        };

        macro_rules! level_list {
            ($msg:expr, $moves:pat, $pushes:pat, $stats:pat) => {
                print_bad($msg, |res| {
                    if let TestResult::Changed($moves, $pushes, $stats) = res {
                        true
                    } else {
                        false
                    }
                });
            };
        }

        level_list!("Better moves", Change::Better, _, _);
        level_list!("Equal moves", Change::Equal, _, _);
        level_list!("Worse moves", Change::Worse, _, _);
        level_list!("Better pushes", _, Change::Better, _);
        level_list!("Equal pushes", _, Change::Equal, _);
        level_list!("Worse pushes", _, Change::Worse, _);
        level_list!("Better stats", _, _, Change::Better);
        level_list!("Equal stats", _, _, Change::Equal);
        level_list!("Worse stats", _, _, Change::Worse);

        print_bad("Solvability changed", |res| {
            res == TestResult::SolvabilityChanged
        });

        assert_eq!(succeeded, levels.len());
    }

    #[must_use]
    fn test_level<L: AsRef<str> + Display>(
        method: Method,
        level_pack: &str,
        level_name: L,
    ) -> TestResult {
        // for updating results more easily
        // (need to update when equal too because the file includes individual depths)
        #![allow(clippy::collapsible_if)]

        let method_name = method.to_string();
        let level_path = format!("levels/{}/{}", level_pack, level_name);
        let result_dir = format!("solutions/{}/{}", method_name, level_pack);
        let result_file = format!("{}/{}", result_dir, level_name);

        println!("Solving: method {}, level {}", method_name, level_path);
        let started = Instant::now();

        let level = level_path.load_level().unwrap();
        let solution = level.solve(method, false).unwrap();

        // innacurate, only useful to quickly see which levels are difficult
        println!(
            "Finished in approximately {} ms",
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
            let include_steps = method == Method::Moves;
            write!(out, "{}", level.xsb_solution(moves, include_steps)).unwrap();
        }

        if !Path::new(&result_dir).exists() {
            fs::create_dir_all(&result_dir).unwrap();
        }

        if !Path::new(&result_file).exists() {
            fs::write(&result_file, &out).unwrap();
            print!("Solution:\n{}", out);
            println!("\t>>> SAVED NEW SOLUTION <<<\n\n");
            // we could return here but let's read the file back out as a sanity check
        }

        let expected = fs::read_to_string(&result_file).unwrap();
        if out != expected {
            //print!("\t>>> Expected:\n{}", expected);
            //print!("\t>>> Got:\n{}", out);
            print!("{}", Changeset::new(&expected, &out, "\n"));

            // other stats can go up with a better solution
            let (maybe_out_lens, out_created, out_visited) = parse_stats(&out);
            let (maybe_expected_lens, expected_created, expected_visited) = parse_stats(&expected);
            if maybe_out_lens.is_some() != maybe_expected_lens.is_some() {
                println!("\t>>> SOLVABILITY CHANGED <<<\n\n");
                TestResult::SolvabilityChanged
            } else {
                let (out_moves, out_pushes) = maybe_out_lens.unwrap_or((-1, -1));
                let (expected_moves, expected_pushes) = maybe_expected_lens.unwrap_or((-1, -1));

                let moves_change = if out_moves > expected_moves {
                    println!("\t>>> WORSE MOVES <<<");
                    Change::Worse
                } else if out_moves == expected_moves {
                    println!("\t>>> EQUAL MOVES <<<");
                    Change::Equal
                } else {
                    println!("\t>>> BETTER MOVES <<<");
                    Change::Better
                };

                let pushes_change = if out_pushes > expected_pushes {
                    println!("\t>>> WORSE PUSHES <<<");
                    Change::Worse
                } else if out_pushes == expected_pushes {
                    println!("\t>>> EQUAL PUSHES <<<");
                    Change::Equal
                } else {
                    println!("\t>>> BETTER PUSHES <<<");
                    Change::Better
                };

                let stats_change =
                    if out_created > expected_created || out_visited > expected_visited {
                        println!("\t>>> WORSE STATS <<<");
                        Change::Worse
                    } else if out_created == expected_created && out_visited == expected_visited {
                        println!("\t>>> EQUAL STATS <<<");
                        Change::Equal
                    } else {
                        println!("\t>>> BETTER STATS <<<");
                        Change::Better
                    };

                println!();
                println!();

                // uncomment to update results (might also wanna run with higher difficulty to update all levels)
                //fs::write(&result_file, &out).unwrap();

                TestResult::Changed(moves_change, pushes_change, stats_change)
            }
        } else {
            TestResult::Ok
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
