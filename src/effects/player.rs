//! Player and executor for custom JSON lighting effects and profiles.

use std::fs;
use std::path::Path;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread;
use std::time::{Duration, Instant};

use super::schema::{canonical_zone_name, CustomEffect, EffectKeyframe, ZoneState};
use crate::config::{BRIGHTNESS_100, MODE_STATIC, SAVE_OFFSET, SPEED_MEDIUM};
use crate::controller::zone::{set_onboard_zones, set_zone_data};
use crate::controller::{MSIMysticLightB550, MoboController};
use crate::gpu::{GigabyteGPURGB, GpuController, GPU_BRIGHTNESS_MAX};

fn seed_animation_state(
    effect: &CustomEffect,
    keyframes: &[EffectKeyframe],
) -> (ZoneState, ZoneState) {
    let mut unscaled = ZoneState::default();
    if let Some(ref zones) = effect.zones {
        unscaled.apply_zones(zones);
    }
    let mut current = unscaled;
    if effect.zones.is_none() && effect.r#loop && keyframes.len() > 1 {
        for kf in keyframes {
            let (next_unscaled, next_scaled) = ZoneState::from_keyframe(kf, &unscaled);
            unscaled = next_unscaled;
            current = next_scaled;
        }
    }
    (unscaled, current)
}

fn write_zone_state(packet: &mut [u8; 185], state: &ZoneState) {
    set_zone_data(
        packet,
        "j_rgb_1",
        MODE_STATIC,
        state.j_rgb_1.to_tuple(),
        SPEED_MEDIUM,
        BRIGHTNESS_100,
    );
    set_zone_data(
        packet,
        "j_rainbow_1",
        MODE_STATIC,
        state.j_rainbow_1.to_tuple(),
        SPEED_MEDIUM,
        BRIGHTNESS_100,
    );
    set_zone_data(
        packet,
        "j_rainbow_2",
        MODE_STATIC,
        state.j_rainbow_2.to_tuple(),
        SPEED_MEDIUM,
        BRIGHTNESS_100,
    );
    set_onboard_zones(
        packet,
        MODE_STATIC,
        state.on_board_led.to_tuple(),
        SPEED_MEDIUM,
        BRIGHTNESS_100,
    );
}

pub fn load_effect_from_file<P: AsRef<Path>>(path: P) -> Result<CustomEffect, String> {
    let content = fs::read_to_string(path.as_ref()).map_err(|e| {
        format!(
            "Failed to read effect file '{}': {}",
            path.as_ref().display(),
            e
        )
    })?;

    let effect: CustomEffect = serde_json::from_str(&content)
        .map_err(|e| format!("Failed to parse JSON effect syntax: {}", e))?;

    Ok(effect)
}

fn should_sync_gpu(effect: &CustomEffect) -> bool {
    let has_gpu_in_zones = effect
        .zones
        .as_ref()
        .map(|z| {
            z.keys().any(|k| {
                canonical_zone_name(k) == Some("gpu") || canonical_zone_name(k) == Some("all")
            })
        })
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
                        .map(|z| {
                            z.keys().any(|k| {
                                canonical_zone_name(k) == Some("gpu")
                                    || canonical_zone_name(k) == Some("all")
                            })
                        })
                        .unwrap_or(false)
            })
        })
        .unwrap_or(false);

    effect
        .sync_gpu
        .unwrap_or(has_gpu_in_zones || has_gpu_in_keyframes)
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

    let sync_gpu = should_sync_gpu(effect);

    // Case 1: Static Profile (no keyframes defined, or empty keyframes)
    let is_static = match &effect.keyframes {
        None => true,
        Some(kfs) if kfs.is_empty() => true,
        _ => false,
    };

    if is_static {
        let controller = MSIMysticLightB550::open()
            .map_err(|e| format!("Could not connect to MSI Motherboard: {}", e))?;
        let mut gpu_dev = GigabyteGPURGB::new();
        let gpu_opt = sync_gpu.then_some(&mut gpu_dev as &mut dyn GpuController);
        return apply_static_profile_with(effect, &controller, gpu_opt);
    }

    // Case 2: Animated Keyframes
    let mut controller = MSIMysticLightB550::open()
        .map_err(|e| format!("Could not connect to MSI Motherboard: {}", e))?;

    let mut gpu_dev = GigabyteGPURGB::new();
    let gpu = if sync_gpu {
        if gpu_dev.probe_and_connect() {
            println!("[OK] GPU synchronized: {}", gpu_dev.gpu_name);
            Some(&mut gpu_dev as &mut dyn GpuController)
        } else {
            eprintln!("[WARN] GPU sync enabled in effect, but GPU controller was not detected.");
            None
        }
    } else {
        None
    };

    let running = Arc::new(AtomicBool::new(true));
    let r_sig = Arc::clone(&running);
    let _ = ctrlc::set_handler(move || {
        r_sig.store(false, Ordering::SeqCst);
    });

    run_animated_effect_with(effect, &mut controller, gpu, running)
}

