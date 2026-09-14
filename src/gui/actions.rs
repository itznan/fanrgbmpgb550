//! Hardware execution and event action methods for GuiApp.

use std::sync::atomic::Ordering;
use std::sync::Arc;
use std::thread;
use std::time::Duration;

use crate::config::*;
use crate::controller::*;
use crate::gpu::*;
use crate::visualizer::*;

use super::state::GuiApp;

impl GuiApp {
    pub fn reconnect_hardware(&mut self) {
        // Connect Motherboard
        match MSIMysticLightB550::open() {
            Ok(dev) => {
                let fw = dev
                    .read_firmware_versions()
                    .map(|(ap, ld)| format!("APROM: {}, LDROM: {}", ap, ld))
                    .unwrap_or_else(|_| "APROM/LDROM Read Error".to_string());
                self.mobo_fw_info = fw;
                self.mobo_connected = true;
                self.mobo = Some(dev);
            }
            Err(_) => {
                self.mobo_connected = false;
                self.mobo_fw_info = "Device not found / Run as Admin".to_string();
                self.mobo = None;
            }
        }

        // Connect GPU
        let mut gpu_dev = GigabyteGPURGB::new(None);
        if gpu_dev.probe_and_connect() {
            self.gpu_connected = true;
            self.gpu_model_name = if gpu_dev.gpu_name.is_empty() {
                "Gigabyte NVIDIA GPU".to_string()
            } else {
                gpu_dev.gpu_name.clone()
            };
            self.gpu_i2c_addr = gpu_dev.active_address;
            self.gpu = Some(gpu_dev);
        } else {
            self.gpu_connected = false;
            self.gpu_model_name = "Not Detected / NVAPI unavailable".to_string();
            self.gpu_i2c_addr = None;
            self.gpu = None;
        }

        if self.mobo_connected && self.gpu_connected {
            self.set_status("[OK] MSI Motherboard and Gigabyte GPU connected.", false);
        } else if self.mobo_connected {
            self.set_status("[OK] MSI Motherboard connected (GPU offline).", false);
        } else if self.gpu_connected {
            self.set_status("[OK] Gigabyte GPU connected (Motherboard offline).", false);
        } else {
            self.set_status("[Notice] No supported hardware detected. Check USB/NVAPI permissions.", true);
        }
    }

    pub fn apply_motherboard(&mut self) {
        if self.viz_running.load(Ordering::SeqCst) {
            self.set_status("Cannot apply: Audio Visualizer is currently running.", true);
            return;
        }

        if let Some(ref mut dev) = self.mobo {
            let res = dev.apply_zone(
                self.mobo_zone.key(),
                self.mobo_mode,
                self.mobo_color[0],
                self.mobo_color[1],
                self.mobo_color[2],
                self.mobo_speed,
                self.mobo_brightness,
            );
            match res {
                Ok(_) => self.set_status(
                    &format!(
                        "[OK] Motherboard {} set to RGB({}, {}, {}) | Mode 0x{:02X}",
                        self.mobo_zone.label(),
                        self.mobo_color[0],
                        self.mobo_color[1],
                        self.mobo_color[2],
                        self.mobo_mode
                    ),
                    false,
                ),
                Err(e) => self.set_status(&format!("[FAIL] Motherboard error: {}", e), true),
            }
        } else {
            self.set_status("[FAIL] Motherboard controller is disconnected.", true);
        }
    }

    pub fn turn_off_motherboard(&mut self) {
        if let Some(ref mut dev) = self.mobo {
            if dev.apply_color_to_all(0, 0, 0, MODE_DISABLE).is_ok() {
                self.set_status("[OK] Motherboard lighting turned OFF.", false);
            }
        }
    }

