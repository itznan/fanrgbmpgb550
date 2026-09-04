//! Unified CLI for MSI MPG B550 Motherboard & Gigabyte GPU RGB.

mod config;
mod controller;
mod gpu;
mod visualizer;

use std::env;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};

use config::*;
use controller::*;
use gpu::*;
use visualizer::*;

fn parse_color(args: &[String]) -> Result<((u8, u8, u8), &[String]), String> {
    if args.is_empty() {
        return Err("No color arguments provided.".to_string());
    }

    let first = &args[0];
    let hex_str = first.trim_start_matches('#');
    if hex_str.len() == 6 && hex_str.chars().all(|c| c.is_ascii_hexdigit()) {
        let r = u8::from_str_radix(&hex_str[0..2], 16).map_err(|e| e.to_string())?;
        let g = u8::from_str_radix(&hex_str[2..4], 16).map_err(|e| e.to_string())?;
        let b = u8::from_str_radix(&hex_str[4..6], 16).map_err(|e| e.to_string())?;
        return Ok(((r, g, b), &args[1..]));
    }

    if let Some(rgb) = get_color_preset(first) {
        return Ok((rgb, &args[1..]));
    }

    if args.len() >= 3 {
        let r = args[0].parse::<u8>().map_err(|_| "Invalid R component")?;
        let g = args[1].parse::<u8>().map_err(|_| "Invalid G component")?;
        let b = args[2].parse::<u8>().map_err(|_| "Invalid B component")?;
        return Ok(((r, g, b), &args[3..]));
    }

    Err(format!("Cannot parse color from: {:?}", args))
}

fn print_help() {
    println!(
        r#"
MSI Motherboard & Gigabyte GPU RGB Controller CLI (Rust Edition)

Color values can be specified as:
  - A preset name : red, blue, green, off, ...
  - Hex           : #RRGGBB  or  RRGGBB  (e.g. #FF0000 or FF0000)
  - RGB integers  : R G B               (e.g. 255 0 0)

=== Motherboard Commands ===
  fanrgb status
  fanrgb <preset_name>             (red, blue, green, off, etc.)
  fanrgb <#RRGGBB|RRGGBB>         (e.g. #FF0000 or FF0000)
  fanrgb <r> <g> <b>               (e.g. 255 0 0)
  fanrgb mode <animation_mode>     (rainbow_wave, breathing, meteor, etc.)
  fanrgb mode <animation_mode> <#RRGGBB|R G B>

=== GPU Commands ===
  fanrgb gpu status                (Checks NVAPI connection and I2C address)
  fanrgb gpu <preset_name>         (e.g. gpu red, gpu blue, gpu off)
  fanrgb gpu <#RRGGBB|RRGGBB>     (e.g. gpu #FF0000 or gpu FF0000)
  fanrgb gpu <r> <g> <b>           (e.g. gpu 255 0 0)
  fanrgb gpu mode <mode_name>      (breathing, flash, color_cycle, wave)

=== Synchronized Control (Motherboard + GPU) ===
  fanrgb sync <preset_name>        (e.g. sync red, sync off)
  fanrgb sync <#RRGGBB|RRGGBB>    (e.g. sync #FF0000 or sync FF0000)
  fanrgb sync <r> <g> <b>          (e.g. sync 255 0 0)
  fanrgb bass [--gpu]              (Pure red bass visualizer synced to Motherboard & GPU)

Examples:
  fanrgb bass --gpu                # Bass visualizer pulsing Motherboard AND GPU logo
  fanrgb sync red                  # Set entire PC to pure red
  fanrgb sync #FF0000              # Same as above, using hex
  fanrgb gpu mode breathing        # GPU pulsing red breathing mode
  fanrgb sync off                  # Power down all RGB across the system
"#
    );
}

fn run_cli_bass(sync_gpu: bool) {
    println!("=================================================================");
    println!(" Pure Red Bass & Kick-Drum Visualizer (Terminal Mode - Rust)");
    println!("=================================================================");
    println!("Audio Source: Default Playback Device (WASAPI Loopback)");
    println!("Color: 100% Pure Red (Zero orange, zero amber)");
    if sync_gpu {
        println!("Hardware Sync: MSI B550 Motherboard + Gigabyte GPU");
    } else {
        println!("Hardware Sync: MSI B550 Motherboard (use --gpu to include GPU)");
    }
    println!("Press Ctrl+C to stop.\n");

    let running = Arc::new(AtomicBool::new(true));
    let r_clone = Arc::clone(&running);

    let metrics = Arc::new(Mutex::new(AudioMetrics::default()));

    ctrlc::set_handler(move || {
        r_clone.store(false, Ordering::SeqCst);
    })
    .expect("Error setting Ctrl-C handler");

    start_audio_capture(Arc::clone(&running), Arc::clone(&metrics), 28.0, 90.0);

    let config = VisualizerConfig {
        mode: "hybrid".to_string(),
        sync_gpu,
        sensitivity: 1.0,
        threshold: 0.05,
        gamma: 1.8,
        decay: 0.82,
        min_brightness: 0,
    };

    run_visualizer(running, metrics, config);
}

