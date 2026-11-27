use std::path::PathBuf;

use crate::config::*;
use crate::controller::{zone::get_zone_info, MSIMysticLightB550, MoboController};
use crate::effects::{get_template_json, load_effect_from_file, play_custom_effect};
use crate::gpu::*;

use super::args::parse_color;
use super::bass::run_cli_bass;
use super::help::print_help;

pub fn run_cli(args: &[String]) -> Result<(), String> {
    let cmd = args[1].to_lowercase();

    match cmd.as_str() {
        "help" | "--help" | "-h" => {
            print_help();
            Ok(())
        }
        "--version" | "-v" | "version" => {
            println!("fanrgb {}", env!("CARGO_PKG_VERSION"));
            Ok(())
        }
        "effect" | "play" | "profile" => handle_effect(&args[2..]),
        "bass" => handle_bass(&args[2..]),
        "gpu" => handle_gpu(&args[2..]),
        "sync" => handle_sync(&args[2..]),
        "status" => {
            if args.len() != 2 {
                return Err("Usage: fanrgb status".to_string());
            }
            let controller = MSIMysticLightB550::open()
                .map_err(|e| format!("Could not open MSI Motherboard: {}", e))?;
            handle_mobo_status(&controller)
        }
        "mode" => {
            if args.len() < 3 {
                return Err("Usage: fanrgb mode <animation_mode> [color]".to_string());
            }
            let mode_name = args[2].to_lowercase();
            let mode_code = get_animation_mode(&mode_name).ok_or_else(|| {
                format!(
                    "Unknown animation mode: '{}'. Run 'fanrgb help' for options.",
                    mode_name
                )
            })?;
            let (r, g, b) = if args.len() >= 4 {
                parse_color(&args[3..])?
            } else {
                (255, 0, 0)
            };
            let mut controller = MSIMysticLightB550::open()
                .map_err(|e| format!("Could not open MSI Motherboard: {}", e))?;
            controller
                .apply_color_to_all(r, g, b, mode_code)
                .map_err(|e| format!("Error setting motherboard mode: {}", e))?;
            println!(
                "[OK] Switched motherboard mode to {} (R={}, G={}, B={})",
                mode_name.to_uppercase(),
                r,
                g,
                b
            );
            Ok(())
        }
        _ => {
            let (r, g, b) = match parse_color(&args[1..]) {
                Ok(rgb) => rgb,
                Err(_) => {
                    print_help();
                    return Err(format!(
                        "Unknown command or invalid color: {}",
                        args[1..].join(" ")
                    ));
                }
            };
            let mode = if cmd == "off" {
                MODE_DISABLE
            } else {
                MODE_STATIC
            };
            let mut controller = MSIMysticLightB550::open()
                .map_err(|e| format!("Could not open MSI Motherboard: {}", e))?;
            controller
                .apply_color_to_all(r, g, b, mode)
                .map_err(|e| format!("Error updating motherboard color: {}", e))?;
            println!("[OK] Set motherboard to RGB({}, {}, {})", r, g, b);
            Ok(())
        }
    }
}

fn handle_mobo_status(controller: &dyn MoboController) -> Result<(), String> {
    let packet = controller
        .read_packet()
        .map_err(|e| format!("Error reading packet: {}", e))?;
    println!("Current Active Zones (Motherboard):");
    for z in &["j_rgb_1", "j_rainbow_1", "j_rainbow_2", "on_board_led"] {
        if let Some(info) = get_zone_info(&packet, z) {
            println!(
                "  {}: Mode={} RGB={:?} Brightness={}/10",
                z, info.effect, info.primary_rgb, info.brightness
            );
        }
    }
    Ok(())
}

fn parse_gpu_mode(name: &str) -> Option<u8> {
    match name.to_lowercase().as_str() {
        "static" => Some(GPU_MODE_STATIC),
        "breathing" | "pulse" => Some(GPU_MODE_BREATHING),
        "color_cycle" | "rainbow" => Some(GPU_MODE_COLOR_CYCLE),
        "flash" | "flashing" => Some(GPU_MODE_FLASHING),
        "double_flash" => Some(GPU_MODE_DUAL_FLASHING),
        "gradient" => Some(GPU_MODE_GRADIENT),
        "wave" | "rainbow_wave" => Some(GPU_MODE_WAVE),
        _ => None,
    }
}