    pub fn apply_gpu(&mut self) {
        if self.viz_running.load(Ordering::SeqCst) {
            self.set_status("Cannot apply: Audio Visualizer is currently running.", true);
            return;
        }

        if let Some(ref mut dev) = self.gpu {
            let ok = if self.gpu_mode == GPU_MODE_STATIC {
                dev.apply_color(
                    self.gpu_color[0],
                    self.gpu_color[1],
                    self.gpu_color[2],
                    self.gpu_brightness,
                )
            } else {
                dev.apply_mode(
                    self.gpu_mode,
                    self.gpu_color[0],
                    self.gpu_color[1],
                    self.gpu_color[2],
                    self.gpu_speed,
                    self.gpu_brightness,
                )
            };
            if ok {
                self.set_status(
                    &format!(
                        "[OK] GPU set to RGB({}, {}, {}) | Mode 0x{:02X}",
                        self.gpu_color[0], self.gpu_color[1], self.gpu_color[2], self.gpu_mode
                    ),
                    false,
                );
            } else {
                self.set_status("[FAIL] GPU update failed via NVAPI.", true);
            }
        } else {
            self.set_status("[FAIL] GPU controller is disconnected.", true);
        }
    }

    pub fn turn_off_gpu(&mut self) {
        if let Some(ref mut dev) = self.gpu {
            if dev.turn_off() {
                self.set_status("[OK] GPU lighting turned OFF.", false);
            }
        }
    }

    pub fn sync_all(&mut self, color: [u8; 3]) {
        self.master_color = color;
        self.mobo_color = color;
        self.gpu_color = color;
        self.mobo_mode = MODE_STATIC;
        self.gpu_mode = GPU_MODE_STATIC;

        let mut errors = Vec::new();

        if let Some(ref mut dev) = self.mobo {
            if let Err(e) = dev.apply_color_to_all(color[0], color[1], color[2], MODE_STATIC) {
                errors.push(format!("Motherboard: {}", e));
            }
        }

        if let Some(ref mut dev) = self.gpu {
            if !dev.apply_color(color[0], color[1], color[2], self.gpu_brightness) {
                errors.push("GPU: NVAPI write error".to_string());
            }
        }

        if errors.is_empty() {
            self.set_status(
                &format!(
                    "[OK] Synchronized Motherboard + GPU to RGB({}, {}, {})",
                    color[0], color[1], color[2]
                ),
                false,
            );
        } else {
            self.set_status(&format!("[Warning] Sync partial: {}", errors.join("; ")), true);
        }
    }

    pub fn blackout_all(&mut self) {
        self.turn_off_motherboard();
        self.turn_off_gpu();
        self.set_status("[OK] BLACKOUT: All hardware lighting turned OFF.", false);
    }

    pub fn toggle_visualizer(&mut self) {
        if self.viz_running.load(Ordering::SeqCst) {
            // Stop
            self.viz_running.store(false, Ordering::SeqCst);
            thread::sleep(Duration::from_millis(160));
            self.reconnect_hardware();
            self.set_status("[OK] Sub-bass visualizer stopped. Hardware restored.", false);
        } else {
            // Start: release handles so visualizer thread has full access
            self.mobo = None;
            self.gpu = None;
            self.viz_running.store(true, Ordering::SeqCst);

            let running_clone = Arc::clone(&self.viz_running);
            let metrics_clone = Arc::clone(&self.viz_metrics);

            let device_target = if self.selected_device_idx > 0 && self.selected_device_idx < self.audio_devices.len() {
                Some(self.audio_devices[self.selected_device_idx].clone())
            } else {
                None
            };

            let f_min = self.viz_fmin;
            let f_max = self.viz_fmax;
            let decay = self.viz_decay;
            let min_b = self.viz_min_brightness;
            let sync_gpu = self.viz_sync_gpu;
            let base_col = (self.viz_color[0], self.viz_color[1], self.viz_color[2]);

            thread::spawn(move || {
                start_audio_capture(
                    Arc::clone(&running_clone),
                    Arc::clone(&metrics_clone),
                    f_min,
                    f_max,
                    device_target,
                );
                let cfg = VisualizerConfig {
                    mode: "hybrid".to_string(),
                    sync_gpu,
                    sensitivity: 1.25,
                    threshold: 0.07,
                    gamma: 2.0,
                    decay,
                    min_brightness: min_b,
                    base_color: base_col,
                };
                run_visualizer(running_clone, metrics_clone, cfg);
            });

            self.set_status("[OK] WASAPI Sub-Bass Visualizer active in real-time.", false);
        }
    }
}
