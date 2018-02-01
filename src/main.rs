#![cfg_attr(test, feature(proc_macro))]
#![cfg_attr(test, feature(test))]
#![cfg_attr(test, feature(inclusive_range_syntax))]

#[cfg(test)]
extern crate test_case_derive;
#[cfg(test)]
extern crate test;

extern crate clap;
extern crate separator;

mod parser;
mod solver;
mod level;
mod data;
mod extensions;
mod utils;

use std::env;
use std::process;

use clap::{App, Arg, ArgGroup};

use data::Format;

fn main() {
    let matches = App::new("sokoban-solver")
        .author("martin-t")
        .version("0.0")
        .arg(Arg::with_name("custom")
            .short("-c")
            .long("--custom")
            .help("parse as custom format"))
        .arg(Arg::with_name("xsb")
            .short("-x")
            .long("--xsb")
            .help("parse as XSB format (default)"))
        .group(ArgGroup::with_name("format")
            .arg("custom")
            .arg("xsb"))
        .arg(Arg::with_name("file")
            .required(true))
        .get_matches();

    let format = if matches.is_present("custom") {
        Format::Custom
    } else {
        Format::Xsb
    };
    let path = matches.value_of("file").unwrap();

    let level = utils::read_file(path).unwrap_or_else(|err| {
        let current_dir = env::current_dir().unwrap();
        println!("Can't read file {} in {}: {}", path, current_dir.display(), err);
        process::exit(1);
    });

    let level = parser::parse(&level, format).unwrap_or_else(|err| {
        println!("Failed to parse: {}", err);
        process::exit(1);
    });

    println!("Solving...");
    // TODO use steps instead?
    let solver_ok = solver::solve(&level,  true).unwrap();
    println!("{}", solver_ok.stats);
    match solver_ok.path_states {
        Some(path) => {
            println!("Found solution:");
            for state in &path {
                println!("{}", level.map.to_string(&state, format));
            }
            println!("{} steps", &path.len() - 1);
        }
        None => println!("No solution"),
    }
}

#[cfg(test)]
mod tests {
    use test_case_derive::test_case;
    use test::Bencher;

    use super::*;
    use data::Format::*;

    /// `expected_path_states` includes initial state
    #[test_case(Xsb, "levels/custom/01-simplest-xsb.txt", Some(2), 2, 2)]
    #[test_case(Custom, "levels/custom/01-simplest-custom.txt", Some(2), 2, 2)]
    #[test_case(Custom, "levels/custom/02-one-way.txt", Some(4), 4, 4)]
    #[test_case(Custom, "levels/custom/03-long-way.txt", Some(9), 10, 9)]
    #[test_case(Custom, "levels/custom/04-two-boxes.txt", Some(21), 313, 148)]
    #[test_case(Custom, "levels/custom/05-google-images-play.txt", Some(4), 11, 6)]
    #[test_case(Custom, "levels/custom/06-google-images-1.txt", Some(10), 578, 180)]
    #[test_case(Custom, "levels/custom/07-boxxle-1-1.txt", Some(32), 1552, 977)]
    #[test_case(Xsb, "levels/custom/no-solution-parking.txt", None, 102, 52)]
    #[test_case(Custom, "levels/custom/easy-2.txt", Some(11), 4583, 480)]
    #[test_case(Custom, "levels/custom/moderate-6.txt", Some(33), 211, 137)]
    #[test_case(Custom, "levels/custom/moderate-7.txt", Some(6), 24, 14)]
    fn test_custom(format: Format, level_path: &str, expected_path_states: Option<usize>, created: i32, visited: i32) {
        test_level(format, level_path, expected_path_states, created, visited);
    }

    // separate fn to get stack traces with correct line numbers
    fn test_level(format: Format, level_path: &str, expected_path_states: Option<usize>, created: i32, visited: i32) {
        use std::io::Write;

        let level = utils::read_file(level_path).unwrap();
        let level = parser::parse(&level, format).unwrap();
        let solution = solver::solve(&level, false).unwrap();

        let stdout = std::io::stdout();
        let mut stdout = stdout.lock();
        writeln!(stdout, "{}", level_path).unwrap();
        match solution.path_states {
            Some(states) => {
                writeln!(stdout, "Path len: {}", states.len()).unwrap();
                assert_eq!(states.len(), expected_path_states.unwrap());
            }
            None => {
                writeln!(stdout, "No solution").unwrap();
                assert_eq!(None, expected_path_states);
            }
        }
        writeln!(stdout, "{:?}", solution.stats).unwrap();
        assert_eq!(solution.stats.total_created(), created);
        assert_eq!(solution.stats.total_visited(), visited);
    }

    #[test]
    fn test_boxxle1() {
        // TODO separate worse vs better - keep stats per step
        // TODO print all info - steps, etc.
        // TODO one test for debugging
        use std::fmt::Write;
        use std::thread;

        let mut threads = Vec::new();
        // 2, 6 and 9 are a bit slow in debug mode
        for i in [1, 3, 4, 5, 7, 8, 10].iter() {
            //for i in 1 ... 108 {
            threads.push(thread::spawn(move || {
                let level_path = format!("levels/boxxle1/{}.txt", i);

                let level = utils::read_file(&level_path).unwrap();
                let level = parser::parse(&level, Format::Xsb).unwrap();
                let solution = solver::solve(&level, false).unwrap();

                let mut out = String::new();
                writeln!(out, "{}", level_path).unwrap();
                match solution.path_states {
                    Some(states) => {
                        writeln!(out, "Path len: {}", states.len()).unwrap();
                    }
                    None => {
                        writeln!(out, "No solution").unwrap();
                    }
                }
                writeln!(out, "{:?}", solution.stats).unwrap();

                let result_file = format!("levels/boxxle1-results/{}.txt", i);
                println!("{}", out);
                assert_eq!(out, utils::read_file(result_file).unwrap()); // for testing
                //utils::write_file(result_file, &out).unwrap(); // for updating
            }));
        }
        for t in threads {
            t.join().unwrap();
        }
    }

    #[bench]
    fn bench_boxxle1_1(b: &mut Bencher) {
        let level = utils::read_file("levels/boxxle1/1.txt").unwrap();
        let level = parser::parse(&level, Format::Xsb).unwrap();

        b.iter(|| {
            solver::solve(&level, false)
        });
    }

    #[bench]
    fn bench_boxxle1_5(b: &mut Bencher) {
        let level = utils::read_file("levels/boxxle1/5.txt").unwrap();
        let level = parser::parse(&level, Format::Xsb).unwrap();

        b.iter(|| {
            solver::solve(&level, false)
        });
    }
}