fn handle_gpu_command(args: &[String]) {
    if args.is_empty() {
        print_help();
        return;
    }

    let sub = args[0].to_lowercase();
    let mut gpu = GigabyteGPURGB::new(None);

    if sub == "status" {
        if gpu.probe_and_connect() {
            println!("[OK] Gigabyte GPU Detected: {}", gpu.gpu_name);
            println!(
                "[OK] Controller I2C Address: 0x{:02X} (Port 1)",
                gpu.active_address.unwrap()
            );
        } else {
            println!("[FAIL] Could not detect Gigabyte GPU RGB controller.");
        }
        return;
    }

    if !gpu.probe_and_connect() {
        println!("[FAIL] Unable to connect to GPU RGB controller via NVAPI.");
        return;
    }

    if sub == "off" {
        gpu.turn_off();
        println!("[OK] GPU RGB turned OFF.");
        return;
    }

    if sub == "mode" && args.len() >= 2 {
        let mode_name = args[1].to_lowercase();
        let mode_code = match mode_name.as_str() {
            "static" => GPU_MODE_STATIC,
            "breathing" | "pulse" => GPU_MODE_BREATHING,
            "color_cycle" => GPU_MODE_COLOR_CYCLE,
            "flash" | "flashing" => GPU_MODE_FLASHING,
            "double_flash" => GPU_MODE_DUAL_FLASHING,
            "gradient" => GPU_MODE_GRADIENT,
            "wave" => GPU_MODE_WAVE,
            _ => {
                println!("Unknown GPU mode: '{}'.", mode_name);
                return;
            }
        };

        gpu.apply_mode(mode_code, 255, 0, 0, GPU_SPEED_NORMAL, GPU_BRIGHTNESS_MAX);
        println!("[OK] GPU mode set to {}.", mode_name.to_uppercase());
        return;
    }

    if let Ok(((r, g, b), _)) = parse_color(args) {
        gpu.apply_color(r, g, b, GPU_BRIGHTNESS_MAX);
        println!("[OK] GPU set to RGB({}, {}, {})", r, g, b);
        return;
    }

    print_help();
}

fn handle_sync_command(args: &[String]) {
    if args.is_empty() {
        print_help();
        return;
    }

    let sub = args[0].to_lowercase();
    let ((r, g, b), mode) = if sub == "off" {
        ((0, 0, 0), MODE_DISABLE)
    } else if let Ok((rgb, _)) = parse_color(args) {
        (rgb, MODE_STATIC)
    } else {
        print_help();
        return;
    };

    // 1. Update Motherboard
    match MSIMysticLightB550::open() {
        Ok(mut controller) => {
            if let Err(e) = controller.apply_color_to_all(r, g, b, mode) {
                eprintln!("[Warning] Motherboard update error: {}", e);
            }
        }
        Err(e) => eprintln!("[Warning] Could not open motherboard: {}", e),
    }

    // 2. Update GPU
    let mut gpu = GigabyteGPURGB::new(None);
    if gpu.probe_and_connect() {
        if mode == MODE_DISABLE {
            gpu.turn_off();
        } else {
            gpu.apply_color(r, g, b, GPU_BRIGHTNESS_MAX);
        }
    }

    println!("[OK] Synchronized Motherboard + GPU to RGB({}, {}, {})", r, g, b);
}

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        print_help();
        return;
    }

    let cmd = args[1].to_lowercase();

    if cmd == "bass" {
        let sync_gpu = args.iter().any(|a| a == "--gpu" || a == "-g");
        run_cli_bass(sync_gpu);
        return;
    }

    if cmd == "gpu" {
        handle_gpu_command(&args[2..]);
        return;
    }

    if cmd == "sync" {
        handle_sync_command(&args[2..]);
        return;
    }

    let mut controller = match MSIMysticLightB550::open() {
        Ok(c) => c,
        Err(e) => {
            eprintln!("[FAIL] Could not open MSI Motherboard: {}", e);
            return;
        }
    };

    if cmd == "status" {
        match controller.read_packet() {
            Ok(packet) => {
                println!("Current Active Zones (Motherboard):");
                for z in &["j_rgb_1", "j_rainbow_1", "j_rainbow_2", "on_board_led"] {
                    if let Some(info) = controller.get_zone_info(&packet, z) {
                        println!(
                            "  {}: Mode={} RGB={:?} Brightness={}/10",
                            z, info.effect, info.primary_rgb, info.brightness
                        );
                    }
                }
            }
            Err(e) => eprintln!("[FAIL] Error reading packet: {}", e),
        }
        return;
    }

    if cmd == "mode" && args.len() >= 3 {
        let mode_name = args[2].to_lowercase();
        if let Some(mode_code) = get_animation_mode(&mode_name) {
            let (r, g, b) = if args.len() >= 4 {
                parse_color(&args[3..]).map(|(rgb, _)| rgb).unwrap_or((255, 0, 0))
            } else {
                (255, 0, 0)
            };

            if let Err(e) = controller.apply_color_to_all(r, g, b, mode_code) {
                eprintln!("[FAIL] Error setting motherboard mode: {}", e);
            } else {
                println!(
                    "[OK] Switched motherboard mode to {} (R={}, G={}, B={})",
                    mode_name.to_uppercase(),
                    r,
                    g,
                    b
                );
            }
            return;
        } else {
            println!("Unknown animation mode: '{}'.", mode_name);
            return;
        }
    }

    if let Ok(((r, g, b), _)) = parse_color(&args[1..]) {
        let mode = if cmd == "off" { MODE_DISABLE } else { MODE_STATIC };
        if let Err(e) = controller.apply_color_to_all(r, g, b, mode) {
            eprintln!("[FAIL] Error updating motherboard color: {}", e);
        } else {
            println!("[OK] Set motherboard to RGB({}, {}, {})", r, g, b);
        }
        return;
    }

    print_help();
}
