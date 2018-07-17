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

#[macro_use]
extern crate clap;

extern crate sokoban_solver;

use std::env;
use std::process;

use clap::{App, Arg, ArgGroup};

use sokoban_solver::config::{Format, Method};
use sokoban_solver::map::Map;
use sokoban_solver::solver;
use sokoban_solver::utils;

fn main() {
    let matches = App::new("sokoban-solver")
        .author(crate_authors!())
        .version(crate_version!())
        .arg(
            Arg::with_name("custom")
                .short("-c")
                .long("--custom")
                .help("print as custom format"),
        )
        .arg(
            Arg::with_name("xsb")
                .conflicts_with("custom")
                .short("-x")
                .long("--xsb")
                .help("print as XSB format (default)"),
        )
        .group(ArgGroup::with_name("format").args(&["custom", "xsb"]))
        .arg(
            Arg::with_name("moves")
                .short("-m")
                .long("--moves")
                .help("search for move-optimal solution"),
        )
        .arg(
            Arg::with_name("pushes")
                .conflicts_with("moves")
                .short("-p")
                .long("--pushes")
                .help("search for push-optimal solution (default)"),
        )
        .group(ArgGroup::with_name("method").args(&["moves", "pushes"]))
        .arg(Arg::with_name("level-file").required(true))
        .get_matches();

    let format = if matches.is_present("custom") {
        Format::Custom
    } else {
        Format::Xsb
    };
    let method = if matches.is_present("moves") {
        Method::Moves
    } else {
        Method::Pushes
    };
    let path = matches.value_of("level-file").unwrap();

    let level = utils::read_file(path).unwrap_or_else(|err| {
        let current_dir = env::current_dir().unwrap();
        println!(
            "Can't read file {} in {}: {}",
            path,
            current_dir.display(),
            err
        );
        process::exit(1);
    });

    let level = level.parse().unwrap_or_else(|err| {
        println!("Failed to parse: {}", err);
        process::exit(1);
    });

    println!("Solving...");
    // TODO use steps/moves/pushes/actions instead
    let solver_ok = solver::solve(&level, method, true).unwrap_or_else(|err| {
        println!("Invalid level: {}", err);
        process::exit(1);
    });
    println!("{}", solver_ok.stats);
    match solver_ok.path_states {
        Some(path) => {
            println!("Found solution:");
            for state in &path {
                println!("{}", level.map.format_with_state(format, &state));
            }
            println!("{} steps", &path.len() - 1);
        }
        None => println!("No solution"),
    }
}

#[cfg(test)]
mod tests {
    use test::Bencher;

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
        for &(method, level_pack, level_name) in levels.iter() {
            test_level(method, level_pack, level_name);
        }
    }

    fn test_level(method: Method, level_pack: &str, level_name: &str) {
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

        let level = utils::read_file(&level_path).unwrap();
        let level = level.parse().unwrap();
        let solution = solver::solve(&level, method, false).unwrap();

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
        //utils::write_file(&result_file, &out).unwrap();

        let expected = utils::read_file(&result_file).unwrap();
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
                //utils::write_file(&result_file, &out).unwrap();
            }

            assert!(false);
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
        let level = utils::read_file(level_path).unwrap();
        let level = level.parse().unwrap();

        b.iter(|| {
            test::black_box(solver::solve(
                test::black_box(&level),
                test::black_box(method),
                test::black_box(false),
            ))
        });
    }
}
