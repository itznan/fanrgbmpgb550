//! CLI real-time audio sub-bass visualizer runner.

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};

use crate::visualizer::*;

pub fn run_cli_bass(
    sync_gpu: bool,
    base_color: (u8, u8, u8),
    min_brightness: u8,
    device_name: Option<String>,
    f_min: f32,
    f_max: f32,
    decay: f32,
) {
    println!("=================================================================");
    println!(" Sub-Bass & Kick-Drum Audio Visualizer (Terminal Mode - Rust)");
    println!("=================================================================");
    println!("Audio Source: WASAPI Loopback");
    println!("Sub-Bass Frequency Window: {:.1} Hz - {:.1} Hz", f_min, f_max);
    println!("Reactive Color: RGB({}, {}, {})", base_color.0, base_color.1, base_color.2);
    println!("Idle Minimum Brightness: {}/255 (0 = 0 PWM off)", min_brightness);
    println!("Decay Speed: {:.2}", decay);
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

    start_audio_capture(Arc::clone(&running), Arc::clone(&metrics), f_min, f_max, device_name);

    let config = VisualizerConfig {
        mode: "hybrid".to_string(),
        sync_gpu,
        sensitivity: 1.25,
        threshold: 0.07,
        gamma: 2.0,
        decay,
        min_brightness,
        base_color,
    };

    run_visualizer(running, metrics, config);
}
