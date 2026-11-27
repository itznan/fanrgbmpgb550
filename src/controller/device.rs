//! MSI MPG B550 GAMING PLUS (MS-7C56) Hardware Controller.
//! Based strictly on OpenRGB's MSIMotherboard185Controller implementation.
//!
//! SAFETY RULES:
//! - Never write save_data = 0x01 (always 0x00 for RAM-only volatile updates).
//! - Feature report size must be exactly 185 bytes (Report ID 0x52 + 184 bytes payload).

use hidapi::{HidApi, HidDevice};
use std::thread;
use std::time::Duration;

use crate::config::{BRIGHTNESS_100, MSI_USB_PID, MSI_USB_VID, SAVE_OFFSET, SPEED_MEDIUM};
use crate::controller::zone::set_all_zones;

pub trait MoboController {
    fn read_packet(&self) -> Result<[u8; 185], String>;
    fn update_hardware(&self, target_packet: &mut [u8; 185]) -> Result<(), String>;
    fn stream_update(
        &mut self,
        target_packet: &mut [u8; 185],
        pause_duration: Duration,
    ) -> Result<(), String>;

    fn apply_color_to_all(&mut self, r: u8, g: u8, b: u8, mode: u8) -> Result<(), String> {
        let mut packet = self.read_packet()?;
        set_all_zones(&mut packet, mode, (r, g, b), SPEED_MEDIUM, BRIGHTNESS_100);
        self.update_hardware(&mut packet)
    }
}

pub struct MSIMysticLightB550 {
    device: HidDevice,
    stream_cached_packet: Option<[u8; 185]>,
}

impl MSIMysticLightB550 {
    pub fn open() -> Result<Self, String> {
        let api = HidApi::new().map_err(|e| format!("Failed to initialize HID API: {}", e))?;
        let device = api.open(MSI_USB_VID, MSI_USB_PID).map_err(|e| {
            format!(
                "Could not open MSI Motherboard (VID 0x{:04X}, PID 0x{:04X}): {}",
                MSI_USB_VID, MSI_USB_PID, e
            )
        })?;

        Ok(Self {
            device,
            stream_cached_packet: None,
        })
    }

    pub fn read_packet(&self) -> Result<[u8; 185], String> {
        let mut buf = [0u8; 185];
        buf[0] = 0x52; // Feature report ID 0x52
        let size = self
            .device
            .get_feature_report(&mut buf)
            .map_err(|e| format!("get_feature_report failed: {}", e))?;

        if size != 185 {
            return Err(format!(
                "Expected 185 bytes from HID report 0x52, got {}",
                size
            ));
        }

        Ok(buf)
    }

    /// Sends the standard two-packet update handshake to the device.
    /// STRICT SAFETY: save_data byte 184 is forced to 0x00 (RAM only).
    pub fn update_hardware(&self, target_packet: &mut [u8; 185]) -> Result<(), String> {
        // ALWAYS enforce volatile RAM-only flag (no flash writes)
        target_packet[SAVE_OFFSET] = 0x00;

        // Step 1: Read current state from board
        let mut old_state = self.read_packet()?;
        old_state[SAVE_OFFSET] = 0x00;

        // Step 2: Send old state first (Mystic Light transition handshake)
        self.device
            .send_feature_report(&old_state)
            .map_err(|e| format!("Failed to send old state feature report: {}", e))?;

        thread::sleep(Duration::from_millis(20));

        // Step 3: Send target state
        self.device
            .send_feature_report(target_packet)
            .map_err(|e| format!("Failed to send target state feature report: {}", e))?;

        Ok(())
    }

    /// High-throughput streaming update for real-time visualizers.
    /// Maintains the two-packet handshake without an expensive read_packet() on every frame.
    /// STRICT SAFETY: save_data byte 184 is forced to 0x00 (RAM only).
    pub fn stream_update(
        &mut self,
        target_packet: &mut [u8; 185],
        pause_duration: Duration,
    ) -> Result<(), String> {
        target_packet[SAVE_OFFSET] = 0x00;

        if self.stream_cached_packet.is_none() {
            let mut init_pkt = self.read_packet()?;
            init_pkt[SAVE_OFFSET] = 0x00;
            self.stream_cached_packet = Some(init_pkt);
        }

        let cached = self.stream_cached_packet.as_ref().unwrap();

        // Step 1: Send cached previous state
        self.device
            .send_feature_report(cached)
            .map_err(|e| format!("Stream update send previous state failed: {}", e))?;

        if !pause_duration.is_zero() {
            thread::sleep(pause_duration);
        }

        // Step 2: Send target state
        self.device
            .send_feature_report(target_packet)
            .map_err(|e| format!("Stream update send target state failed: {}", e))?;

        // Step 3: Cache target state for next frame
        self.stream_cached_packet = Some(*target_packet);

        Ok(())
    }

    /// Applies an RGB color to JRGB1, JRAINBOW1, JRAINBOW2, and ONBOARD LEDs.
    pub fn apply_color_to_all(&mut self, r: u8, g: u8, b: u8, mode: u8) -> Result<(), String> {
        <Self as MoboController>::apply_color_to_all(self, r, g, b, mode)
    }
}

impl MoboController for MSIMysticLightB550 {
    fn read_packet(&self) -> Result<[u8; 185], String> {
        MSIMysticLightB550::read_packet(self)
    }

    fn update_hardware(&self, target_packet: &mut [u8; 185]) -> Result<(), String> {
        MSIMysticLightB550::update_hardware(self, target_packet)
    }

    fn stream_update(
        &mut self,
        target_packet: &mut [u8; 185],
        pause_duration: Duration,
    ) -> Result<(), String> {
        MSIMysticLightB550::stream_update(self, target_packet, pause_duration)
    }
}
