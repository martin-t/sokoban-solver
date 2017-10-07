#![cfg_attr(test, feature(proc_macro))]
#[cfg(test)]
extern crate test_case_derive;

extern crate clap;

mod formatter;
mod solver;
mod data;
mod extensions;
mod utils;

use std::env;
use std::process;

use clap::{App, Arg, ArgGroup};

use formatter::Format;

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

    let level = utils::load_file(path).unwrap_or_else(|err| {
        let current_dir = env::current_dir().unwrap();
        println!("Can't read file {} in {:?}: {}", path, current_dir, err);
        process::exit(1);
    });

    let (map, initial_state) = formatter::parse(&level, format).unwrap_or_else(|err| {
        println!("Failed to parse: {}", err);
        process::exit(1);
    });

    /*println!("Empty map:\n{}", map.to_string());
    println!("Initial state:\n{}",
             map.clone().with_state(&initial_state).to_string());*/
    //println!("Expanding: {:?}", expand(&map, &initial_state));

    let mut map_state = map.empty_map_state();

    println!("Dead ends:");
    solver::mark_dead_ends(&mut map_state);

    println!("Solving...");
    let (path_states, stats) = solver::search(&map_state, &initial_state, true);
    println!("{}", stats);
    match path_states {
        Some(path) => {
            println!("Found solution:");
            for state in &path {
                println!("{}", map_state.clone().with_state(state).to_string());
            }
            println!("{} steps", &path.len() - 1);
        }
        None => println!("No solution"),
    }
}

#[cfg(test)]
mod tests {
    use test_case_derive::test_case;
    use super::*;
    use formatter::Format::*;

    /// `expected_path_states` includes initial state
    #[test_case(Xsb, "levels/custom/01-simplest-xsb.txt", Some(2), 2, 2)]
    #[test_case(Custom, "levels/custom/01-simplest-custom.txt", Some(2), 2, 2)]
    #[test_case(Custom, "levels/custom/02-one-way.txt", Some(4), 4, 4)]
    #[test_case(Custom, "levels/custom/03-long-way.txt", Some(9), 10, 9)]
    #[test_case(Custom, "levels/custom/04-two-boxes.txt", Some(21), 313, 148)]
    #[test_case(Custom, "levels/custom/05-google-images-play.txt", Some(4), 11, 6)]
    #[test_case(Custom, "levels/custom/06-google-images-1.txt", Some(10), 341, 117)]
    #[test_case(Custom, "levels/custom/07-boxxle-1-1.txt", Some(32), 1563, 983)]
    #[test_case(Xsb, "levels/custom/no-solution-parking.txt", None, 102, 52)]
    #[test_case(Custom, "levels/custom/easy-2.txt", Some(11), 4673, 488)]
    #[test_case(Custom, "levels/custom/moderate-6.txt", Some(33), 211, 137)]
    #[test_case(Custom, "levels/custom/moderate-7.txt", Some(6), 21, 12)]
    fn test_level(format: Format, level_path: &str, expected_path_states: Option<usize>, created: i32, visited: i32) {
        use std::io::Write;

        let level = utils::load_file(level_path).unwrap();
        let (map, initial_state) = formatter::parse(&level, format).unwrap();
        let mut map_state = map.empty_map_state();
        solver::mark_dead_ends(&mut map_state);
        let (path_states, stats) = solver::search(&map_state, &initial_state, false);

        let stdout = std::io::stdout();
        let mut stdout = stdout.lock();
        writeln!(stdout, "{}", level_path).unwrap();
        match path_states {
            Some(states) => {
                writeln!(stdout, "Path len: {}", states.len()).unwrap();
                assert_eq!(states.len(), expected_path_states.unwrap());
            }
            None => {
                writeln!(stdout, "No solution").unwrap();
                assert_eq!(None, expected_path_states);
            }
        }
        writeln!(stdout, "{:?}", stats).unwrap();
        assert_eq!(stats.total_created(), created);
        assert_eq!(stats.total_visited(), visited);
    }
}