fn apply_static_profile_with(
    effect: &CustomEffect,
    controller: &dyn MoboController,
    gpu: Option<&mut dyn GpuController>,
) -> Result<(), String> {
    let mut packet = controller
        .read_packet()
        .map_err(|e| format!("Failed to read motherboard initial packet: {}", e))?;

    let mut base_state = ZoneState::default();
    if let Some(ref zones) = effect.zones {
        base_state.apply_zones(zones);
    }

    write_zone_state(&mut packet, &base_state);
    controller.update_hardware(&mut packet)?;
    println!("[OK] Applied static lighting profile to Motherboard headers.");

    if let Some(gpu_dev) = gpu {
        if gpu_dev.probe_and_connect()
            && gpu_dev.apply_color(
                base_state.gpu.r,
                base_state.gpu.g,
                base_state.gpu.b,
                GPU_BRIGHTNESS_MAX,
            )
        {
            println!(
                "[OK] Applied static lighting profile to GPU: RGB({}, {}, {})",
                base_state.gpu.r, base_state.gpu.g, base_state.gpu.b
            );
        } else {
            eprintln!(
                "[WARN] GPU requested in profile, but RGB controller not detected or write failed."
            );
        }
    }

    Ok(())
}

fn run_animated_effect_with(
    effect: &CustomEffect,
    controller: &mut dyn MoboController,
    gpu: Option<&mut dyn GpuController>,
    running: Arc<AtomicBool>,
) -> Result<(), String> {
    let keyframes = match &effect.keyframes {
        Some(kfs) if !kfs.is_empty() => kfs,
        _ => return Err("No keyframes found in animation effect.".to_string()),
    };

    if effect.repeat == Some(0) {
        return Ok(());
    }

    let fps = effect.fps.clamp(5, 60);
    let frame_interval = Duration::from_micros(1_000_000 / fps as u64);

    let mut initial_packet = controller
        .read_packet()
        .map_err(|e| format!("Failed to read motherboard initial state: {}", e))?;
    initial_packet[SAVE_OFFSET] = 0x00;
    let mut packet = initial_packet;

    println!("Playing animation at {} FPS. Press Ctrl+C to stop.\n", fps);

    // Seed starting state from top-level static zones or the end of a looping cycle.
    let (mut unscaled_state, mut current_state) = seed_animation_state(effect, keyframes);

    let mut iteration = 0u32;
    let mut frame_count = 0u64;
    let mut last_sent: Option<ZoneState> = None;

    while running.load(Ordering::SeqCst) {
        iteration += 1;

        for (kf_idx, target_kf) in keyframes.iter().enumerate() {
            if !running.load(Ordering::SeqCst) {
                break;
            }

            let (next_unscaled, target_state) =
                ZoneState::from_keyframe(target_kf, &unscaled_state);
            unscaled_state = next_unscaled;
            let duration_ms = target_kf.duration_ms.max(10);
            let total_steps = ((duration_ms as f32 / 1000.0) * (fps as f32))
                .round()
                .max(1.0) as u32;

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

                if last_sent.as_ref() != Some(&current_state)
                    || frame_count.is_multiple_of(fps as u64)
                {
                    write_zone_state(&mut packet, &current_state);
                    controller.stream_update(&mut packet, Duration::from_millis(8))?;
                    if let Some(ref gpu_dev) = gpu {
                        gpu_dev.stream_color_fast(
                            current_state.gpu.r,
                            current_state.gpu.g,
                            current_state.gpu.b,
                        );
                    }
                    last_sent = Some(current_state);
                }
                frame_count += 1;

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

        if (!effect.r#loop && effect.repeat.is_none())
            || effect.repeat.is_some_and(|rep| iteration >= rep)
        {
            break;
        }
    }

    if !running.load(Ordering::SeqCst) {
        let _ = controller.update_hardware(&mut initial_packet);
    }

    println!("\n[OK] Animation stopped.");
    Ok(())
}

#[cfg(test)]
mod tests {
    use std::cell::RefCell;
    use std::sync::atomic::AtomicBool;
    use std::sync::Arc;
    use std::time::Duration;

    use crate::config::SAVE_OFFSET;
    use crate::controller::{zone::get_zone_info, MoboController};
    use crate::effects::schema::{ColorRGB, CustomEffect};
    use crate::gpu::GpuController;

    use super::{apply_static_profile_with, run_animated_effect_with, seed_animation_state};

    struct FakeMobo {
        initial: [u8; 185],
        restored: RefCell<Option<[u8; 185]>>,
        streamed: Vec<[u8; 185]>,
    }

    impl FakeMobo {
        fn new(seed_byte: u8) -> Self {
            let mut initial = [0u8; 185];
            initial[0] = 0x52;
            initial[32] = seed_byte;
            Self {
                initial,
                restored: RefCell::new(None),
                streamed: Vec::new(),
            }
        }
    }

    impl MoboController for FakeMobo {
        fn read_packet(&self) -> Result<[u8; 185], String> {
            Ok(self.initial)
        }

        fn update_hardware(&self, target_packet: &mut [u8; 185]) -> Result<(), String> {
            target_packet[SAVE_OFFSET] = 0x00;
            *self.restored.borrow_mut() = Some(*target_packet);
            Ok(())
        }

        fn stream_update(
            &mut self,
            target_packet: &mut [u8; 185],
            _pause_duration: Duration,
        ) -> Result<(), String> {
            target_packet[SAVE_OFFSET] = 0x00;
            self.streamed.push(*target_packet);
            Ok(())
        }
    }

    struct FakeGpu {
        colors: RefCell<Vec<(u8, u8, u8)>>,
    }

    impl FakeGpu {
        fn new() -> Self {
            Self {
                colors: RefCell::new(Vec::new()),
            }
        }
    }

    impl GpuController for FakeGpu {
        fn probe_and_connect(&mut self) -> bool {
            true
        }
        fn gpu_name(&self) -> &str {
            "Fake GPU"
        }
        fn active_address(&self) -> Option<u8> {
            Some(0x32)
        }
        fn apply_color(&mut self, r: u8, g: u8, b: u8, _brightness: u8) -> bool {
            self.colors.borrow_mut().push((r, g, b));
            true
        }
        fn stream_color_fast(&self, r: u8, g: u8, b: u8) -> bool {
            self.colors.borrow_mut().push((r, g, b));
            true
        }
        fn apply_mode(&mut self, _m: u8, _r: u8, _g: u8, _b: u8, _s: u8, _br: u8) -> bool {
            true
        }
    }

    #[test]
    fn looping_effect_seeds_cycle_one_from_cycle_end_and_respects_zones_fallback() {
        let fade: CustomEffect =
            serde_json::from_str(include_str!("../../effects/blue_purple_fade.json")).unwrap();
        let (_, start) = seed_animation_state(&fade, fade.keyframes.as_ref().unwrap());
        assert_eq!(start.j_rainbow_1, ColorRGB::new(0, 0, 0));

        let breath: CustomEffect =
            serde_json::from_str(include_str!("../../effects/amber_breath.json")).unwrap();
        let (_, breath_start) = seed_animation_state(&breath, breath.keyframes.as_ref().unwrap());
        assert_eq!(breath_start.j_rainbow_1, ColorRGB::new(28, 0, 0));

        let fallback: CustomEffect = serde_json::from_str(
            r##"{"loop":true,"zones":{"all":"#00FF00"},"keyframes":[{"duration_ms":100,"color":"#FF0000"},{"duration_ms":100,"color":"#0000FF"}]}"##,
        )
        .unwrap();
        let (_, fallback_start) =
            seed_animation_state(&fallback, fallback.keyframes.as_ref().unwrap());
        assert_eq!(fallback_start.j_rainbow_1, ColorRGB::new(0, 255, 0));
    }

    #[test]
    fn static_profile_and_animated_effect_respect_save_offset_and_restore_on_interrupt() {
        let profile: CustomEffect =
            serde_json::from_str(include_str!("../../effects/static_profile.json")).unwrap();
        let mobo = FakeMobo::new(7);
        let mut gpu = FakeGpu::new();
        apply_static_profile_with(&profile, &mobo, Some(&mut gpu)).unwrap();

        let applied = mobo.restored.borrow().unwrap();
        assert_eq!(applied[SAVE_OFFSET], 0x00);
        assert_eq!(
            get_zone_info(&applied, "j_rainbow_1").unwrap().primary_rgb,
            (80, 200, 255)
        );
        assert_eq!(gpu.colors.borrow().as_slice(), &[(80, 200, 255)]);

        // One-shot short animation streams packets with SAVE_OFFSET == 0x00.
        let anim: CustomEffect = serde_json::from_str(
            r##"{"loop":false,"repeat":1,"fps":60,"keyframes":[{"duration_ms":20,"transition":"instant","color":"#FF0000"}]}"##,
        )
        .unwrap();
        let mut mobo_anim = FakeMobo::new(9);
        let mut gpu_anim = FakeGpu::new();
        run_animated_effect_with(
            &anim,
            &mut mobo_anim,
            Some(&mut gpu_anim),
            Arc::new(AtomicBool::new(true)),
        )
        .unwrap();
        assert!(!mobo_anim.streamed.is_empty());
        assert!(mobo_anim.streamed.iter().all(|p| p[SAVE_OFFSET] == 0x00));
        assert!(mobo_anim.restored.borrow().is_none());

        // Pre-interrupted run restores initial packet on exit.
        let mut mobo_cancel = FakeMobo::new(42);
        run_animated_effect_with(
            &anim,
            &mut mobo_cancel,
            None,
            Arc::new(AtomicBool::new(false)),
        )
        .unwrap();
        assert_eq!(mobo_cancel.restored.borrow().unwrap()[32], 42);
    }
}
