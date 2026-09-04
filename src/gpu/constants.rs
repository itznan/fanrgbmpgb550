//! NVAPI function hashes, RGB Fusion 2.0 register addresses, and mode constants.
#![allow(dead_code)]

// NVAPI Hashes
pub const NVAPI_INITIALIZE: u32 = 0x0150E828;
pub const NVAPI_UNLOAD: u32 = 0xD22BDD7E;
pub const NVAPI_ENUM_PHYSICAL_GPUS: u32 = 0xE5AC921F;
pub const NVAPI_GPU_GET_FULL_NAME: u32 = 0xCEEE8E9F;
pub const NVAPI_I2C_WRITE_EX: u32 = 0x283AC65A;
pub const NVAPI_I2C_READ_EX: u32 = 0x4D7B0709;

// RGB Fusion 2.0 Registers
pub const REG_COLOR: u8 = 0x40;
pub const REG_MODE: u8 = 0x88;
pub const REG_COLOR_LEFT_MID: u8 = 0xB0;
pub const REG_COLOR_RIGHT: u8 = 0xB1;

// RGB Fusion 2.0 Modes
pub const GPU_MODE_STATIC: u8 = 0x01;
pub const GPU_MODE_BREATHING: u8 = 0x02;
pub const GPU_MODE_COLOR_CYCLE: u8 = 0x03;
pub const GPU_MODE_FLASHING: u8 = 0x04;
pub const GPU_MODE_GRADIENT: u8 = 0x05;
pub const GPU_MODE_COLOR_SHIFT: u8 = 0x06;
pub const GPU_MODE_WAVE: u8 = 0x07;
pub const GPU_MODE_DUAL_FLASHING: u8 = 0x08;
pub const GPU_MODE_TRICOLOR: u8 = 0x0B;

// Speeds
pub const GPU_SPEED_SLOWEST: u8 = 0x00;
pub const GPU_SPEED_SLOW: u8 = 0x01;
pub const GPU_SPEED_NORMAL: u8 = 0x02;
pub const GPU_SPEED_FAST: u8 = 0x03;
pub const GPU_SPEED_FASTEST: u8 = 0x05;

// Brightness: 0..99
pub const GPU_BRIGHTNESS_MIN: u8 = 0;
pub const GPU_BRIGHTNESS_MAX: u8 = 99;
