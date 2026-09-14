//! GUI application state definitions.

use std::sync::atomic::AtomicBool;
use std::sync::{Arc, Mutex};
use cpal::traits::{DeviceTrait, HostTrait};

use crate::config::*;
use crate::controller::*;
use crate::gpu::*;
use crate::visualizer::*;

use super::zone::MoboZone;

pub struct GuiApp {
    // Hardware handles
    pub mobo: Option<MSIMysticLightB550>,
    pub gpu: Option<GigabyteGPURGB>,

    // Hardware status
    pub mobo_connected: bool,
    pub mobo_fw_info: String,
    pub gpu_connected: bool,
    pub gpu_model_name: String,
    pub gpu_i2c_addr: Option<u8>,

    // Global / Master controls
    pub master_color: [u8; 3],
    pub live_preview: bool,

    // Motherboard controls
    pub mobo_zone: MoboZone,
    pub mobo_mode: u8,
    pub mobo_color: [u8; 3],
    pub mobo_speed: u8,      // 0..2
    pub mobo_brightness: u8, // 0..10

    // GPU controls
    pub gpu_mode: u8,
    pub gpu_color: [u8; 3],
    pub gpu_speed: u8,      // 0..5
    pub gpu_brightness: u8, // 0..99

    // Audio Visualizer controls
    pub viz_running: Arc<AtomicBool>,
    pub viz_metrics: Arc<Mutex<AudioMetrics>>,
    pub viz_color: [u8; 3],
    pub viz_fmin: f32,
    pub viz_fmax: f32,
    pub viz_decay: f32,
    pub viz_min_brightness: u8,
    pub viz_sync_gpu: bool,
    pub audio_devices: Vec<String>,
    pub selected_device_idx: usize,

    // Feedback & UI status
    pub status_message: String,
    pub status_is_error: bool,
}

impl GuiApp {
    pub fn new() -> Self {
        let mut app = Self {
            mobo: None,
            gpu: None,

            mobo_connected: false,
            mobo_fw_info: "Unknown".to_string(),
            gpu_connected: false,
            gpu_model_name: "Not Detected".to_string(),
            gpu_i2c_addr: None,

            master_color: [0, 200, 255], // Default pleasant cyan
            live_preview: false,

            mobo_zone: MoboZone::All,
            mobo_mode: MODE_STATIC,
            mobo_color: [0, 200, 255],
            mobo_speed: SPEED_MEDIUM,
            mobo_brightness: BRIGHTNESS_100,

            gpu_mode: GPU_MODE_STATIC,
            gpu_color: [0, 200, 255],
            gpu_speed: GPU_SPEED_NORMAL,
            gpu_brightness: GPU_BRIGHTNESS_MAX,

            viz_running: Arc::new(AtomicBool::new(false)),
            viz_metrics: Arc::new(Mutex::new(AudioMetrics::default())),
            viz_color: [0, 80, 255],
            viz_fmin: 1.0,
            viz_fmax: 25.0,
            viz_decay: 0.82,
            viz_min_brightness: 0,
            viz_sync_gpu: true,
            audio_devices: Self::list_audio_devices(),
            selected_device_idx: 0,

            status_message: "Ready. Hardware initialized.".to_string(),
            status_is_error: false,
        };

        app.reconnect_hardware();
        app
    }

    pub fn list_audio_devices() -> Vec<String> {
        let host = cpal::default_host();
        let mut devices = vec!["System Default Output".to_string()];
        if let Ok(devs) = host.output_devices() {
            for d in devs {
                if let Ok(name) = d.name() {
                    if !devices.contains(&name) {
                        devices.push(name);
                    }
                }
            }
        }
        devices
    }

    pub fn set_status(&mut self, msg: &str, is_error: bool) {
        self.status_message = msg.to_string();
        self.status_is_error = is_error;
    }
}