fn handle_gpu(gpu_args: &[String]) -> Result<(), String> {
    let mut gpu = GigabyteGPURGB::new();
    handle_gpu_with(gpu_args, &mut gpu)
}

fn handle_gpu_with(gpu_args: &[String], gpu: &mut dyn GpuController) -> Result<(), String> {
    if gpu_args.is_empty() {
        print_help();
        return Err("Missing GPU subcommand (status, off, mode, or color).".to_string());
    }
    let sub = gpu_args[0].to_lowercase();

    enum GpuAction {
        Status,
        Off,
        Mode(String, u8),
        Color(u8, u8, u8),
    }

    let action = match sub.as_str() {
        "status" => {
            if gpu_args.len() != 1 {
                return Err("Usage: fanrgb gpu status".to_string());
            }
            GpuAction::Status
        }
        "off" => {
            if gpu_args.len() != 1 {
                return Err("Usage: fanrgb gpu off".to_string());
            }
            GpuAction::Off
        }
        "mode" => {
            if gpu_args.len() != 2 {
                return Err("Usage: fanrgb gpu mode <mode_name>".to_string());
            }
            let mode_name = gpu_args[1].to_lowercase();
            let mode_code = parse_gpu_mode(&mode_name)
                .ok_or_else(|| format!("Unknown GPU mode: '{}'.", mode_name))?;
            GpuAction::Mode(mode_name, mode_code)
        }
        _ => {
            let (r, g, b) = parse_color(gpu_args)?;
            GpuAction::Color(r, g, b)
        }
    };

    if !gpu.probe_and_connect() {
        return Err("Could not detect Gigabyte GPU RGB controller via NVAPI.".to_string());
    }

    match action {
        GpuAction::Status => {
            println!("[OK] Gigabyte GPU Detected: {}", gpu.gpu_name());
            if let Some(addr) = gpu.active_address() {
                println!("[OK] Controller I2C Address: 0x{:02X}", addr);
            }
        }
        GpuAction::Off => {
            if !gpu.turn_off() {
                return Err("Failed to send GPU turn-off command over I2C.".to_string());
            }
            println!("[OK] GPU RGB turned OFF.");
        }
        GpuAction::Mode(mode_name, mode_code) => {
            if !gpu.apply_mode(mode_code, 255, 0, 0, GPU_SPEED_NORMAL, GPU_BRIGHTNESS_MAX) {
                return Err(format!("Failed to set GPU mode to {}.", mode_name));
            }
            println!("[OK] GPU mode set to {}.", mode_name.to_uppercase());
        }
        GpuAction::Color(r, g, b) => {
            if !gpu.apply_color(r, g, b, GPU_BRIGHTNESS_MAX) {
                return Err("Failed to write GPU color over I2C.".to_string());
            }
            println!("[OK] GPU set to RGB({}, {}, {})", r, g, b);
        }
    }
    Ok(())
}

fn handle_sync(sync_args: &[String]) -> Result<(), String> {
    let mut mobo =
        MSIMysticLightB550::open().map_err(|e| format!("Could not open MSI Motherboard: {}", e));
    let mut gpu = GigabyteGPURGB::new();
    handle_sync_with(
        sync_args,
        mobo.as_mut()
            .map(|m| m as &mut dyn MoboController)
            .map_err(|e| e.clone()),
        &mut gpu,
    )
}

fn handle_sync_with(
    sync_args: &[String],
    mobo: Result<&mut dyn MoboController, String>,
    gpu: &mut dyn GpuController,
) -> Result<(), String> {
    if sync_args.is_empty() {
        print_help();
        return Err("Usage: fanrgb sync <preset|hex|r g b|off>".to_string());
    }
    let ((r, g, b), mode) = if sync_args.len() == 1 && sync_args[0].eq_ignore_ascii_case("off") {
        ((0, 0, 0), MODE_DISABLE)
    } else {
        (parse_color(sync_args)?, MODE_STATIC)
    };

    let mobo_res = mobo.and_then(|c| c.apply_color_to_all(r, g, b, mode));

    let gpu_ok = gpu.probe_and_connect()
        && if mode == MODE_DISABLE {
            gpu.turn_off()
        } else {
            gpu.apply_color(r, g, b, GPU_BRIGHTNESS_MAX)
        };

    match (mobo_res, gpu_ok) {
        (Ok(()), true) => {
            println!(
                "[OK] Synchronized Motherboard + GPU to RGB({}, {}, {})",
                r, g, b
            );
            Ok(())
        }
        (Err(e), false) => Err(format!(
            "Sync failed on Motherboard ({}) and GPU (not detected or I2C write failed).",
            e
        )),
        (Err(e), true) => Err(format!("GPU updated, but Motherboard failed: {}", e)),
        (Ok(()), false) => Err(
            "Motherboard updated, but GPU sync failed (not detected or I2C write failed)."
                .to_string(),
        ),
    }
}

