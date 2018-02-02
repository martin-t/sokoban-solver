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
    let solver_ok = solver::solve(&level, true).unwrap();
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
    #[test_case("levels/custom/01-simplest-xsb.txt", Xsb, Some(2), 2, 2)]
    #[test_case("levels/custom/01-simplest-custom.txt", Custom, Some(2), 2, 2)]
    #[test_case("levels/custom/02-one-way.txt", Custom, Some(4), 4, 4)]
    #[test_case("levels/custom/03-long-way.txt", Custom, Some(9), 10, 9)]
    #[test_case("levels/custom/04-two-boxes.txt", Custom, Some(21), 313, 148)]
    #[test_case("levels/custom/05-google-images-play.txt", Custom, Some(4), 11, 6)]
    #[test_case("levels/custom/06-google-images-1.txt", Custom, Some(10), 578, 180)]
    #[test_case("levels/custom/07-boxxle-1-1.txt", Custom, Some(32), 1552, 977)]
    #[test_case("levels/custom/no-solution-parking.txt", Xsb, None, 102, 52)]
    #[test_case("levels/custom/easy-2.txt", Custom, Some(11), 4583, 480)]
    #[test_case("levels/custom/moderate-6.txt", Custom, Some(33), 211, 137)]
    #[test_case("levels/custom/moderate-7.txt", Custom, Some(6), 24, 14)]
    fn test_custom(level_path: &str, format: Format, expected_path_states: Option<usize>, created: i32, visited: i32) {
        test_level(format, level_path, expected_path_states, created, visited);
    }

    // separate fn to get stack traces with correct line numbers
    fn test_level(format: Format, level_path: &str, expected_path_states: Option<usize>, created: i32, visited: i32) {
        let level = utils::read_file(level_path).unwrap();
        let level = parser::parse(&level, format).unwrap();
        let solution = solver::solve(&level, false).unwrap();

        println!("{}", level_path);
        match solution.path_states {
            Some(states) => {
                println!("Path len: {}", states.len());
                assert_eq!(states.len(), expected_path_states.unwrap());
            }
            None => {
                println!("No solution");
                assert_eq!(None, expected_path_states);
            }
        }
        println!("{:?}", solution.stats);
        assert_eq!(solution.stats.total_created(), created);
        assert_eq!(solution.stats.total_unique_visited(), visited);
    }

    // 2, 6 and 9 are a bit slow in debug mode
    #[test_case("boxxle1", "1.txt", Xsb)]
    //#[test_case("boxxle1", "2.txt", Xsb)]
    #[test_case("boxxle1", "3.txt", Xsb)]
    #[test_case("boxxle1", "4.txt", Xsb)]
    #[test_case("boxxle1", "5.txt", Xsb)]
    //#[test_case("boxxle1", "6.txt", Xsb)]
    #[test_case("boxxle1", "7.txt", Xsb)]
    #[test_case("boxxle1", "8.txt", Xsb)]
    //#[test_case("boxxle1", "9.txt", Xsb)]
    #[test_case("boxxle1", "10.txt", Xsb)]
    fn test_boxxle1(level_pack: &str, level_name: &str, format: Format) {
        test_level2(level_pack, level_name, format);
    }

    fn test_level2(level_pack: &str, level_name: &str, format: Format) {
        // TODO readable stats, per step?
        // TODO one test for debugging - readable output
        use std::fmt::Write;

        let level_path = format!("levels/{}/{}", level_pack, level_name);
        let result_file = format!("levels/{}-results/{}", level_pack, level_name);

        let level = utils::read_file(&level_path).unwrap();
        let level = parser::parse(&level, format).unwrap();
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
        println!("{}", out);
        assert_eq!(out, utils::read_file(result_file).unwrap()); // for testing
        //utils::write_file(result_file, &out).unwrap(); // for updating
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
