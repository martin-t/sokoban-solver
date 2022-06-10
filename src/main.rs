// Additional warnings that are allow by default (`rustc -W help`)
#![warn(missing_copy_implementations)]
#![warn(missing_debug_implementations)]
#![warn(trivial_casts)]
#![warn(trivial_numeric_casts)]
#![warn(unreachable_pub)]
#![warn(unused)]
#![warn(clippy::all)]
// Enable pedantic since about two thirds seem useful to me, then disable individual lints which are too strict:
#![warn(clippy::pedantic)]
#![allow(clippy::items_after_statements)]
#![allow(clippy::too_many_lines)]
// ^ End of pedantic overrides

#[cfg(unix)]
use std::fs;
use std::process;

use clap::Parser;

use sokoban_solver::config::{Format, Method};
use sokoban_solver::{LoadLevel, Solve};

#[derive(Parser)]
struct Args {
    format: Format,

    method: Method,

    #[clap(long, short)]
    verbose: bool,
}

fn main() {
    // // if anybody thinks this is overkill, i made a typo twice already
    // const CUSTOM: &str = "custom";
    // const XSB: &str = "xsb";
    // const MOVES_PUSHES: &str = "moves-pushes";
    // const MOVES: &str = "moves";
    // const PUSHES_MOVES: &str = "pushes-moves";
    // const PUSHES: &str = "pushes";
    // const ANY: &str = "any";
    // const LEVEL_FILE: &str = "level-file";
    // const VERBOSE: &str = "verbose";

    // let app = Command::new("sokoban-solver")
    //     .author(crate_authors!())
    //     .version(crate_version!())
    //     .arg(
    //         Arg::with_name(CUSTOM)
    //             .short("c")
    //             .long(CUSTOM)
    //             .help("Output in the custom format"),
    //     )
    //     .arg(
    //         Arg::with_name(XSB)
    //             .short("x")
    //             .long(XSB)
    //             .help("Output in the XSB format (default)"),
    //     )
    //     .group(ArgGroup::with_name("format").args(&[CUSTOM, XSB]))
    //     .arg(
    //         Arg::with_name(MOVES_PUSHES)
    //             .short("M")
    //             .long(MOVES_PUSHES)
    //             .help("Search for a move-optimal solution with minimal pushes"),
    //     )
    //     .arg(
    //         Arg::with_name(MOVES)
    //             .short("m")
    //             .long(MOVES)
    //             .help("Search for a move-optimal solution"),
    //     )
    //     .arg(
    //         Arg::with_name(PUSHES_MOVES)
    //             .short("P")
    //             .long(PUSHES_MOVES)
    //             .help("Search for a push-optimal solution with minimal moves"),
    //     )
    //     .arg(
    //         Arg::with_name(PUSHES)
    //             .short("p")
    //             .long(PUSHES)
    //             .help("Search for a push-optimal solution"),
    //     )
    //     .arg(
    //         Arg::with_name(ANY)
    //             .short("a")
    //             .long(ANY)
    //             .help("Search for any solution (default, currently push optimal)"),
    //     )
    //     .group(ArgGroup::with_name("method").args(&[
    //         MOVES_PUSHES,
    //         MOVES,
    //         PUSHES_MOVES,
    //         PUSHES,
    //         ANY,
    //     ]))
    //     .arg(Arg::with_name(LEVEL_FILE).required(true).multiple(true));

    // #[cfg(debug_assertions)]
    // let app = app.arg(
    //     Arg::with_name(VERBOSE)
    //         .short("v")
    //         .long(VERBOSE)
    //         .help("Print all log levels (only available in debug builds)"),
    // );

    // let matches = app.get_matches();

    // let format = if matches.is_present(CUSTOM) {
    //     Format::Custom
    // } else {
    //     Format::Xsb
    // };

    // let method = if matches.is_present(MOVES_PUSHES) {
    //     Method::MovesPushes
    // } else if matches.is_present(MOVES) {
    //     Method::Moves
    // } else if matches.is_present(PUSHES_MOVES) {
    //     Method::PushesMoves
    // } else if matches.is_present(PUSHES) {
    //     Method::Pushes
    // } else {
    //     Method::Any
    // };

    // let verbose = matches.is_present(VERBOSE);

    let args = Args::parse();

    let log_level = if verbose {
        log::LevelFilter::Trace
    } else {
        log::LevelFilter::Info
    };
    env_logger::Builder::from_default_env()
        .filter_level(log_level)
        .init();

    // Chrome uses 300 (which means vscode does too) and gets killed when trying to solve hard levels.
    #[cfg(unix)]
    fs::write(
        &format!("/proc/{}/oom_score_adj", process::id()),
        500.to_string(),
    )
    .unwrap_or_else(|_| eprintln!("Couldn't change oom_score_adj"));

    for path in matches
        .values_of_os(LEVEL_FILE)
        .expect("Level path is required")
    {
        let level = path.load_level().unwrap_or_else(|err| {
            eprintln!("Can't load level: {}", err);
            process::exit(1);
        });

        println!("Solving {}...", path.to_string_lossy());
        let solver_ok = level.solve(method, true).unwrap_or_else(|err| {
            eprintln!("Invalid level: {}", err);
            process::exit(1);
        });

        match solver_ok.moves {
            None => {
                println!("No solution");
                println!("{}", solver_ok.stats);
            }
            Some(moves) => {
                let include_steps = method == Method::Moves;
                println!("Found solution:");
                print!("{}", level.format_solution(format, &moves, include_steps));
                println!("{}", solver_ok.stats);
                println!("{}", moves);
                println!("Moves: {}", moves.move_cnt());
                println!("Pushes: {}", moves.push_cnt());
            }
        }
    }
}
