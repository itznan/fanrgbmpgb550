//! Unified CLI & Tauri GUI for MSI MPG B550 Motherboard & Gigabyte GPU RGB.

mod config;
mod controller;
mod gpu;
mod visualizer;

use std::env;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

use serde::{Deserialize, Serialize};
use tauri::State;

use config::*;
use controller::*;
use gpu::*;
use visualizer::*;

pub struct AppState {
    pub vis_running: Arc<AtomicBool>,
    pub vis_metrics: Arc<Mutex<AudioMetrics>>,
}

#[derive(Serialize, Deserialize)]
pub struct ZoneStatus {
    pub name: String,
    pub effect: u8,
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub brightness: u8,
}

#[derive(Serialize, Deserialize)]
pub struct GpuStatus {
    pub detected: bool,
    pub name: String,
    pub address: String,
}

mod commands {
    use super::*;

    fn stop_vis_if_active(state: &State<'_, AppState>) {
        if state.vis_running.load(Ordering::SeqCst) {
            state.vis_running.store(false, Ordering::SeqCst);
            thread::sleep(Duration::from_millis(60));
        }
    }

    #[tauri::command]
    pub fn get_mb_status() -> Result<Vec<ZoneStatus>, String> {
        let controller = MSIMysticLightB550::open()?;
        let packet = controller.read_packet()?;
        let mut zones = Vec::new();
        for z in &["j_rgb_1", "j_rainbow_1", "j_rainbow_2", "on_board_led"] {
            if let Some(info) = controller.get_zone_info(&packet, z) {
                zones.push(ZoneStatus {
                    name: z.to_string(),
                    effect: info.effect,
                    r: info.primary_rgb.0,
                    g: info.primary_rgb.1,
                    b: info.primary_rgb.2,
                    brightness: info.brightness,
                });
            }
        }
        Ok(zones)
    }

    #[tauri::command]
    pub fn get_gpu_status() -> Result<GpuStatus, String> {
        let mut gpu = GigabyteGPURGB::new(None);
        if gpu.probe_and_connect() {
            Ok(GpuStatus {
                detected: true,
                name: gpu.gpu_name.clone(),
                address: format!("0x{:02X}", gpu.active_address.unwrap_or(0)),
            })
        } else {
            Err("GPU not detected".to_string())
        }
    }

