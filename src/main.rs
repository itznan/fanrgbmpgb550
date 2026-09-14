//! FanRGB: MSI MPG B550 Motherboard & Gigabyte GPU RGB Controller.

mod cli;
mod config;
mod controller;
mod gpu;
mod gui;
mod visualizer;

use std::env;

fn main() {
    let args: Vec<String> = env::args().collect();

    // Double-clicking in Explorer or running without arguments opens the GUI
    if args.len() < 2 {
        if let Err(e) = gui::run_gui() {
            eprintln!("[GUI Error]: {}", e);
        }
        return;
    }

    let first = args[1].to_lowercase();
    if first == "gui" || first == "--gui" || first == "-g" {
        if let Err(e) = gui::run_gui() {
            eprintln!("[GUI Error]: {}", e);
        }
        return;
    }

    cli::run_cli(&args);
}
