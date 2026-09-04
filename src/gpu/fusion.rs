//! Gigabyte GPU RGB Fusion 2.0 Controller via NVIDIA NVAPI.
#![allow(dead_code)]

use std::ffi::{c_char, c_void, CStr};
use std::ptr;
use libloading::Library;

use crate::gpu::constants::*;
use crate::gpu::nvapi::NvI2cInfoV3;

type QueryInterfaceFn = unsafe extern "C" fn(u32) -> *mut c_void;
type NvInitFn = unsafe extern "C" fn() -> i32;
type NvEnumGpusFn = unsafe extern "C" fn(*mut [*mut c_void; 64], *mut i32) -> i32;
type NvGetFullNameFn = unsafe extern "C" fn(*mut c_void, *mut c_char) -> i32;
type NvI2cWriteExFn = unsafe extern "C" fn(*mut c_void, *mut NvI2cInfoV3, *mut u32) -> i32;
type NvI2cReadExFn = unsafe extern "C" fn(*mut c_void, *mut NvI2cInfoV3, *mut u32) -> i32;

pub struct GigabyteGPURGB {
    pub active_address: Option<u8>,
    pub gpu_name: String,
    target_address: Option<u8>,
    gpu_handle: *mut c_void,
    i2c_write_fn: Option<NvI2cWriteExFn>,
    i2c_read_fn: Option<NvI2cReadExFn>,
    _lib: Option<Library>,
    initialized: bool,
}

unsafe impl Send for GigabyteGPURGB {}
unsafe impl Sync for GigabyteGPURGB {}

impl GigabyteGPURGB {
    pub const CANDIDATE_ADDRESSES: &'static [u8] = &[0x32, 0x62, 0x71, 0x70, 0x64, 0x65];

    pub fn new(target_address: Option<u8>) -> Self {
        Self {
            active_address: None,
            gpu_name: String::new(),
            target_address,
            gpu_handle: ptr::null_mut(),
            i2c_write_fn: None,
            i2c_read_fn: None,
            _lib: None,
            initialized: false,
        }
    }

    fn init_nvapi(&mut self) -> Result<(), String> {
        if self.initialized {
            return Ok(());
        }

        let lib = unsafe {
            Library::new("nvapi64.dll").map_err(|e| format!("Failed to load nvapi64.dll: {}", e))?
        };

        unsafe {
            let qi_symbol: libloading::Symbol<QueryInterfaceFn> = lib
                .get(b"nvapi_QueryInterface\0")
                .map_err(|e| format!("QueryInterface not found in nvapi64.dll: {}", e))?;

            let qi = *qi_symbol;

            let get_proc = |hash: u32| -> *mut c_void { qi(hash) };

            let init_ptr = get_proc(NVAPI_INITIALIZE);
            if init_ptr.is_null() {
                return Err("NvAPI_Initialize interface pointer is null.".to_string());
            }
            let fn_init: NvInitFn = std::mem::transmute(init_ptr);
            if fn_init() != 0 {
                return Err("NvAPI_Initialize failed.".to_string());
            }

            let enum_ptr = get_proc(NVAPI_ENUM_PHYSICAL_GPUS);
            if enum_ptr.is_null() {
                return Err("NvAPI_EnumPhysicalGPUs pointer is null.".to_string());
            }
            let fn_enum: NvEnumGpusFn = std::mem::transmute(enum_ptr);

            let mut handles: [*mut c_void; 64] = [ptr::null_mut(); 64];
            let mut count: i32 = 0;
            if fn_enum(&mut handles, &mut count) != 0 || count == 0 {
                return Err("No physical NVIDIA GPUs detected via NVAPI.".to_string());
            }

            self.gpu_handle = handles[0];

            let name_ptr = get_proc(NVAPI_GPU_GET_FULL_NAME);
            if !name_ptr.is_null() {
                let fn_name: NvGetFullNameFn = std::mem::transmute(name_ptr);
                let mut buf = [0i8; 128];
                if fn_name(self.gpu_handle, buf.as_mut_ptr()) == 0 {
                    let cstr = CStr::from_ptr(buf.as_ptr());
                    self.gpu_name = cstr.to_string_lossy().to_string();
                }
            }

            let write_ptr = get_proc(NVAPI_I2C_WRITE_EX);
            let read_ptr = get_proc(NVAPI_I2C_READ_EX);

            if write_ptr.is_null() || read_ptr.is_null() {
                return Err("NvAPI I2C read/write functions unavailable.".to_string());
            }

            self.i2c_write_fn = Some(std::mem::transmute(write_ptr));
            self.i2c_read_fn = Some(std::mem::transmute(read_ptr));
            self._lib = Some(lib);
            self.initialized = true;
        }

        Ok(())
    }