#[derive(Debug, PartialEq)]
struct BassOptions {
    sync_gpu: bool,
    base_color: (u8, u8, u8),
    min_brightness: u8,
    device_name: Option<String>,
    f_min: f32,
    f_max: f32,
    decay: f32,
}

fn next_flag_value<'a>(args: &'a [String], i: &mut usize, flag: &str) -> Result<&'a str, String> {
    if *i + 1 >= args.len() {
        return Err(format!("Missing value for flag '{}'", flag));
    }
    let val = &args[*i + 1];
    *i += 2;
    Ok(val)
}

fn parse_bass_args(args: &[String]) -> Result<BassOptions, String> {
    let mut sync_gpu = false;
    let mut min_brightness: u8 = 0;
    let mut device_name: Option<String> = None;
    let mut f_min: f32 = 1.0;
    let mut f_max: f32 = 25.0;
    let mut decay: f32 = 0.82;
    let mut color_args = Vec::new();
    let mut i = 0;

    while i < args.len() {
        let arg = &args[i];
        match arg.as_str() {
            "--gpu" | "-g" => {
                sync_gpu = true;
                i += 1;
            }
            "--min" | "-m" => {
                let raw = next_flag_value(args, &mut i, arg)?;
                min_brightness = raw
                    .parse::<u8>()
                    .map_err(|_| format!("Invalid --min brightness '{}' (expected 0-255)", raw))?;
            }
            "--device" | "-d" => {
                device_name = Some(next_flag_value(args, &mut i, arg)?.to_string());
            }
            "--fmin" | "--min-freq" => {
                let raw = next_flag_value(args, &mut i, arg)?;
                f_min = raw
                    .parse::<f32>()
                    .map_err(|_| format!("Invalid --fmin frequency '{}'", raw))?;
                if !f_min.is_finite() || f_min < 0.0 {
                    return Err(format!("--fmin must be >= 0.0 Hz, got {}", raw));
                }
            }
            "--fmax" | "--max-freq" => {
                let raw = next_flag_value(args, &mut i, arg)?;
                f_max = raw
                    .parse::<f32>()
                    .map_err(|_| format!("Invalid --fmax frequency '{}'", raw))?;
                if !f_max.is_finite() || f_max <= 0.0 {
                    return Err(format!("--fmax must be > 0.0 Hz, got {}", raw));
                }
            }
            "--decay" => {
                let raw = next_flag_value(args, &mut i, arg)?;
                decay = raw
                    .parse::<f32>()
                    .map_err(|_| format!("Invalid --decay value '{}'", raw))?;
                if !decay.is_finite() || !(0.0..=1.0).contains(&decay) {
                    return Err(format!("--decay must be between 0.0 and 1.0, got {}", raw));
                }
            }
            _ if arg.starts_with('-') => {
                return Err(format!("Unknown bass flag: '{}'", arg));
            }
            _ => {
                color_args.push(arg.clone());
                i += 1;
            }
        }
    }

    if f_min > f_max {
        return Err(format!(
            "Invalid frequency window: --fmin ({}) cannot exceed --fmax ({})",
            f_min, f_max
        ));
    }

    let base_color = if color_args.is_empty() {
        (0, 0, 255)
    } else {
        parse_color(&color_args)?
    };

    Ok(BassOptions {
        sync_gpu,
        base_color,
        min_brightness,
        device_name,
        f_min,
        f_max,
        decay,
    })
}

fn handle_bass(bass_args: &[String]) -> Result<(), String> {
    let opts = parse_bass_args(bass_args)?;
    run_cli_bass(
        opts.sync_gpu,
        opts.base_color,
        opts.min_brightness,
        opts.device_name,
        opts.f_min,
        opts.f_max,
        opts.decay,
    )
}

fn exe_effects_dir() -> Option<PathBuf> {
    std::env::current_exe()
        .ok()
        .and_then(|p| p.parent().map(|dir| dir.join("effects")))
}

