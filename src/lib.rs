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

    use crate::config::Method;

    use super::*;

    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    struct TestResult {
        /// move_cnt, push_cnt
        counts: Option<(i32, i32)>,
        comparison: TestComparison,
    }

    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    enum TestComparison {
        Ok,
        /// moves, pushes, stats
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

        const UNSOLVED: i32 = 3;
        const SLOW: i32 = 2; // slow even in release
        const RELEASE: i32 = 1; // only in release, too slow in debug
        const OK: i32 = 0;

        // yes, debug actually tests fewer levels because it's really slow
        #[cfg(debug_assertions)]
        const MAX_DIFFICULTY: i32 = OK;

        #[cfg(not(debug_assertions))]
        const MAX_DIFFICULTY: i32 = RELEASE; // set to SLOW to update all levels

        const ALL_UNSOLVED: [i32; 4] = [UNSOLVED, UNSOLVED, UNSOLVED, UNSOLVED];
        const ALL_SLOW: [i32; 4] = [SLOW, SLOW, SLOW, SLOW];
        const ALL_RELEASE: [i32; 4] = [RELEASE, RELEASE, RELEASE, RELEASE];
        const ALL_OK: [i32; 4] = [OK, OK, OK, OK];

        // elastic tabstops would make this readable but humanity has yet to achieve
        // that level of sophistication in an editor that is also able to automatically save when it loses focus :(
        #[cfg_attr(rustfmt, rustfmt_skip)]
        let levels = [
            ("custom", "00-empty.txt", ALL_OK),
            ("custom", "00-solved.txt", ALL_OK),
            ("custom", "01-simplest-custom.txt", ALL_OK),
            ("custom", "01-simplest-xsb.txt", ALL_OK),
            ("custom", "02-one-way-xsb.txt", ALL_OK),
            ("custom", "02-one-way.txt", ALL_OK),
            ("custom", "03-long-way.txt", ALL_OK),
            ("custom", "03-two-ways.txt", ALL_OK),
            ("custom", "04-two-boxes-no-packing.txt", ALL_OK),
            ("custom", "04-two-boxes.txt", ALL_OK),
            ("custom", "05-same-moves-diff-pushes.txt", ALL_OK),
            ("custom", "05-same-pushes-diff-moves.txt", ALL_OK),
            ("custom", "deadlock-cell-on-dead-end.txt", ALL_OK),
            ("custom", "deadlock-original-28.txt", ALL_UNSOLVED),
            ("custom", "no-solution-parking.txt", ALL_OK),
            ("custom", "remover-00-solved.txt", ALL_OK),
            ("custom", "remover-01-simplest-custom.txt", ALL_OK),
            ("custom", "remover-01-simplest-xsb.txt", ALL_OK),
            ("custom", "remover-02-one-way-xsb.txt", ALL_OK),
            ("custom", "remover-02-one-way.txt", ALL_OK),
            ("custom", "remover-03-long-way.txt", ALL_OK),
            ("custom", "remover-04-two-boxes.txt", ALL_OK),
            ("custom", "remover-original-01.txt", [SLOW, SLOW, SLOW, OK]),
            ("custom", "remover-original-02.txt", [UNSOLVED, UNSOLVED, SLOW, SLOW]),
            ("custom", "remover-original-03.txt", [UNSOLVED, UNSOLVED, SLOW, SLOW]),
            ("custom", "remover-original-04.txt", ALL_UNSOLVED),
            ("custom", "supaplex-remover.txt", [SLOW, SLOW, SLOW, RELEASE]),
            ("custom", "supaplex-goals.txt", ALL_SLOW),
            ("696", "1.txt", ALL_OK),
            ("696", "2.txt", ALL_OK),
            ("696", "3.txt", ALL_OK),
            ("696", "4.txt", ALL_OK),
            ("696", "5.txt", ALL_OK),
            ("696", "6.txt", ALL_OK),
            ("696", "7.txt", ALL_OK),
            ("696", "8.txt", ALL_OK),
            ("696", "9.txt", ALL_OK),
            ("696", "10.txt", ALL_OK),
            ("696", "11.txt", ALL_RELEASE),
            ("696", "12.txt", ALL_RELEASE),
            ("696", "13.txt", ALL_RELEASE),
            ("696", "14.txt", ALL_RELEASE),
            ("696", "15.txt", ALL_RELEASE),
            ("696", "16.txt", ALL_RELEASE),
            ("696", "17.txt", ALL_RELEASE),
            ("696", "18.txt", ALL_RELEASE),
            ("696", "19.txt", ALL_RELEASE),
            ("696", "20.txt", ALL_RELEASE),
            ("696", "21.txt", ALL_RELEASE),
            ("696", "22.txt", ALL_RELEASE),
            ("696", "23.txt", ALL_RELEASE),
            ("696", "24.txt", ALL_RELEASE),
            ("696", "25.txt", ALL_RELEASE),
            ("696", "26.txt", ALL_RELEASE),
            ("696", "27.txt", ALL_RELEASE),
            ("696", "28.txt", ALL_RELEASE),
            ("696", "29.txt", ALL_RELEASE),
            ("696", "30.txt", ALL_RELEASE),
            ("696", "31.txt", ALL_RELEASE),
            ("696", "32.txt", ALL_RELEASE),
            ("696", "33.txt", ALL_RELEASE),
            ("696", "34.txt", ALL_RELEASE),
            ("696", "35.txt", ALL_RELEASE),
            ("696", "36.txt", ALL_RELEASE),
            ("696", "37.txt", ALL_RELEASE),
            ("696", "38.txt", ALL_RELEASE),
            ("696", "39.txt", ALL_RELEASE),
            ("696", "40.txt", ALL_RELEASE),
            ("696", "41.txt", ALL_RELEASE),
            ("696", "42.txt", ALL_RELEASE),
            ("696", "43.txt", ALL_RELEASE),
            ("696", "44.txt", ALL_RELEASE),
            ("696", "45.txt", ALL_RELEASE),
            ("696", "46.txt", ALL_RELEASE),
            ("696", "47.txt", ALL_RELEASE),
            ("696", "48.txt", ALL_RELEASE),
            ("696", "49.txt", ALL_RELEASE),
            ("696", "50.txt", ALL_RELEASE),
            ("696", "51.txt", ALL_RELEASE),
            ("696", "52.txt", ALL_RELEASE),
            ("696", "53.txt", ALL_RELEASE),
            ("696", "54.txt", ALL_RELEASE),
            ("696", "55.txt", ALL_RELEASE),
            ("696", "56.txt", ALL_RELEASE),
            ("696", "57.txt", ALL_RELEASE),
            ("696", "58.txt", ALL_RELEASE),
            ("696", "59.txt", ALL_RELEASE),
            ("696", "60.txt", ALL_RELEASE),
            ("696", "61.txt", ALL_RELEASE),
            ("696", "62.txt", ALL_RELEASE),
            ("696", "63.txt", ALL_RELEASE),
            ("696", "64.txt", ALL_RELEASE),
            ("696", "65.txt", ALL_RELEASE),
            ("696", "66.txt", ALL_RELEASE),
            ("696", "67.txt", ALL_RELEASE),
            ("696", "68.txt", ALL_RELEASE),
            ("696", "69.txt", ALL_RELEASE),
            ("696", "70.txt", ALL_RELEASE),
            ("696", "71.txt", ALL_RELEASE),
            ("696", "72.txt", ALL_RELEASE),
            ("696", "73.txt", ALL_RELEASE),
            ("696", "74.txt", ALL_RELEASE),
            ("696", "75.txt", ALL_RELEASE),
            ("696", "76.txt", ALL_RELEASE),
            ("696", "77.txt", ALL_RELEASE),
            ("696", "78.txt", ALL_RELEASE),
            ("696", "79.txt", ALL_RELEASE),
            ("696", "80.txt", ALL_RELEASE),
            ("696", "81.txt", ALL_RELEASE),
            ("696", "82.txt", ALL_RELEASE),
            ("696", "83.txt", ALL_RELEASE),
            ("696", "84.txt", ALL_RELEASE),
            ("696", "85.txt", ALL_RELEASE),
            ("696", "86.txt", ALL_RELEASE),
            ("696", "87.txt", ALL_RELEASE),
            ("696", "88.txt", ALL_RELEASE),
            ("696", "89.txt", ALL_RELEASE),
            ("696", "90.txt", ALL_RELEASE),
            ("696", "91.txt", ALL_RELEASE),
            ("696", "92.txt", ALL_RELEASE),
            ("696", "93.txt", ALL_RELEASE),
            ("696", "94.txt", ALL_RELEASE),
            ("696", "95.txt", ALL_RELEASE),
            ("696", "96.txt", ALL_RELEASE),
            ("696", "97.txt", ALL_RELEASE),
            ("696", "98.txt", ALL_RELEASE),
            ("696", "99.txt", ALL_RELEASE),
            ("boxxle1", "1.txt", ALL_OK),
            ("boxxle1", "2.txt", [RELEASE, RELEASE, OK, OK]),
            ("boxxle1", "3.txt", ALL_OK),
            ("boxxle1", "4.txt", ALL_OK),
            ("boxxle1", "5.txt", ALL_OK),
            ("boxxle1", "6.txt", [SLOW, SLOW, SLOW, RELEASE]),
            ("boxxle1", "7.txt", [RELEASE, RELEASE, OK, OK]),
            ("boxxle1", "8.txt", ALL_OK),
            ("boxxle1", "9.txt", [SLOW, SLOW, RELEASE, RELEASE]),
            ("boxxle1", "10.txt", ALL_OK),
            ("boxxle1", "11.txt", ALL_OK),
            ("boxxle1", "12.txt", ALL_SLOW),
            ("boxxle1", "13.txt", ALL_OK),
            ("boxxle1", "14.txt", ALL_UNSOLVED),
            ("boxxle1", "15.txt", ALL_OK),
            ("boxxle1", "16.txt", ALL_UNSOLVED),
            ("boxxle1", "17.txt", [SLOW, SLOW, SLOW, RELEASE]),
            ("boxxle1", "18.txt", ALL_RELEASE),
            ("boxxle1", "19.txt", ALL_OK),
            ("boxxle1", "20.txt", ALL_OK),
            ("boxxle1", "21.txt", ALL_RELEASE),
            ("boxxle1", "22.txt", ALL_UNSOLVED),
            ("boxxle1", "23.txt", ALL_RELEASE),
            ("boxxle1", "24.txt", ALL_UNSOLVED),
            ("boxxle1", "25.txt", ALL_SLOW),
            ("boxxle1", "26.txt", ALL_UNSOLVED),
            ("boxxle1", "27.txt", ALL_RELEASE),
            ("boxxle1", "28.txt", ALL_RELEASE),
            ("boxxle1", "29.txt", ALL_SLOW),
            ("boxxle1", "30.txt", ALL_UNSOLVED),
            ("boxxle1", "108.txt", ALL_RELEASE),
            ("boxxle2", "1.txt", ALL_OK),
            ("boxxle2", "2.txt", ALL_OK),
            ("boxxle2", "3.txt", [RELEASE, RELEASE, OK, OK]),
            ("boxxle2", "4.txt", [UNSOLVED, UNSOLVED, SLOW, RELEASE]),
            ("boxxle2", "5.txt", ALL_UNSOLVED),
            ("boxxle2", "6.txt", [SLOW, SLOW, SLOW, RELEASE]),
            ("boxxle2", "7.txt", [UNSOLVED, UNSOLVED, SLOW, SLOW]),
            ("boxxle2", "8.txt", ALL_UNSOLVED),
            ("boxxle2", "9.txt", ALL_UNSOLVED),
            ("boxxle2", "10.txt", ALL_UNSOLVED),
            ("original-and-extra", "1.txt", ALL_SLOW),
        ];

        let levels: Vec<_> = levels
            .iter()
            .map(|&(pack, level, difficulties)| {
                (
                    pack,
                    level,
                    difficulties.iter().map(|&d| d <= MAX_DIFFICULTY).collect(),
                )
            })
            .collect();
        test_and_time_levels(&levels);
    }

    #[test]
    #[ignore] // most are simple but there's so many of them that testing all of them takes too long
    fn test_696() {
        let levels: Vec<_> = (100..=696)
            .filter(|&i| i != 250 && i != 693) // currently can't solve these two
            .map(|num| {
                (
                    "696",
                    format!("{}.txt", num),
                    vec![false, false, false, true],
                )
            })
            .collect();
        test_and_time_levels(&levels);
    }

    #[test]
    #[ignore]
    fn test_aymeric() {
        let levels: Vec<_> = (1..=20)
            .map(|num| {
                (
                    "aymeric-cosmonotes",
                    format!("{}.txt", num),
                    vec![false, false, false, true],
                )
            })
            .chain((1..=40).map(|num| {
                (
                    "aymeric-microcosmos",
                    format!("{}.txt", num),
                    vec![false, false, false, true],
                )
            }))
            .chain((1..=40).map(|num| {
                (
                    "aymeric-minicosmos",
                    format!("{}.txt", num),
                    vec![false, false, false, true],
                )
            }))
            .chain((1..=40).map(|num| {
                (
                    "aymeric-nabocosmos",
                    format!("{}.txt", num),
                    vec![false, false, false, true],
                )
            }))
            .chain((1..=20).map(|num| {
                (
                    "aymeric-picocosmos",
                    format!("{}.txt", num),
                    vec![false, false, false, true],
                )
            }))
            .collect();
        test_and_time_levels(&levels);
    }

    #[test]
    #[ignore]
    fn test_microban1() {
        let levels: Vec<_> = (1..=155)
            .filter(|&num| num != 93 && num != 144 && num != 153)
            .map(|num| {
                (
                    "microban1",
                    format!("{}.txt", num),
                    vec![false, false, false, true],
                )
            })
            .collect();
        test_and_time_levels(&levels);
    }

    #[test]
    #[ignore]
    fn test_microban2() {
        let levels: Vec<_> = (1..=135)
            .filter(|&num| num != 66 && num != 102 && num != 104 && num < 100)
            .map(|num| {
                (
                    "microban2",
                    format!("{}.txt", num),
                    vec![false, false, false, true],
                )
            })
            .collect();
        test_and_time_levels(&levels);
    }

    fn test_and_time_levels<L: AsRef<str> + Display>(levels: &[(&str, L, Vec<bool>)]) {
        #![allow(clippy::cast_lossless)]

        let started = Instant::now();

        let mut total_methods = 0;
        let results: Vec<_> = levels
            .iter()
            .filter(|(_, _, methods)| methods[0] || methods[1] || methods[2] || methods[3])
            .map(|(pack, name, methods)| {
                (
                    pack,
                    name,
                    [
                        if methods[0] {
                            total_methods += 1;
                            Some(test_level(pack, name, Method::MovesPushes))
                        } else {
                            None
                        },
                        if methods[1] {
                            total_methods += 1;
                            Some(test_level(pack, name, Method::Moves))
                        } else {
                            None
                        },
                        if methods[2] {
                            total_methods += 1;
                            Some(test_level(pack, name, Method::PushesMoves))
                        } else {
                            None
                        },
                        if methods[3] {
                            total_methods += 1;
                            Some(test_level(pack, name, Method::Pushes))
                        } else {
                            None
                        },
                    ],
                )
            })
            .collect();

        println!(
            "Tested {} levels using on average {:.2} methods per level in {} ms",
            results.len(),
            (total_methods as f64) / (results.len() as f64),
            (started.elapsed().as_millis() as u64).separated_string() // separator doesn't support u128
        );

        let mut all_levels_passed = true;

        // verify that methods which minimize moves/pushes actually produce
        // better or equal numbers than methods which don't
        type OptimalityPred = dyn Fn((i32, i32), (i32, i32)) -> bool;
        let not_optimal =
            |mres: [Option<TestResult>; 4], m1: usize, m2: usize, pred: &OptimalityPred| {
                if let Some(method_res_1) = mres[m1] {
                    if let Some(method_res_2) = mres[m2] {
                        let counts1 = method_res_1.counts.unwrap_or((-1, -1));
                        let counts2 = method_res_2.counts.unwrap_or((-1, -1));

                        if !pred(counts1, counts2) {
                            return true;
                        }
                    }
                }

                false
            };
        #[cfg_attr(rustfmt, rustfmt_skip)]
        let comparisons: &[(_, _, &OptimalityPred)] = &[
            (0, 1, &|(mp_m, mp_p), (m_m, m_p)| mp_m == m_m && mp_p <= m_p),
            (0, 2, &|(mp_m, mp_p), (pm_m, pm_p)| mp_m <= pm_m && mp_p >= pm_p),
            (0, 3, &|(mp_m, mp_p), (p_m, p_p)| mp_m <= p_m && mp_p >= p_p),
            (1, 2, &|(m_m, m_p), (pm_m, pm_p)| m_m <= pm_m && m_p >= pm_p),
            (1, 3, &|(m_m, m_p), (p_m, p_p)| m_m <= p_m && m_p >= p_p),
            (2, 3, &|(pm_m, pm_p), (p_m, p_p)| pm_m <= p_m && pm_p == p_p),
        ];
        for &(pack, name, method_results) in &results {
            if comparisons
                .iter()
                .any(|(m1, m2, is_optimal)| not_optimal(method_results, *m1, *m2, is_optimal))
            {
                println!("Optimality broken: {}/{}", pack, name);
                all_levels_passed = false;
            }
        }

        // print levels that differ from the saved results
        let mut print_bad = |msg, is_bad: fn(TestComparison) -> bool| {
            use self::Method::*;

            let mut bad_levels = Vec::new();
            for &(pack, name, method_results) in &results {
                for (&mres, method) in
                    method_results
                        .iter()
                        .zip(&[MovesPushes, Moves, PushesMoves, Pushes])
                {
                    if let Some(mres) = mres {
                        if is_bad(mres.comparison) {
                            bad_levels.push((pack, name, method))
                        }
                    }
                }
            }
            if !bad_levels.is_empty() {
                all_levels_passed = false;
                println!("{} ({}):", msg, bad_levels.len());
                for (pack, name, method) in bad_levels {
                    println!("\t{}/{} method {}", pack, name, method);
                }
            }
        };

        macro_rules! level_list {
            ($msg:expr, $moves:pat, $pushes:pat, $stats:pat) => {
                print_bad($msg, |cmp| {
                    if let TestComparison::Changed($moves, $pushes, $stats) = cmp {
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

        print_bad("Solvability changed", |cmp| {
            cmp == TestComparison::SolvabilityChanged
        });

        assert!(all_levels_passed);
    }

    #[must_use]
    fn test_level<L: AsRef<str> + Display>(
        level_pack: &str,
        level_name: L,
        method: Method,
    ) -> TestResult {
        let method_name = method.to_string();
        let level_path = format!("levels/{}/{}", level_pack, level_name);
        let result_dir = format!("solutions/{}/{}", method_name, level_pack);
        let result_file = format!("{}/{}", result_dir, level_name);

        println!("Solving level {} using method {}", level_path, method_name);
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
            // we could return here but let's parse the output as a sanity check
        }

        let expected = fs::read_to_string(&result_file).unwrap();

        let (maybe_out_lens, out_created, out_visited) = parse_stats(&out);
        let (maybe_expected_lens, expected_created, expected_visited) = parse_stats(&expected);

        if out == expected {
            return TestResult {
                counts: maybe_out_lens,
                comparison: TestComparison::Ok,
            };
        }

        //print!("\t>>> Expected:\n{}", expected);
        //print!("\t>>> Got:\n{}", out);
        print!("{}", Changeset::new(&expected, &out, "\n"));

        if maybe_out_lens.is_some() != maybe_expected_lens.is_some() {
            println!("\t>>> SOLVABILITY CHANGED <<<\n\n");
            return TestResult {
                counts: maybe_out_lens,
                comparison: TestComparison::SolvabilityChanged,
            };
        }

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

        let stats_change = if out_created > expected_created || out_visited > expected_visited {
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

        TestResult {
            counts: maybe_out_lens,
            comparison: TestComparison::Changed(moves_change, pushes_change, stats_change),
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

        // other stats can go up with a better solution
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
