// Opt in to unstable features expected for Rust 2018
#![feature(rust_2018_preview)]
// Opt in to warnings about new 2018 idioms
#![warn(rust_2018_idioms)]
// https://github.com/rust-lang/rust/issues/31844
#![feature(specialization)]
#![cfg_attr(test, feature(proc_macro))]
#![cfg_attr(test, feature(proc_macro_gen))]
#![cfg_attr(test, feature(test))]
#![allow(unknown_lints)]

#[cfg(test)]
extern crate test;
#[cfg(test)]
extern crate test_case_derive;

#[macro_use]
extern crate clap;
extern crate separator;

mod data;
mod level;
mod map;
mod parser;
mod solver;
mod utils;
mod vec2d;

use std::env;
use std::process;

use clap::{App, Arg, ArgGroup};

use data::Format;
use map::Map;
use solver::Method;

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
    use test_case_derive::test_case;

    use super::*;

    // additional parens are a workaround for https://github.com/synek317/test-case-derive/issues/2
    #[test_case(("custom", "01-simplest-xsb.txt"))]
    #[test_case("custom", "01-simplest-custom.txt")]
    #[test_case("custom", "02-one-way.txt")]
    #[test_case("custom", "03-long-way.txt")]
    #[test_case("custom", "04-two-boxes-no-packing.txt")]
    //#[test_case("custom", "04-two-boxes-remover.txt")]
    #[test_case("custom", "04-two-boxes.txt")]
    #[test_case("custom", "no-solution-parking.txt")]
    //#[test_case("custom", "original-sokoban-01-remover.txt")]
    //#[test_case("custom", "supaplex.txt")]
    #[test_case("boxxle1", "1.txt")]
    #[test_case("boxxle1", "2.txt")]
    #[test_case("boxxle1", "3.txt")]
    #[test_case("boxxle1", "4.txt")]
    #[test_case("boxxle1", "5.txt")]
    #[test_case("boxxle1", "6.txt")]
    #[test_case("boxxle1", "7.txt")]
    #[test_case("boxxle1", "8.txt")]
    #[test_case("boxxle1", "9.txt")]
    #[test_case("boxxle1", "10.txt")]
    fn push_optimal(level_pack: &str, level_name: &str) {
        test_level(level_pack, level_name, Method::Pushes);
    }

    #[test_case(("custom", "01-simplest-xsb.txt"))]
    #[test_case("custom", "01-simplest-custom.txt")]
    #[test_case("custom", "02-one-way.txt")]
    #[test_case("custom", "03-long-way.txt")]
    #[test_case("custom", "04-two-boxes-no-packing.txt")]
    //#[test_case("custom", "04-two-boxes-remover.txt")]
    #[test_case("custom", "04-two-boxes.txt")]
    #[test_case("custom", "no-solution-parking.txt")]
    //#[test_case("custom", "original-sokoban-01-remover.txt")]
    //#[test_case("custom", "supaplex.txt")]
    #[test_case("boxxle1", "1.txt")]
    //#[test_case("boxxle1", "2.txt")]
    #[test_case("boxxle1", "3.txt")]
    #[test_case("boxxle1", "4.txt")]
    #[test_case("boxxle1", "5.txt")]
    //#[test_case("boxxle1", "6.txt")]
    #[test_case("boxxle1", "7.txt")]
    #[test_case("boxxle1", "8.txt")]
    //#[test_case("boxxle1", "9.txt")]
    #[test_case("boxxle1", "10.txt")]
    fn move_optimal(level_pack: &str, level_name: &str) {
        test_level(level_pack, level_name, Method::Moves);
    }

    // separate fn to get stack traces with correct line numbers
    fn test_level(level_pack: &str, level_name: &str, method: Method) {
        use std::fmt::Write;

        let res_folder = method.to_string().to_lowercase();
        let level_path = format!("levels/{}/{}", level_pack, level_name);
        let result_file = format!("solutions/{}-{}/{}", level_pack, res_folder, level_name);
        println!("{}", level_path);

        let level = utils::read_file(&level_path).unwrap();
        let level = level.parse().unwrap();
        let solution = solver::solve(&level, method, false).unwrap();

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

    #[bench]
    fn bench_boxxle1_001(b: &mut Bencher) {
        // 3 goals in a row
        bench_level("levels/boxxle1/1.txt", Method::Pushes, b);
    }

    #[bench]
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
