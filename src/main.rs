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

    #[test_case("custom", "01-simplest-xsb.txt", Xsb)]
    #[test_case("custom", "01-simplest-custom.txt", Custom)]
    #[test_case("custom", "02-one-way.txt", Custom)]
    #[test_case("custom", "03-long-way.txt", Custom)]
    #[test_case("custom", "04-two-boxes.txt", Custom)]
    #[test_case("custom", "05-google-images-play.txt", Custom)]
    #[test_case("custom", "06-google-images-1.txt", Custom)]
    #[test_case("custom", "07-boxxle-1-1.txt", Custom)]
    #[test_case("custom", "easy-2.txt", Custom)]
    #[test_case("custom", "moderate-6.txt", Custom)]
    #[test_case("custom", "moderate-7.txt", Custom)]
    #[test_case("custom", "no-solution-parking.txt", Xsb)]
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
    fn test_levels(level_pack: &str, level_name: &str, format: Format) {
        test_level(level_pack, level_name, format);
    }

    // separate fn to get stack traces with correct line numbers
    fn test_level(level_pack: &str, level_name: &str, format: Format) {
        use std::fmt::Write;

        let level_path = format!("levels/{}/{}", level_pack, level_name);
        let result_file = format!("levels/{}-results/{}", level_pack, level_name);
        println!("{}", level_path);

        let level = utils::read_file(&level_path).unwrap();
        let level = parser::parse(&level, format).unwrap();
        let solution = solver::solve(&level, false).unwrap();

        let mut out = String::new();
        match solution.path_states {
            None => writeln!(out, "No solution").unwrap(),
            Some(states) => writeln!(out, "Path len: {}", states.len()).unwrap(),
        }
        writeln!(out, "{}", solution.stats).unwrap();

        //utils::write_file(&result_file, &out).unwrap(); // uncomment to update results

        let expected = utils::read_file(&result_file).unwrap();
        if out != expected {
            println!("Expected:\n{}", expected);
            println!("Got:\n{}", out);
            assert!(false);
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
