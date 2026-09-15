//! Player and executor for custom JSON lighting effects and profiles.

use std::fs;
use std::path::Path;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread;
use std::time::{Duration, Instant};

use crate::config::{BRIGHTNESS_100, MODE_STATIC, SAVE_OFFSET, SPEED_MEDIUM};
use crate::controller::MSIMysticLightB550;
use crate::gpu::GigabyteGPURGB;
use super::schema::{canonical_zone_name, CustomEffect, ZoneState};

pub fn load_effect_from_file<P: AsRef<Path>>(path: P) -> Result<CustomEffect, String> {
    let content = fs::read_to_string(path.as_ref())
        .map_err(|e| format!("Failed to read effect file '{}': {}", path.as_ref().display(), e))?;

    let effect: CustomEffect = serde_json::from_str(&content)
        .map_err(|e| format!("Failed to parse JSON effect syntax: {}", e))?;

    Ok(effect)
}

pub fn play_custom_effect(effect: &CustomEffect) -> Result<(), String> {
    let effect_name = effect.name.as_deref().unwrap_or("Custom Effect");
    println!("=================================================================");
    println!(" Effect: {}", effect_name);
    if let Some(ref desc) = effect.description {
        println!(" Description: {}", desc);
    }
    if let Some(ref author) = effect.author {
        println!(" Author: {}", author);
    }
    println!("=================================================================");

    // Determine GPU usage
    let has_gpu_in_zones = effect
        .zones
        .as_ref()
        .map(|z| z.keys().any(|k| canonical_zone_name(k) == Some("gpu") || canonical_zone_name(k) == Some("all")))
        .unwrap_or(false);

    let has_gpu_in_keyframes = effect
        .keyframes
        .as_ref()
        .map(|kfs| {
            kfs.iter().any(|kf| {
                kf.color.is_some()
                    || kf
                        .zones
                        .as_ref()
                        .map(|z| z.keys().any(|k| canonical_zone_name(k) == Some("gpu") || canonical_zone_name(k) == Some("all")))
                        .unwrap_or(false)
            })
        })
        .unwrap_or(false);

    let sync_gpu = effect.sync_gpu.unwrap_or(has_gpu_in_zones || has_gpu_in_keyframes);

    // Case 1: Static Profile (no keyframes defined, or empty keyframes)
    let is_static = match &effect.keyframes {
        None => true,
        Some(kfs) if kfs.is_empty() => true,
        _ => false,
    };

    if is_static {
        return apply_static_profile(effect, sync_gpu);
    }

    // Case 2: Animated Keyframes
    run_animated_effect(effect, sync_gpu)
}

fn apply_static_profile(effect: &CustomEffect, sync_gpu: bool) -> Result<(), String> {
    let controller = MSIMysticLightB550::open()
        .map_err(|e| format!("Could not connect to MSI Motherboard: {}", e))?;

    let mut packet = controller.read_packet()
        .map_err(|e| format!("Failed to read motherboard initial packet: {}", e))?;

    let mut base_state = ZoneState::default();

    if let Some(ref zones) = effect.zones {
        for (key, color) in zones {
            match canonical_zone_name(key) {
                Some("all") => base_state = ZoneState::solid(*color),
                Some("j_rgb_1") => base_state.j_rgb_1 = *color,
                Some("j_rainbow_1") => base_state.j_rainbow_1 = *color,
                Some("j_rainbow_2") => base_state.j_rainbow_2 = *color,
                Some("on_board_led") => base_state.on_board_led = *color,
                Some("gpu") => base_state.gpu = *color,
                _ => {}
            }
        }
    }

    // Apply to motherboard
    controller.set_zone_data(&mut packet, "j_rgb_1", MODE_STATIC, base_state.j_rgb_1.r, base_state.j_rgb_1.g, base_state.j_rgb_1.b, SPEED_MEDIUM, BRIGHTNESS_100, 100);
    controller.set_zone_data(&mut packet, "j_rainbow_1", MODE_STATIC, base_state.j_rainbow_1.r, base_state.j_rainbow_1.g, base_state.j_rainbow_1.b, SPEED_MEDIUM, BRIGHTNESS_100, 100);
    controller.set_zone_data(&mut packet, "j_rainbow_2", MODE_STATIC, base_state.j_rainbow_2.r, base_state.j_rainbow_2.g, base_state.j_rainbow_2.b, SPEED_MEDIUM, BRIGHTNESS_100, 100);
    controller.set_zone_data(&mut packet, "on_board_led", MODE_STATIC, base_state.on_board_led.r, base_state.on_board_led.g, base_state.on_board_led.b, SPEED_MEDIUM, BRIGHTNESS_100, 100);
    for i in 1..=6 {
        let zone_key = format!("on_board_led_{}", i);
        controller.set_zone_data(&mut packet, &zone_key, MODE_STATIC, base_state.on_board_led.r, base_state.on_board_led.g, base_state.on_board_led.b, SPEED_MEDIUM, BRIGHTNESS_100, 100);
    }

    controller.update_hardware(&mut packet)?;
    println!("[OK] Applied static lighting profile to Motherboard headers.");

    if sync_gpu {
        let mut gpu = GigabyteGPURGB::new(None);
        if gpu.probe_and_connect() {
            gpu.apply_color(base_state.gpu.r, base_state.gpu.g, base_state.gpu.b, 255);
            println!("[OK] Applied static lighting profile to GPU: RGB({}, {}, {})", base_state.gpu.r, base_state.gpu.g, base_state.gpu.b);
        } else {
            eprintln!("[WARN] GPU requested in profile, but RGB controller not detected.");
        }
    }

    Ok(())
}

