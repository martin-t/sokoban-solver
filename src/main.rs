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
#![warn(unknown_lints)]

use std::process;

use clap::{crate_authors, crate_version};
use clap::{App, Arg, ArgGroup};
use env_logger;
use log;

use sokoban_solver::config::{Format, Method};
use sokoban_solver::{LoadLevel, Solve};

fn main() {
    let app = App::new("sokoban-solver")
        .author(crate_authors!())
        .version(crate_version!())
        .arg(
            Arg::with_name("custom")
                .short("-c")
                .long("--custom")
                .help("print as custom format"),
        ).arg(
            Arg::with_name("xsb")
                .short("-x")
                .long("--xsb")
                .help("print as XSB format (default)"),
        ).group(ArgGroup::with_name("format").args(&["custom", "xsb"]))
        .arg(
            Arg::with_name("move-optimal")
                .short("-m")
                .long("--move-optimal")
                .help("search for move-optimal solution"),
        ).arg(
            Arg::with_name("push-optimal")
                .short("-p")
                .long("--push-optimal")
                .help("search for push-optimal solution (default)"),
        ).group(ArgGroup::with_name("method").args(&["move-optimal", "push-optimal"]))
        .arg(Arg::with_name("level-file").required(true).multiple(true));

    #[cfg(debug_assertions)]
    let app = app.arg(
        Arg::with_name("verbose")
            .short("-v")
            .long("--verbose")
            .help("Print all log levels (only in debug builds)"),
    );

    let matches = app.get_matches();

    let format = if matches.is_present("custom") {
        Format::Custom
    } else {
        Format::Xsb
    };
    let method = if matches.is_present("move-optimal") {
        Method::MoveOptimal
    } else {
        Method::PushOptimal
    };

    let verbose = matches.is_present("verbose");

    let log_level = if verbose {
        log::LevelFilter::Trace
    } else {
        log::LevelFilter::Info
    };
    env_logger::Builder::from_default_env()
        .filter_level(log_level)
        .init();

    for path in matches.values_of_os("level-file").unwrap() {
        let level = path.load_level().unwrap_or_else(|err| {
            eprintln!("Can't load level: {}", err);
            process::exit(1);
        });

        println!("Solving {}...", path.to_string_lossy());
        let solver_ok = level.solve(method, true).unwrap_or_else(|err| {
            eprintln!("Invalid level: {}", err);
            process::exit(1);
        });

        println!("{}", solver_ok.stats);
        match solver_ok.moves {
            None => println!("No solution"),
            Some(moves) => {
                let include_steps = method == Method::MoveOptimal;
                println!("Found solution:");
                print!("{}", level.format_solution(format, &moves, include_steps));
                println!("{}", moves);
                println!("Moves: {}", moves.move_cnt());
                println!("Pushes: {}", moves.push_cnt());
            }
        }
    }
}