    fn i2c_write(&self, address: u8, data: &[u8]) -> bool {
        if !self.initialized || self.i2c_write_fn.is_none() {
            return false;
        }

        let write_fn = self.i2c_write_fn.unwrap();
        let mut data_vec = data.to_vec();

        let mut info = NvI2cInfoV3::default();
        info.i2c_dev_address = address << 1;
        info.data = data_vec.as_mut_ptr();
        info.size = data_vec.len() as u32;

        let mut unknown: u32 = 0;
        let res = unsafe { write_fn(self.gpu_handle, &mut info, &mut unknown) };
        res == 0
    }

    fn i2c_read(&self, address: u8, length: usize) -> Option<Vec<u8>> {
        if !self.initialized || self.i2c_read_fn.is_none() {
            return None;
        }

        let read_fn = self.i2c_read_fn.unwrap();
        let mut buf = vec![0u8; length];

        let mut info = NvI2cInfoV3::default();
        info.i2c_dev_address = address << 1;
        info.data = buf.as_mut_ptr();
        info.size = length as u32;

        let mut unknown: u32 = 0;
        let res = unsafe { read_fn(self.gpu_handle, &mut info, &mut unknown) };
        if res == 0 {
            Some(buf)
        } else {
            None
        }
    }

    /// Safely scans candidate I2C addresses and validates the 0xAB signature.
    pub fn probe_and_connect(&mut self) -> bool {
        if self.init_nvapi().is_err() {
            return false;
        }

        let addresses = if let Some(addr) = self.target_address {
            vec![addr]
        } else {
            Self::CANDIDATE_ADDRESSES.to_vec()
        };

        for addr in addresses {
            // Send OpenRGB 0xAB probe packet
            let probe_pkt = [0xAB, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00];
            if self.i2c_write(addr, &probe_pkt) {
                if let Some(resp) = self.i2c_read(addr, 4) {
                    if !resp.is_empty() && resp[0] == 0xAB {
                        self.active_address = Some(addr);
                        return true;
                    }
                }
            }
        }

        false
    }

    /// Sets a static RGB color across the GPU logo/shroud.
    pub fn apply_color(&mut self, r: u8, g: u8, b: u8, brightness: u8) -> bool {
        if self.active_address.is_none() && !self.probe_and_connect() {
            return false;
        }

        let addr = self.active_address.unwrap();
        let b_clamped = brightness.min(GPU_BRIGHTNESS_MAX);

        let mode_pkt = [REG_MODE, GPU_MODE_STATIC, GPU_SPEED_NORMAL, b_clamped, 0x00, 0x01, 0x00, 0x00];
        self.i2c_write(addr, &mode_pkt);

        let color_pkt = [REG_COLOR_LEFT_MID, GPU_MODE_STATIC, r, g, b, r, g, b];
        self.i2c_write(addr, &color_pkt)
    }

    /// High-frequency color update for audio visualizers (skips mode packet).
    pub fn stream_color_fast(&self, r: u8, g: u8, b: u8) -> bool {
        if let Some(addr) = self.active_address {
            let color_pkt = [REG_COLOR_LEFT_MID, GPU_MODE_STATIC, r, g, b, r, g, b];
            self.i2c_write(addr, &color_pkt)
        } else {
            false
        }
    }

    /// Applies a hardware animation effect mode.
    pub fn apply_mode(
        &mut self,
        mode: u8,
        r: u8,
        g: u8,
        b: u8,
        speed: u8,
        brightness: u8,
    ) -> bool {
        if self.active_address.is_none() && !self.probe_and_connect() {
            return false;
        }

        let addr = self.active_address.unwrap();
        let b_clamped = brightness.min(GPU_BRIGHTNESS_MAX);
        let s_clamped = speed.min(GPU_SPEED_FASTEST);

        let mystery_flag = if mode == GPU_MODE_GRADIENT || mode == GPU_MODE_TRICOLOR {
            0x08
        } else {
            0x00
        };

        let mode_pkt = [REG_MODE, mode, s_clamped, b_clamped, mystery_flag, 0x01, 0x00, 0x00];
        self.i2c_write(addr, &mode_pkt);

        if mode != GPU_MODE_COLOR_CYCLE && mode != GPU_MODE_WAVE {
            let color_pkt = [REG_COLOR_LEFT_MID, mode, r, g, b, r, g, b];
            self.i2c_write(addr, &color_pkt);
        }

        true
    }

    pub fn turn_off(&mut self) -> bool {
        self.apply_color(0, 0, 0, 0)
    }

    pub fn save_to_eeprom(&self) -> bool {
        if let Some(addr) = self.active_address {
            let save_pkt = [0xAA, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00];
            self.i2c_write(addr, &save_pkt)
        } else {
            false
        }
    }
}
