//! FanRGB: MSI MPG B550 Motherboard & Gigabyte GPU RGB Controller (CLI).

mod cli;
mod config;
mod controller;
pub mod effects;
mod gpu;
mod visualizer;

use std::env;

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        cli::help::print_help();
        return;
    }

    cli::run_cli(&args);
}
