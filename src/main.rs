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

use std::ffi::OsString;
#[cfg(unix)]
use std::{fs, process};

use clap::{crate_authors, crate_version, Arg, ArgAction, ArgGroup, Command, value_parser};

use sokoban_solver::{
    config::{Format, Method},
    LoadLevel, Solve,
};

fn main() {
    // Use consts for strings which appear in multiple places.
    // If anybody thinks this is overkill, i made a typo twice already.
    const CUSTOM: &str = "custom";
    const XSB: &str = "xsb";
    const MOVES_PUSHES: &str = "moves-pushes";
    const MOVES: &str = "moves";
    const PUSHES_MOVES: &str = "pushes-moves";
    const PUSHES: &str = "pushes";
    const ANY: &str = "any";
    const LEVEL_FILE: &str = "level-file";
    const VERBOSE: &str = "verbose";

    let app = Command::new("sokoban-solver")
        .author(crate_authors!())
        .version(crate_version!())
        .arg(
            Arg::new(CUSTOM)
                .short('c')
                .long(CUSTOM)
                .help("Output in the custom format")
                .action(ArgAction::SetTrue)
                .conflicts_with(XSB),
        )
        .arg(
            Arg::new(XSB)
                .short('x')
                .long(XSB)
                .help("Output in the XSB format (default)")
                .action(ArgAction::SetTrue),
        )
        .group(ArgGroup::new("format").args(&[CUSTOM, XSB]))
        .arg(
            Arg::new(MOVES_PUSHES)
                .short('M')
                .long(MOVES_PUSHES)
                .help("Search for a move-optimal solution with minimal pushes")
                .action(ArgAction::SetTrue)
                .conflicts_with_all(&[MOVES, PUSHES_MOVES, PUSHES, ANY]),
        )
        .arg(
            Arg::new(MOVES)
                .short('m')
                .long(MOVES)
                .help("Search for a move-optimal solution")
                .action(ArgAction::SetTrue)
                .conflicts_with_all(&[PUSHES_MOVES, PUSHES, ANY]),
        )
        .arg(
            Arg::new(PUSHES_MOVES)
                .short('P')
                .long(PUSHES_MOVES)
                .help("Search for a push-optimal solution with minimal moves")
                .action(ArgAction::SetTrue)
                .conflicts_with_all(&[PUSHES, ANY]),
        )
        .arg(
            Arg::new(PUSHES)
                .short('p')
                .long(PUSHES)
                .help("Search for a push-optimal solution")
                .action(ArgAction::SetTrue)
                .conflicts_with_all(&[ANY]),
        )
        .arg(
            Arg::new(ANY)
                .short('a')
                .long(ANY)
                .help("Search for any solution (default, currently push optimal)")
                .action(ArgAction::SetTrue),
        )
        .group(ArgGroup::new("method").args(&[MOVES_PUSHES, MOVES, PUSHES_MOVES, PUSHES, ANY]))
        .arg(
            Arg::new(LEVEL_FILE)
                .value_parser(value_parser!(OsString))
                .required(true)
                .action(ArgAction::Append),
        );

    #[cfg(debug_assertions)]
    let app = app.arg(
        Arg::new(VERBOSE)
            .short('v')
            .long(VERBOSE)
            .help("Print all log levels (only available in debug builds)"),
    );

    let matches = app.get_matches();

    let format = if matches.get_flag(CUSTOM) {
        Format::Custom
    } else {
        Format::Xsb
    };

    let method = if matches.get_flag(MOVES_PUSHES) {
        Method::MovesPushes
    } else if matches.get_flag(MOVES) {
        Method::Moves
    } else if matches.get_flag(PUSHES_MOVES) {
        Method::PushesMoves
    } else if matches.get_flag(PUSHES) {
        Method::Pushes
    } else {
        Method::Any
    };

    let verbose = matches.get_flag(VERBOSE);

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
        .get_many::<OsString>(LEVEL_FILE)
        .expect("Level path is required")
    {
        let level = path.load_level().unwrap_or_else(|err| {
            eprintln!("Can't load level: {err}");
            process::exit(1);
        });

        println!("Solving {}...", path.to_string_lossy());
        let solver_ok = level.solve(method, true).unwrap_or_else(|err| {
            eprintln!("Invalid level: {err}");
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
                println!("{moves}");
                println!("Moves: {}", moves.move_cnt());
                println!("Pushes: {}", moves.push_cnt());
            }
        }
    }
}