fn effects_dir_or_exe() -> PathBuf {
    if std::path::Path::new("effects").is_dir() {
        PathBuf::from("effects")
    } else if let Some(exe_dir) = exe_effects_dir().filter(|d| d.is_dir()) {
        exe_dir
    } else {
        PathBuf::from("effects")
    }
}

fn resolve_effect_path(raw_path: &str) -> PathBuf {
    let mut candidates = vec![
        PathBuf::from(raw_path),
        PathBuf::from(format!("{}.json", raw_path)),
        PathBuf::from("effects").join(raw_path),
        PathBuf::from("effects").join(format!("{}.json", raw_path)),
    ];
    if let Some(exe_dir) = exe_effects_dir() {
        candidates.push(exe_dir.join(raw_path));
        candidates.push(exe_dir.join(format!("{}.json", raw_path)));
    }
    candidates
        .into_iter()
        .find(|p| p.exists())
        .unwrap_or_else(|| PathBuf::from(raw_path))
}

fn handle_effect(args: &[String]) -> Result<(), String> {
    if args.is_empty() {
        println!("Custom JSON Effect Usage:");
        println!("  fanrgb effect <file.json>               Play / apply custom effect or profile");
        println!("  fanrgb effect list                      List bundled / local effect files");
        println!("  fanrgb effect init [template] [file]    Generate a starter JSON template");
        println!("                                          Templates: cyberpunk, police, breath, zones, static");
        return Err("Missing effect subcommand or filename.".to_string());
    }

    let sub = args[0].to_lowercase();

    if sub == "list" {
        let dir = effects_dir_or_exe();
        println!("Available Custom JSON Effects (in {}):", dir.display());
        match std::fs::read_dir(&dir) {
            Ok(entries) => {
                let mut paths: Vec<PathBuf> = entries
                    .flatten()
                    .map(|e| e.path())
                    .filter(|p| p.extension().is_some_and(|e| e == "json"))
                    .collect();
                paths.sort();
                for path in paths {
                    let file_name = path
                        .file_name()
                        .map(|n| n.to_string_lossy().into_owned())
                        .unwrap_or_default();
                    if let Ok(effect) = load_effect_from_file(&path) {
                        let fallback = path
                            .file_stem()
                            .map(|s| s.to_string_lossy().into_owned())
                            .unwrap_or_default();
                        let name = effect.name.unwrap_or(fallback);
                        let desc = effect.description.unwrap_or_default();
                        println!("  • {:<20} - {} ({})", file_name, name, desc);
                    } else {
                        println!("  • {}", file_name);
                    }
                }
            }
            Err(_) => println!("  (No effects directory found)"),
        }
        return Ok(());
    }

    if sub == "init" || sub == "template" {
        if args.len() > 3 {
            return Err("Usage: fanrgb effect init [template] [file]".to_string());
        }
        let template_type = if args.len() >= 2 {
            &args[1]
        } else {
            "cyberpunk"
        };
        let target_filename = if args.len() >= 3 {
            args[2].clone()
        } else {
            format!("{}.json", template_type)
        };

        let json_content = get_template_json(template_type).ok_or_else(|| {
            format!(
                "Unknown template type '{}'. Choose from: cyberpunk, police, breath, zones, static",
                template_type
            )
        })?;
        std::fs::write(&target_filename, json_content)
            .map_err(|e| format!("Could not write template file '{}': {}", target_filename, e))?;
        println!("[OK] Created custom effect template: {}", target_filename);
        return Ok(());
    }

    if args.len() > 1 {
        return Err("Usage: fanrgb effect <file.json>".to_string());
    }

    let path = resolve_effect_path(&args[0]);
    let effect = load_effect_from_file(&path)?;
    play_custom_effect(&effect)
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use crate::config::SAVE_OFFSET;
    use crate::controller::MoboController;
    use crate::gpu::GpuController;

    use super::{
        handle_gpu_with, handle_mobo_status, handle_sync_with, parse_bass_args,
        resolve_effect_path, BassOptions,
    };

    fn s(items: &[&str]) -> Vec<String> {
        items.iter().map(|x| x.to_string()).collect()
    }

    struct MockMobo {
        packet: [u8; 185],
        updates: Vec<[u8; 185]>,
    }

    impl MockMobo {
        fn new() -> Self {
            let mut packet = [0u8; 185];
            packet[0] = 0x52;
            Self {
                packet,
                updates: Vec::new(),
            }
        }
    }

    impl MoboController for MockMobo {
        fn read_packet(&self) -> Result<[u8; 185], String> {
            Ok(self.packet)
        }

        fn update_hardware(&self, target_packet: &mut [u8; 185]) -> Result<(), String> {
            target_packet[SAVE_OFFSET] = 0x00;
            Ok(())
        }

        fn stream_update(
            &mut self,
            target_packet: &mut [u8; 185],
            _pause_duration: Duration,
        ) -> Result<(), String> {
            target_packet[SAVE_OFFSET] = 0x00;
            self.updates.push(*target_packet);
            Ok(())
        }
    }

    struct MockGpu {
        connected: bool,
        write_ok: bool,
        last_color: Option<(u8, u8, u8, u8)>,
        last_mode: Option<u8>,
    }

    impl MockGpu {
        fn new(connected: bool, write_ok: bool) -> Self {
            Self {
                connected,
                write_ok,
                last_color: None,
                last_mode: None,
            }
        }
    }

    impl GpuController for MockGpu {
        fn probe_and_connect(&mut self) -> bool {
            self.connected
        }
        fn gpu_name(&self) -> &str {
            "Mock RTX 3060 Ti"
        }
        fn active_address(&self) -> Option<u8> {
            self.connected.then_some(0x32)
        }
        fn apply_color(&mut self, r: u8, g: u8, b: u8, brightness: u8) -> bool {
            self.last_color = Some((r, g, b, brightness));
            self.write_ok
        }
        fn stream_color_fast(&self, _r: u8, _g: u8, _b: u8) -> bool {
            self.write_ok
        }
        fn apply_mode(&mut self, mode: u8, _r: u8, _g: u8, _b: u8, _s: u8, _br: u8) -> bool {
            self.last_mode = Some(mode);
            self.write_ok
        }
    }

    #[test]
    fn parses_bass_flags_and_color_in_any_order() {
        let opts = parse_bass_args(&s(&[
            "--gpu", "dark", "blue", "--min", "15", "--fmin", "20", "--fmax", "80", "--decay",
            "0.9",
        ]))
        .unwrap();
        assert_eq!(
            opts,
            BassOptions {
                sync_gpu: true,
                base_color: (0, 20, 180),
                min_brightness: 15,
                device_name: None,
                f_min: 20.0,
                f_max: 80.0,
                decay: 0.9,
            }
        );
    }

    #[test]
    fn rejects_invalid_bass_flags_and_ranges() {
        assert!(parse_bass_args(&s(&["--unknown"])).is_err());
        assert!(parse_bass_args(&s(&["--min"])).is_err());
        assert!(parse_bass_args(&s(&["--min", "300"])).is_err());
        assert!(parse_bass_args(&s(&["--fmin", "80", "--fmax", "20"])).is_err());
        assert!(parse_bass_args(&s(&["--decay", "1.5"])).is_err());
        assert!(parse_bass_args(&s(&["notacolor"])).is_err());
    }

    #[test]
    fn sync_and_gpu_handlers_report_partial_and_i2c_failures() {
        let mut mobo = MockMobo::new();
        let mut gpu_ok = MockGpu::new(true, true);
        assert!(handle_mobo_status(&mobo).is_ok());
        assert!(handle_sync_with(&s(&["purple"]), Ok(&mut mobo), &mut gpu_ok).is_ok());
        assert_eq!(gpu_ok.last_color, Some((160, 32, 240, 99)));

        let mut gpu_missing = MockGpu::new(false, true);
        assert!(handle_sync_with(&s(&["purple"]), Ok(&mut mobo), &mut gpu_missing).is_err());
        assert!(handle_sync_with(&s(&["purple"]), Err("no mobo".into()), &mut gpu_ok).is_err());

        let mut gpu_write_fail = MockGpu::new(true, false);
        assert!(handle_gpu_with(&s(&["status"]), &mut gpu_ok).is_ok());
        assert!(handle_gpu_with(&s(&["mode", "wave"]), &mut gpu_ok).is_ok());
        assert!(handle_gpu_with(&s(&["red"]), &mut gpu_write_fail).is_err());
        assert!(handle_gpu_with(&s(&["mode", "bogus"]), &mut gpu_ok).is_err());
    }

    #[test]
    fn resolves_effect_paths_in_effects_dir() {
        let path = resolve_effect_path("cyberpunk");
        assert!(path.exists());
    }
}
