//! RGB render worker for BassVisualizer.

use std::io::{self, Write};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant};

use crate::config::{MODE_STATIC, SAVE_OFFSET};
use crate::controller::MSIMysticLightB550;
use crate::gpu::GigabyteGPURGB;
use crate::visualizer::audio::AudioMetrics;

pub struct VisualizerConfig {
    pub mode: String, // "hybrid", "kick", "rumble"
    pub sync_gpu: bool,
    pub sensitivity: f32,
    pub threshold: f32,
    pub gamma: f32,
    pub decay: f32,
    pub min_brightness: u8,
}

impl Default for VisualizerConfig {
    fn default() -> Self {
        Self {
            mode: "hybrid".to_string(),
            sync_gpu: false,
            sensitivity: 1.0,
            threshold: 0.05,
            gamma: 1.8,
            decay: 0.82,
            min_brightness: 0,
        }
    }
}

pub fn run_visualizer(
    running: Arc<AtomicBool>,
    metrics: Arc<Mutex<AudioMetrics>>,
    config: VisualizerConfig,
) {
    let mut controller = match MSIMysticLightB550::open() {
        Ok(c) => c,
        Err(e) => {
            eprintln!("[Visualizer Error]: Could not open motherboard: {}", e);
            running.store(false, Ordering::SeqCst);
            return;
        }
    };

    let mut gpu: Option<GigabyteGPURGB> = None;
    if config.sync_gpu {
        let mut gpu_dev = GigabyteGPURGB::new(None);
        if gpu_dev.probe_and_connect() {
            println!("[OK] GPU Visualizer sync enabled: {}", gpu_dev.gpu_name);
            gpu = Some(gpu_dev);
        } else {
            eprintln!("[Visualizer GPU Warning]: Could not detect GPU RGB controller.");
        }
    }

    let mut packet = match controller.read_packet() {
        Ok(p) => p,
        Err(e) => {
            eprintln!("[Visualizer Error]: Failed to read initial packet: {}", e);
            running.store(false, Ordering::SeqCst);
            return;
        }
    };
    packet[SAVE_OFFSET] = 0x00;

    let mut peak_bass = 20.0f32;
    let mut floor_bass = 1.0f32;
    let mut peak_flux = 8.0f32;
    let mut floor_flux = 0.5f32;

    let mut current_level = 0.0f32;
    let mut frame_count = 0u64;
    let mut last_sent_r = 256i32;

    let t_start = Instant::now();

    while running.load(Ordering::SeqCst) {
        let t_frame = Instant::now();

        let (raw_bass, raw_flux) = {
            let m = metrics.lock().unwrap();
            (m.bass * config.sensitivity, m.flux * config.sensitivity)
        };

        // Dynamic peak / floor follower
        if raw_bass > peak_bass {
            peak_bass = raw_bass;
        } else {
            peak_bass = (peak_bass * 0.985).max(8.0);
        }

        if raw_bass < floor_bass {
            floor_bass = raw_bass;
        } else {
            floor_bass = (floor_bass * 1.01).min(peak_bass * 0.35);
        }

        if raw_flux > peak_flux {
            peak_flux = raw_flux;
        } else {
            peak_flux = (peak_flux * 0.985).max(4.0);
        }

        if raw_flux < floor_flux {
            floor_flux = raw_flux;
        } else {
            floor_flux = (floor_flux * 1.01).min(peak_flux * 0.30);
        }

        // Normalized metrics
        let bass_norm = ((raw_bass - floor_bass) / (peak_bass - floor_bass).max(3.0)).clamp(0.0, 1.0);
        let flux_norm = ((raw_flux - floor_flux) / (peak_flux - floor_flux).max(2.0)).clamp(0.0, 1.0);

        let mut target_val = match config.mode.as_str() {
            "kick" => flux_norm,
            "rumble" => bass_norm,
            _ => (bass_norm * 0.40) + (flux_norm * 0.70),
        };

        // Noise gate
        if target_val < config.threshold {
            target_val = 0.0;
        } else {
            target_val = (target_val - config.threshold) / (1.0 - config.threshold);
        }

        target_val = target_val.powf(config.gamma);

        // Attack / Decay
        if target_val > current_level {
            current_level = current_level * 0.12 + target_val * 0.88;
        } else {
            current_level *= config.decay;
        }

        let level_clamped = current_level.clamp(0.0, 1.0);

        // Pure Red color: G=0, B=0
        let min_r = config.min_brightness as f32;
        let r = (min_r + (255.0 - min_r) * level_clamped) as u8;
        let g = 0u8;
        let b = 0u8;

        // Delta check: only dispatch USB packets when color changes or heartbeat (every 30 frames)
        if r as i32 != last_sent_r || (frame_count % 30 == 0) {
            controller.set_zone_data(&mut packet, "j_rgb_1", MODE_STATIC, r, g, b, 1, 10, 100);
            controller.set_zone_data(&mut packet, "j_rainbow_1", MODE_STATIC, r, g, b, 1, 10, 100);
            controller.set_zone_data(&mut packet, "j_rainbow_2", MODE_STATIC, r, g, b, 1, 10, 100);
            controller.set_zone_data(&mut packet, "on_board_led", MODE_STATIC, r, g, b, 1, 10, 100);
            for i in 1..=6 {
                let zone_key = format!("on_board_led_{}", i);
                controller.set_zone_data(&mut packet, &zone_key, MODE_STATIC, r, g, b, 1, 10, 100);
            }

            let _ = controller.stream_update(&mut packet, Duration::from_millis(12));
            if let Some(ref gpu_dev) = gpu {
                gpu_dev.stream_color_fast(r, 0, 0);
            }
            last_sent_r = r as i32;
        }

        frame_count += 1;
        if frame_count % 3 == 0 {
            let elapsed = t_start.elapsed().as_secs_f32();
            let fps = if elapsed > 0.0 { frame_count as f32 / elapsed } else { 0.0 };

            let bar_len = 24;
            let filled = (bar_len as f32 * level_clamped) as usize;
            let bar = format!("{}{}", "#".repeat(filled), "-".repeat(bar_len - filled));
            print!(
                "\r[{}] Bass: {:3}% | Pure Red: {:3}/255 | FPS: {:.1} ",
                bar,
                (level_clamped * 100.0) as u32,
                r,
                fps
            );
            let _ = io::stdout().flush();
        }

        // Regulate frame rate to ~32 FPS
        let elapsed_frame = t_frame.elapsed();
        let target_frame_dur = Duration::from_millis(28);
        if target_frame_dur > elapsed_frame {
            thread::sleep(target_frame_dur - elapsed_frame);
        }
    }

    println!("\n\nStopping visualizer...");
    if let Some(ref mut gpu_dev) = gpu {
        gpu_dev.apply_color(config.min_brightness, 0, 0, 0);
    }
    let _ = controller.apply_color_to_all(config.min_brightness, 0, 0, MODE_STATIC);
    println!("Done.");
}