    /// Single unified command to instantly set color + effect across Motherboard & GPU
    #[tauri::command]
    pub fn apply_lighting(
        state: State<'_, AppState>,
        r: u8,
        g: u8,
        b: u8,
        mode: String,
    ) -> Result<(), String> {
        stop_vis_if_active(&state);
        let mode_clean = mode.to_lowercase();
        let is_off = (r == 0 && g == 0 && b == 0) || mode_clean == "off" || mode_clean == "disable";

        let mb_mode = if is_off {
            MODE_DISABLE
        } else if mode_clean == "breathing" {
            MODE_BREATHING
        } else if mode_clean == "meteor" {
            MODE_METEOR
        } else if mode_clean == "flashing" {
            MODE_FLASHING
        } else {
            MODE_STATIC
        };

        let mut errors = Vec::new();

        // 1. Motherboard Update
        match MSIMysticLightB550::open() {
            Ok(mut controller) => {
                if let Err(e) = controller.apply_color_to_all(r, g, b, mb_mode) {
                    errors.push(format!("Motherboard error: {}", e));
                }
            }
            Err(e) => errors.push(format!("Motherboard open error: {}", e)),
        }

        // 2. GPU Update
        let mut gpu = GigabyteGPURGB::new(None);
        if gpu.probe_and_connect() {
            if is_off {
                gpu.turn_off();
            } else if mb_mode == MODE_BREATHING || mb_mode == MODE_METEOR {
                gpu.apply_mode(GPU_MODE_BREATHING, r, g, b, GPU_SPEED_NORMAL, GPU_BRIGHTNESS_MAX);
            } else if mb_mode == MODE_FLASHING {
                gpu.apply_mode(GPU_MODE_FLASHING, r, g, b, GPU_SPEED_NORMAL, GPU_BRIGHTNESS_MAX);
            } else {
                gpu.apply_color(r, g, b, GPU_BRIGHTNESS_MAX);
            }
        }

        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors.join(" | "))
        }
    }

    #[tauri::command]
    pub fn set_mb_color(state: State<'_, AppState>, r: u8, g: u8, b: u8) -> Result<(), String> {
        apply_lighting(state, r, g, b, "static".to_string())
    }

    #[tauri::command]
    pub fn set_gpu_color(r: u8, g: u8, b: u8) -> Result<(), String> {
        let mut gpu = GigabyteGPURGB::new(None);
        if gpu.probe_and_connect() {
            if r == 0 && g == 0 && b == 0 {
                gpu.turn_off();
            } else {
                gpu.apply_color(r, g, b, GPU_BRIGHTNESS_MAX);
            }
            Ok(())
        } else {
            Err("GPU not connected via NVAPI".to_string())
        }
    }

    #[tauri::command]
    pub fn set_sync_color(state: State<'_, AppState>, r: u8, g: u8, b: u8) -> Result<(), String> {
        apply_lighting(state, r, g, b, "static".to_string())
    }

    #[tauri::command]
    pub fn set_mb_mode(state: State<'_, AppState>, mode: String, r: u8, g: u8, b: u8) -> Result<(), String> {
        apply_lighting(state, r, g, b, mode)
    }

    #[tauri::command]
    pub fn set_gpu_mode(mode: String) -> Result<(), String> {
        let mode_code = match mode.to_lowercase().as_str() {
            "static" => GPU_MODE_STATIC,
            "breathing" | "pulse" => GPU_MODE_BREATHING,
            "color_cycle" => GPU_MODE_COLOR_CYCLE,
            "flash" | "flashing" => GPU_MODE_FLASHING,
            "gradient" => GPU_MODE_GRADIENT,
            "wave" => GPU_MODE_WAVE,
            _ => return Err(format!("Unknown GPU mode: {}", mode)),
        };

        let mut gpu = GigabyteGPURGB::new(None);
        if gpu.probe_and_connect() {
            gpu.apply_mode(mode_code, 255, 0, 0, GPU_SPEED_NORMAL, GPU_BRIGHTNESS_MAX);
            Ok(())
        } else {
            Err("GPU not connected via NVAPI".to_string())
        }
    }

    #[tauri::command]
    pub fn start_visualizer(state: State<'_, AppState>, sync_gpu: bool) -> Result<(), String> {
        if state.vis_running.load(Ordering::SeqCst) {
            return Ok(());
        }

        state.vis_running.store(true, Ordering::SeqCst);
        let running = Arc::clone(&state.vis_running);
        let metrics = Arc::clone(&state.vis_metrics);

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

        std::thread::spawn(move || {
            run_visualizer(running, metrics, config);
        });

        Ok(())
    }

    #[tauri::command]
    pub fn stop_visualizer(state: State<'_, AppState>) -> Result<(), String> {
        state.vis_running.store(false, Ordering::SeqCst);
        Ok(())
    }

    #[tauri::command]
    pub fn turn_off_all(state: State<'_, AppState>) -> Result<(), String> {
        apply_lighting(state, 0, 0, 0, "off".to_string())
    }
}

// CLI Command Parser
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
MSI Motherboard & Gigabyte GPU RGB Controller CLI & GUI (Rust Edition)

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

=== GPU Commands ===
  fanrgb gpu status                (Checks NVAPI connection and I2C address)
  fanrgb gpu <preset_name>         (e.g. gpu red, gpu blue, gpu off)
  fanrgb gpu mode <mode_name>      (breathing, flash, color_cycle, wave)

=== Synchronized Control (Motherboard + GPU) ===
  fanrgb sync <preset_name>        (e.g. sync red, sync off)
  fanrgb bass [--gpu]              (Pure red bass visualizer synced to Motherboard & GPU)
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

fn run_cli(args: &[String]) {
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
        let gpu_args = &args[2..];
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
        if let Ok(((r, g, b), _)) = parse_color(gpu_args) {
            gpu.apply_color(r, g, b, GPU_BRIGHTNESS_MAX);
            println!("[OK] GPU set to RGB({}, {}, {})", r, g, b);
            return;
        }
        print_help();
        return;
    }

    if cmd == "sync" {
        let sync_args = &args[2..];
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
                    r, g, b
                );
            }
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

fn main() {
    let args: Vec<String> = env::args().collect();
    
    // If command-line arguments are passed, execute in CLI mode directly
    if args.len() > 1 {
        run_cli(&args);
        return;
    }

    // If launched without CLI arguments, run Tauri GUI!
    let app_state = AppState {
        vis_running: Arc::new(AtomicBool::new(false)),
        vis_metrics: Arc::new(Mutex::new(AudioMetrics::default())),
    };

    tauri::Builder::default()
        .manage(app_state)
        .invoke_handler(tauri::generate_handler![
            commands::get_mb_status,
            commands::get_gpu_status,
            commands::apply_lighting,
            commands::set_mb_color,
            commands::set_gpu_color,
            commands::set_sync_color,
            commands::set_mb_mode,
            commands::set_gpu_mode,
            commands::start_visualizer,
            commands::stop_visualizer,
            commands::turn_off_all
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri GUI application");
}