fn run_animated_effect(effect: &CustomEffect, sync_gpu: bool) -> Result<(), String> {
    let keyframes = match &effect.keyframes {
        Some(kfs) if !kfs.is_empty() => kfs,
        _ => return Err("No keyframes found in animation effect.".to_string()),
    };

    let fps = effect.fps.clamp(5, 60);
    let frame_interval = Duration::from_micros(1_000_000 / fps as u64);

    let mut controller = MSIMysticLightB550::open()
        .map_err(|e| format!("Could not connect to MSI Motherboard: {}", e))?;

    let mut packet = controller.read_packet()
        .map_err(|e| format!("Failed to read motherboard initial state: {}", e))?;
    packet[SAVE_OFFSET] = 0x00;

    let mut gpu: Option<GigabyteGPURGB> = None;
    if sync_gpu {
        let mut gpu_dev = GigabyteGPURGB::new(None);
        if gpu_dev.probe_and_connect() {
            println!("[OK] GPU synchronized: {}", gpu_dev.gpu_name);
            gpu = Some(gpu_dev);
        } else {
            eprintln!("[WARN] GPU sync enabled in effect, but GPU controller was not detected.");
        }
    }

    let running = Arc::new(AtomicBool::new(true));
    let r_sig = Arc::clone(&running);

    let _ = ctrlc::set_handler(move || {
        r_sig.store(false, Ordering::SeqCst);
    });

    println!("Playing animation at {} FPS. Press Ctrl+C to stop.\n", fps);

    // Seed initial state from current hardware or first keyframe
    let mut current_state = ZoneState::from_keyframe(&keyframes[0], &ZoneState::default());
    let mut iteration = 0u32;

    while running.load(Ordering::SeqCst) {
        iteration += 1;

        for (kf_idx, target_kf) in keyframes.iter().enumerate() {
            if !running.load(Ordering::SeqCst) {
                break;
            }

            let target_state = ZoneState::from_keyframe(target_kf, &current_state);
            let duration_ms = target_kf.duration_ms.max(10);
            let total_steps = ((duration_ms as f32 / 1000.0) * (fps as f32)).round().max(1.0) as u32;

            let is_instant = target_kf.transition.eq_ignore_ascii_case("instant")
                || target_kf.transition.eq_ignore_ascii_case("step")
                || target_kf.transition.eq_ignore_ascii_case("hold");

            let step_start_state = current_state;

            for step in 1..=total_steps {
                let t_frame_start = Instant::now();

                if !running.load(Ordering::SeqCst) {
                    break;
                }

                let t = if is_instant {
                    1.0
                } else {
                    step as f32 / total_steps as f32
                };

                current_state = step_start_state.lerp(&target_state, t);

                // Update motherboard zones in packet
                controller.set_zone_data(&mut packet, "j_rgb_1", MODE_STATIC, current_state.j_rgb_1.r, current_state.j_rgb_1.g, current_state.j_rgb_1.b, 1, 10, 100);
                controller.set_zone_data(&mut packet, "j_rainbow_1", MODE_STATIC, current_state.j_rainbow_1.r, current_state.j_rainbow_1.g, current_state.j_rainbow_1.b, 1, 10, 100);
                controller.set_zone_data(&mut packet, "j_rainbow_2", MODE_STATIC, current_state.j_rainbow_2.r, current_state.j_rainbow_2.g, current_state.j_rainbow_2.b, 1, 10, 100);
                controller.set_zone_data(&mut packet, "on_board_led", MODE_STATIC, current_state.on_board_led.r, current_state.on_board_led.g, current_state.on_board_led.b, 1, 10, 100);
                for i in 1..=6 {
                    let zone_key = format!("on_board_led_{}", i);
                    controller.set_zone_data(&mut packet, &zone_key, MODE_STATIC, current_state.on_board_led.r, current_state.on_board_led.g, current_state.on_board_led.b, 1, 10, 100);
                }

                let _ = controller.stream_update(&mut packet, Duration::from_millis(8));

                if let Some(ref gpu_dev) = gpu {
                    gpu_dev.stream_color_fast(current_state.gpu.r, current_state.gpu.g, current_state.gpu.b);
                }

                // Status ticker
                print!(
                    "\r[Cycle {:3}] Frame {:2}/{} | Mobo: ({:3},{:3},{:3}) | GPU: ({:3},{:3},{:3}) ",
                    iteration,
                    kf_idx + 1,
                    keyframes.len(),
                    current_state.j_rainbow_1.r,
                    current_state.j_rainbow_1.g,
                    current_state.j_rainbow_1.b,
                    current_state.gpu.r,
                    current_state.gpu.g,
                    current_state.gpu.b,
                );
                let _ = std::io::Write::flush(&mut std::io::stdout());

                // Precise frame sleep
                let elapsed = t_frame_start.elapsed();
                if elapsed < frame_interval {
                    thread::sleep(frame_interval - elapsed);
                }
            }
        }

        if !effect.r#loop {
            if let Some(rep) = effect.repeat {
                if iteration >= rep {
                    break;
                }
            } else {
                break;
            }
        } else if let Some(rep) = effect.repeat {
            if iteration >= rep {
                break;
            }
        }
    }

    println!("\n[OK] Animation stopped.");
    Ok(())
}
