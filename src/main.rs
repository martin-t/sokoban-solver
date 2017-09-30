extern crate clap;

mod formatter;
mod solver;
mod data;
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
            .help("parse as XSB format"))
        .group(ArgGroup::with_name("format")
            .arg("custom")
            .arg("xsb")
            .required(true))
        .arg(Arg::with_name("file"))
        .get_matches();

    let path = matches.value_of("file").unwrap();

    let puzzle = utils::load_file(path).unwrap_or_else(|err| {
        let current_dir = env::current_dir().unwrap();
        println!("Can't read file {} in {:?}: {}", path, current_dir, err);
        process::exit(1);
    });

    let (mut map, initial_state) = formatter::parse_custom(&puzzle).unwrap_or_else(|err| {
        println!("Failed to parse: {}", err);
        process::exit(1);
    });

    /*println!("Empty map:\n{}", map.to_string());
    println!("Initial state:\n{}",
             map.clone().with_state(&initial_state).to_string());*/
    //println!("Expanding: {:?}", expand(&map, &initial_state));

    println!("Dead ends:");
    solver::mark_dead_ends(&mut map);

    println!("Solving...");
    match solver::search(&map, &initial_state, true) {
        Some(path) => {
            println!("Found solution:");
            for state in &path {
                println!("{}", map.clone().with_state(state).to_string());
            }
            println!("{} moves", &path.len() - 1);
        }
        None => println!("No solution"),
    }
}
