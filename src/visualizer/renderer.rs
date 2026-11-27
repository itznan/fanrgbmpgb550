//! RGB render worker for BassVisualizer.

use std::io::{self, Write};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant};

use crate::config::{BRIGHTNESS_100, MODE_DISABLE, MODE_STATIC, SAVE_OFFSET, SPEED_MEDIUM};
use crate::controller::zone::set_all_zones;
use crate::controller::MSIMysticLightB550;
use crate::gpu::{GigabyteGPURGB, GPU_BRIGHTNESS_MAX};
use crate::visualizer::audio::AudioMetrics;

pub struct VisualizerConfig {
    pub mode: String, // "hybrid", "kick", "rumble"
    pub sync_gpu: bool,
    pub sensitivity: f32,
    pub threshold: f32,
    pub gamma: f32,
    pub decay: f32,
    pub min_brightness: u8,
    pub base_color: (u8, u8, u8),
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
            base_color: (255, 0, 0),
        }
    }
}

pub fn run_visualizer(
    running: Arc<AtomicBool>,
    metrics: Arc<Mutex<AudioMetrics>>,
    config: VisualizerConfig,
) -> Result<(), String> {
    let mut controller = match MSIMysticLightB550::open() {
        Ok(c) => c,
        Err(e) => {
            running.store(false, Ordering::SeqCst);
            return Err(format!("Could not open motherboard: {}", e));
        }
    };

    let mut gpu: Option<GigabyteGPURGB> = None;
    if config.sync_gpu {
        let mut gpu_dev = GigabyteGPURGB::new();
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
            running.store(false, Ordering::SeqCst);
            return Err(format!("Failed to read initial packet: {}", e));
        }
    };
    packet[SAVE_OFFSET] = 0x00;

    let mut peak_bass = 20.0f32;
    let mut floor_bass = 1.0f32;
    let mut peak_flux = 8.0f32;
    let mut floor_flux = 0.5f32;

    let mut current_level = 0.0f32;
    let mut frame_count = 0u64;
    let mut last_sent: Option<(u8, u8, u8)> = None;
    let mut stream_err: Option<String> = None;

    let t_start = Instant::now();

    while running.load(Ordering::SeqCst) {
        let t_frame = Instant::now();

        let (raw_bass, raw_flux) = {
            let m = metrics.lock().unwrap_or_else(|e| e.into_inner());
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
        let bass_norm =
            ((raw_bass - floor_bass) / (peak_bass - floor_bass).max(3.0)).clamp(0.0, 1.0);
        let flux_norm =
            ((raw_flux - floor_flux) / (peak_flux - floor_flux).max(2.0)).clamp(0.0, 1.0);

        let mut target_val = match config.mode.as_str() {
            "kick" => flux_norm,
            "rumble" => bass_norm,
            _ => (bass_norm * 0.40) + (flux_norm * 0.70),
        };

        // Noise gate
        if target_val < config.threshold {
            target_val = 0.0;
        } else {
            target_val =
                ((target_val - config.threshold) / (1.0 - config.threshold)).clamp(0.0, 1.0);
        }

        target_val = target_val.powf(config.gamma);

        // Instant sharp attack, rapid drop down
        if target_val > current_level {
            current_level = target_val;
        } else {
            current_level *= config.decay;
            if current_level < 0.015 {
                current_level = 0.0;
            }
        }

        let level_clamped = current_level.clamp(0.0, 1.0);

        // Strict 0 PWM when there is no bass and min_brightness is 0
        let (r, g, b) = if level_clamped <= 0.0 && config.min_brightness == 0 {
            (0, 0, 0)
        } else {
            let min_ratio = (config.min_brightness as f32) / 255.0;
            let scale = min_ratio + (1.0 - min_ratio) * level_clamped;
            (
                ((config.base_color.0 as f32) * scale).clamp(0.0, 255.0) as u8,
                ((config.base_color.1 as f32) * scale).clamp(0.0, 255.0) as u8,
                ((config.base_color.2 as f32) * scale).clamp(0.0, 255.0) as u8,
            )
        };

        // Delta check: only dispatch USB packets when color changes or heartbeat (every 30 frames)
        if last_sent != Some((r, g, b)) || frame_count.is_multiple_of(30) {
            set_all_zones(
                &mut packet,
                MODE_STATIC,
                (r, g, b),
                SPEED_MEDIUM,
                BRIGHTNESS_100,
            );
            if let Err(e) = controller.stream_update(&mut packet, Duration::from_millis(12)) {
                running.store(false, Ordering::SeqCst);
                stream_err = Some(e);
                break;
            }
            if let Some(ref gpu_dev) = gpu {
                gpu_dev.stream_color_fast(r, g, b);
            }
            last_sent = Some((r, g, b));
        }

        frame_count += 1;
        if frame_count.is_multiple_of(3) {
            let elapsed = t_start.elapsed().as_secs_f32();
            let fps = if elapsed > 0.0 {
                frame_count as f32 / elapsed
            } else {
                0.0
            };

            let bar_len = 24;
            let filled = ((bar_len as f32 * level_clamped) as usize).min(bar_len);
            let bar = format!("{}{}", "#".repeat(filled), "-".repeat(bar_len - filled));
            print!(
                "\r[{}] Bass: {:3}% | Color: ({:3}, {:3}, {:3}) | FPS: {:.1} ",
                bar,
                (level_clamped * 100.0) as u32,
                r,
                g,
                b,
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
    if config.min_brightness > 0 {
        let scale = (config.min_brightness as f32) / 255.0;
        let br = ((config.base_color.0 as f32) * scale) as u8;
        let bg = ((config.base_color.1 as f32) * scale) as u8;
        let bb = ((config.base_color.2 as f32) * scale) as u8;
        if let Some(ref mut gpu_dev) = gpu {
            gpu_dev.apply_color(br, bg, bb, GPU_BRIGHTNESS_MAX);
        }
        let _ = controller.apply_color_to_all(br, bg, bb, MODE_STATIC);
    } else {
        if let Some(ref mut gpu_dev) = gpu {
            gpu_dev.turn_off();
        }
        let _ = controller.apply_color_to_all(0, 0, 0, MODE_DISABLE);
    }
    if let Some(e) = stream_err {
        return Err(e);
    }
    println!("Done.");
    Ok(())
}
