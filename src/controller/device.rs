//! MSI MPG B550 GAMING PLUS (MS-7C56) Hardware Controller.
//! Based strictly on OpenRGB's MSIMotherboard185Controller implementation.
//!
//! SAFETY RULES:
//! - Never write save_data = 0x01 (always 0x00 for RAM-only volatile updates).
//! - Feature report size must be exactly 185 bytes (Report ID 0x52 + 184 bytes payload).
#![allow(dead_code)]

use std::thread;
use std::time::Duration;
use hidapi::{HidApi, HidDevice};

use crate::config::{
    BRIGHTNESS_100, MSI_USB_PID, MSI_USB_VID, SAVE_OFFSET, SPEED_MEDIUM,
};
use crate::controller::zone::{get_zone_info, set_zone_data, ZoneInfo};

pub struct MSIMysticLightB550 {
    device: HidDevice,
    stream_cached_packet: Option<[u8; 185]>,
}

impl MSIMysticLightB550 {
    pub fn open() -> Result<Self, String> {
        let api = HidApi::new().map_err(|e| format!("Failed to initialize HID API: {}", e))?;
        let device = api
            .open(MSI_USB_VID, MSI_USB_PID)
            .map_err(|e| format!("Could not open MSI Motherboard (VID 0x{:04X}, PID 0x{:04X}): {}", MSI_USB_VID, MSI_USB_PID, e))?;

        Ok(Self {
            device,
            stream_cached_packet: None,
        })
    }

    pub fn read_firmware_versions(&self) -> Result<(String, String), String> {
        // APROM (0xB0)
        let mut req = [0u8; 64];
        req[0] = 0x01;
        req[1] = 0xB0;
        for i in 2..64 {
            req[i] = 0xCC;
        }

        self.device
            .write(&req)
            .map_err(|e| format!("Write APROM request failed: {}", e))?;

        let mut resp_ap = [0u8; 64];
        let n1 = self
            .device
            .read_timeout(&mut resp_ap, 500)
            .map_err(|e| format!("Read APROM failed: {}", e))?;

        let ap_ver = if n1 >= 3 {
            format!("{}.{}", resp_ap[2] >> 4, resp_ap[2] & 0x0F)
        } else {
            "Unknown".to_string()
        };

        // LDROM (0xB6)
        req[1] = 0xB6;
        self.device
            .write(&req)
            .map_err(|e| format!("Write LDROM request failed: {}", e))?;

        let mut resp_ld = [0u8; 64];
        let n2 = self
            .device
            .read_timeout(&mut resp_ld, 500)
            .map_err(|e| format!("Read LDROM failed: {}", e))?;

        let ld_ver = if n2 >= 3 {
            format!("{}.{}", resp_ld[2] >> 4, resp_ld[2] & 0x0F)
        } else {
            "Unknown".to_string()
        };

        Ok((ap_ver, ld_ver))
    }

    pub fn read_packet(&self) -> Result<[u8; 185], String> {
        let mut buf = [0u8; 185];
        buf[0] = 0x52; // Feature report ID 0x52
        let size = self
            .device
            .get_feature_report(&mut buf)
            .map_err(|e| format!("get_feature_report failed: {}", e))?;

        if size != 185 {
            return Err(format!("Expected 185 bytes from HID report 0x52, got {}", size));
        }

        Ok(buf)
    }

    pub fn get_zone_info(&self, packet: &[u8; 185], zone_name: &str) -> Option<ZoneInfo> {
        get_zone_info(packet, zone_name)
    }

    pub fn set_zone_data(
        &self,
        packet: &mut [u8; 185],
        zone_name: &str,
        mode: u8,
        r: u8,
        g: u8,
        b: u8,
        speed: u8,
        brightness: u8,
        led_count: u8,
    ) -> bool {
        set_zone_data(packet, zone_name, mode, r, g, b, speed, brightness, led_count)
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
        let mut packet = self.read_packet()?;

        // Update JRGB1 (12V 4-pin header)
        set_zone_data(&mut packet, "j_rgb_1", mode, r, g, b, SPEED_MEDIUM, BRIGHTNESS_100, 100);

        // Update JRAINBOW1 (5V 3-pin ARGB header)
        set_zone_data(&mut packet, "j_rainbow_1", mode, r, g, b, SPEED_MEDIUM, BRIGHTNESS_100, 100);

        // Update JRAINBOW2 (5V 3-pin ARGB header)
        set_zone_data(&mut packet, "j_rainbow_2", mode, r, g, b, SPEED_MEDIUM, BRIGHTNESS_100, 100);

        // Update Master ONBOARD
        set_zone_data(&mut packet, "on_board_led", mode, r, g, b, SPEED_MEDIUM, BRIGHTNESS_100, 100);

        // Update individual ONBOARD LEDs 1..6
        for i in 1..=6 {
            let zone_key = format!("on_board_led_{}", i);
            set_zone_data(&mut packet, &zone_key, mode, r, g, b, SPEED_MEDIUM, BRIGHTNESS_100, 100);
        }

        self.update_hardware(&mut packet)
    }
}
