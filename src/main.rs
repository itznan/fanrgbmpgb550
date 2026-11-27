//! FanRGB: MSI MPG B550 Motherboard & Gigabyte GPU RGB Controller (CLI).

mod cli;
mod config;
mod controller;
mod effects;
mod gpu;
mod visualizer;

use std::env;
use std::process;

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        cli::help::print_help();
        return;
    }

    if let Err(e) = cli::run_cli(&args) {
        eprintln!("[FAIL] {}", e);
        process::exit(1);
    }
}
