// https://github.com/rust-lang/rust/issues/31844
#![feature(specialization)]
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

extern crate separator;

pub mod config;
pub mod data;
pub mod level;
pub mod map; // TODO maybe not pub?
pub mod parser;
pub mod solver;
pub mod utils;

mod vec2d;
