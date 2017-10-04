extern crate clap;

mod formatter;
mod solver;
mod data;
mod extensions;
mod utils;

use std::env;
use std::process;

use clap::{App, Arg, ArgGroup};

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

    let custom = matches.is_present("custom");
    let path = matches.value_of("file").unwrap();

    let level = utils::load_file(path).unwrap_or_else(|err| {
        let current_dir = env::current_dir().unwrap();
        println!("Can't read file {} in {:?}: {}", path, current_dir, err);
        process::exit(1);
    });

    let (map, initial_state) = formatter::parse(&level, custom).unwrap_or_else(|err| {
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
    match solver::search(&map_state, &initial_state, true) {
        Some(path) => {
            println!("Found solution:");
            for state in &path {
                println!("{}", map_state.clone().with_state(state).to_string());
            }
            println!("{} moves", &path.len() - 1);
        }
        None => println!("No solution"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn simplest_xsb() {
        let level = r"
#####
#@$.#
#####
";
        let (map, initial_state) = formatter::parse(level, false).unwrap();
        let mut map_state = map.empty_map_state();
        solver::mark_dead_ends(&mut map_state);
        let path = solver::search(&map_state, &initial_state, false).unwrap();
        assert_eq!(path.len(), 2); // initial + 1 step
    }

    #[test]
    fn no_solution() {
        let level = r"
#########
#  #  . #
# $@$   #
#    #. #
#########
";
        let (map, initial_state) = formatter::parse(level, false).unwrap();
        let mut map_state = map.empty_map_state();
        solver::mark_dead_ends(&mut map_state);
        assert_eq!(solver::search(&map_state, &initial_state, false), None);
    }

    #[test]
    fn solve_all_custom() {
        use utils;
        use formatter;
        use std::path::Path;

        // original-sokoban-01.txt and original-sokoban-02.txt are too hard for now
        // so is suppaplex.txt

        let files = "\
01.txt
01-one-way-1.txt
01-one-way-2.txt
02-long-way.txt
02-two-boxes.txt
03-google-images-play.txt
04-google-images-1.txt
04-boxxle-1-1.txt
easy-2.txt
moderate-6.txt
moderate-7.txt";
        for file in files.lines() {
            println!("{}", file);
            let level = utils::load_file(Path::new("levels/custom").join(file)).unwrap();
            let (map, state) = formatter::parse(&level, true).unwrap();
            let mut map_state = map.empty_map_state();
            solver::mark_dead_ends(&mut map_state);
            let path = solver::search(&map_state, &state, false).unwrap();
        }
    }
}
