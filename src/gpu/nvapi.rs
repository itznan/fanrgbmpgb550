//! NVIDIA NVAPI I2C struct definitions.

use std::mem::size_of;

#[repr(C)]
pub struct NvI2cInfoV3 {
    pub version: u32,
    pub display_mask: u32,
    pub is_ddc_port: u8,
    pub i2c_dev_address: u8,
    pub i2c_reg_address: *mut u8,
    pub reg_addr_size: u32,
    pub data: *mut u8,
    pub size: u32,
    pub i2c_speed: u32,
    pub i2c_speed_khz: u32,
    pub port_id: u8,
    pub is_port_id_set: u32,
}

impl Default for NvI2cInfoV3 {
    fn default() -> Self {
        Self {
            version: (3 << 16) | (size_of::<NvI2cInfoV3>() as u32),
            display_mask: 0,
            is_ddc_port: 0,
            i2c_dev_address: 0,
            i2c_reg_address: std::ptr::null_mut(),
            reg_addr_size: 0,
            data: std::ptr::null_mut(),
            size: 0,
            i2c_speed: 0xFFFF,
            i2c_speed_khz: 0,
            port_id: 1,
            is_port_id_set: 1,
        }
    }
}
