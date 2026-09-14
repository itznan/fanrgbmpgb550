//! CLI command router and sub-command handlers.

use crate::config::*;
use crate::controller::*;
use crate::gpu::*;

use super::args::parse_color;
use super::bass::run_cli_bass;
use super::help::print_help;

pub fn run_cli(args: &[String]) {
    let cmd = args[1].to_lowercase();

    if cmd == "help" || cmd == "--help" || cmd == "-h" {
        print_help();
        return;
    }

    if cmd == "bass" {
        handle_bass(args);
        return;
    }

    if cmd == "gpu" {
        handle_gpu(&args[2..]);
        return;
    }

    if cmd == "sync" {
        handle_sync(&args[2..]);
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
        handle_mobo_status(&controller);
        return;
    }

    if cmd == "mode" && args.len() >= 3 {
        handle_mobo_mode(&mut controller, &args[2..]);
        return;
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

fn handle_mobo_status(controller: &MSIMysticLightB550) {
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
}

fn handle_mobo_mode(controller: &mut MSIMysticLightB550, mode_args: &[String]) {
    let mode_name = mode_args[0].to_lowercase();
    if let Some(mode_code) = get_animation_mode(&mode_name) {
        let (r, g, b) = if mode_args.len() >= 2 {
            parse_color(&mode_args[1..]).map(|(rgb, _)| rgb).unwrap_or((255, 0, 0))
        } else {
            (255, 0, 0)
        };
        if let Err(e) = controller.apply_color_to_all(r, g, b, mode_code) {
            eprintln!("[FAIL] Error setting motherboard mode: {}", e);
        } else {
            println!(
                "[OK] Switched motherboard mode to {} (R={}, G={}, B={})",
                mode_name.to_uppercase(),
                r, g, b
            );
        }
    } else {
        println!("Unknown animation mode: '{}'. Run 'fanrgb help' for options.", mode_name);
    }
}

fn handle_gpu(gpu_args: &[String]) {
    if gpu_args.is_empty() {
        print_help();
        return;
    }
    let sub = gpu_args[0].to_lowercase();
    let mut gpu = GigabyteGPURGB::new(None);

    if sub == "status" {
        if gpu.probe_and_connect() {
            println!("[OK] Gigabyte GPU Detected: {}", gpu.gpu_name);
            println!("[OK] Controller I2C Address: 0x{:02X}", gpu.active_address.unwrap());
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

    if sub == "mode" && gpu_args.len() >= 2 {
        let mode_name = gpu_args[1].to_lowercase();
        let mode_code = match mode_name.as_str() {
            "static" => GPU_MODE_STATIC,
            "breathing" | "pulse" => GPU_MODE_BREATHING,
            "color_cycle" | "rainbow" => GPU_MODE_COLOR_CYCLE,
            "flash" | "flashing" => GPU_MODE_FLASHING,
            "double_flash" => GPU_MODE_DUAL_FLASHING,
            "gradient" => GPU_MODE_GRADIENT,
            "wave" | "rainbow_wave" => GPU_MODE_WAVE,
            _ => {
                println!("Unknown GPU mode: '{}'.", mode_name);
                return;
            }
        };
        gpu.apply_mode(mode_code, 255, 0, 0, GPU_SPEED_NORMAL, GPU_BRIGHTNESS_MAX);
        println!("[OK] GPU mode set to {}.", mode_name.to_uppercase());
        return;
    }

    if let Ok(((r, g, b), _)) = parse_color(gpu_args) {
        gpu.apply_color(r, g, b, GPU_BRIGHTNESS_MAX);
        println!("[OK] GPU set to RGB({}, {}, {})", r, g, b);
        return;
    }

    print_help();
}

fn handle_sync(sync_args: &[String]) {
    if sync_args.is_empty() {
        print_help();
        return;
    }
    let sub = sync_args[0].to_lowercase();
    let ((r, g, b), mode) = if sub == "off" {
        ((0, 0, 0), MODE_DISABLE)
    } else if let Ok((rgb, _)) = parse_color(sync_args) {
        (rgb, MODE_STATIC)
    } else {
        print_help();
        return;
    };

    if let Ok(mut controller) = MSIMysticLightB550::open() {
        let _ = controller.apply_color_to_all(r, g, b, mode);
    }
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

fn handle_bass(args: &[String]) {
    let sync_gpu = args.iter().any(|a| a == "--gpu" || a == "-g");
    let mut min_brightness: u8 = 0;
    let mut device_name: Option<String> = None;
    let mut f_min: f32 = 1.0;
    let mut f_max: f32 = 25.0;
    let mut decay: f32 = 0.82;
    let mut filtered_args = Vec::new();
    let mut i = 2;
    while i < args.len() {
        let arg = &args[i];
        if arg == "--gpu" || arg == "-g" {
            i += 1;
        } else if (arg == "--min" || arg == "-m") && i + 1 < args.len() {
            if let Ok(v) = args[i + 1].parse::<u8>() {
                min_brightness = v;
            }
            i += 2;
        } else if (arg == "--device" || arg == "-d") && i + 1 < args.len() {
            device_name = Some(args[i + 1].clone());
            i += 2;
        } else if (arg == "--fmin" || arg == "--min-freq") && i + 1 < args.len() {
            if let Ok(v) = args[i + 1].parse::<f32>() {
                f_min = v;
            }
            i += 2;
        } else if (arg == "--fmax" || arg == "--max-freq") && i + 1 < args.len() {
            if let Ok(v) = args[i + 1].parse::<f32>() {
                f_max = v;
            }
            i += 2;
        } else if (arg == "--decay") && i + 1 < args.len() {
            if let Ok(v) = args[i + 1].parse::<f32>() {
                decay = v;
            }
            i += 2;
        } else {
            filtered_args.push(args[i].clone());
            i += 1;
        }
    }

    let base_color = if !filtered_args.is_empty() {
        parse_color(&filtered_args).map(|(c, _)| c).unwrap_or((0, 0, 255))
    } else {
        (0, 0, 255)
    };
    run_cli_bass(sync_gpu, base_color, min_brightness, device_name, f_min, f_max, decay);
}
