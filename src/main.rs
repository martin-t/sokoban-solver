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

#[macro_use]
extern crate clap;
extern crate env_logger;

extern crate sokoban_solver;

use std::process;

use clap::{App, Arg, ArgGroup};

use sokoban_solver::config::{Format, Method};
use sokoban_solver::map::Map;
use sokoban_solver::{LoadLevel, Solve};

fn main() {
    // show all logs unless disabled in Cargo.toml
    env_logger::Builder::from_default_env()
        .filter_level(log::LevelFilter::Trace)
        .init();

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
        .arg(Arg::with_name("level-file").required(true).multiple(true))
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

    for path in matches.values_of_os("level-file").unwrap() {
        let level = path.load_level().unwrap_or_else(|err| {
            eprintln!("Can't load level: {}", err);
            process::exit(1);
        });

        println!("Solving {}...", path.to_string_lossy());
        // TODO use steps/moves/pushes/actions instead
        let solver_ok = level.solve(method, true).unwrap_or_else(|err| {
            eprintln!("Invalid level: {}", err);
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
        println!();
    }
}
