//! Gigabyte GPU RGB Fusion 2.0 Controller via NVIDIA NVAPI.

#[cfg(target_os = "windows")]
use libloading::Library;
use std::ffi::c_void;
#[cfg(target_os = "windows")]
use std::ffi::{c_char, CStr};
use std::ptr;

use crate::gpu::constants::*;
#[cfg(target_os = "windows")]
use crate::gpu::nvapi::NvI2cInfoV3;

#[cfg(target_os = "windows")]
type QueryInterfaceFn = unsafe extern "C" fn(u32) -> *mut c_void;
#[cfg(target_os = "windows")]
type NvInitFn = unsafe extern "C" fn() -> i32;
#[cfg(target_os = "windows")]
type NvUnloadFn = unsafe extern "C" fn() -> i32;
#[cfg(target_os = "windows")]
type NvEnumGpusFn = unsafe extern "C" fn(*mut [*mut c_void; 64], *mut i32) -> i32;
#[cfg(target_os = "windows")]
type NvGetFullNameFn = unsafe extern "C" fn(*mut c_void, *mut c_char) -> i32;
#[cfg(target_os = "windows")]
type NvI2cWriteExFn = unsafe extern "C" fn(*mut c_void, *mut NvI2cInfoV3, *mut u32) -> i32;
#[cfg(target_os = "windows")]
type NvI2cReadExFn = unsafe extern "C" fn(*mut c_void, *mut NvI2cInfoV3, *mut u32) -> i32;

pub trait GpuController {
    fn probe_and_connect(&mut self) -> bool;
    fn gpu_name(&self) -> &str;
    fn active_address(&self) -> Option<u8>;
    fn apply_color(&mut self, r: u8, g: u8, b: u8, brightness: u8) -> bool;
    fn stream_color_fast(&self, r: u8, g: u8, b: u8) -> bool;
    fn apply_mode(&mut self, mode: u8, r: u8, g: u8, b: u8, speed: u8, brightness: u8) -> bool;
    fn turn_off(&mut self) -> bool {
        self.apply_color(0, 0, 0, 0)
    }
}

pub struct GigabyteGPURGB {
    pub active_address: Option<u8>,
    pub gpu_name: String,
    gpu_handle: *mut c_void,
    gpu_handles: [*mut c_void; 64],
    gpu_count: usize,
    #[cfg(target_os = "windows")]
    get_name_fn: Option<NvGetFullNameFn>,
    #[cfg(target_os = "windows")]
    i2c_write_fn: Option<NvI2cWriteExFn>,
    #[cfg(target_os = "windows")]
    i2c_read_fn: Option<NvI2cReadExFn>,
    #[cfg(target_os = "windows")]
    unload_fn: Option<NvUnloadFn>,
    #[cfg(target_os = "windows")]
    _lib: Option<Library>,
    initialized: bool,
}

unsafe impl Send for GigabyteGPURGB {}
unsafe impl Sync for GigabyteGPURGB {}

impl Drop for GigabyteGPURGB {
    fn drop(&mut self) {
        #[cfg(target_os = "windows")]
        if self.initialized {
            if let Some(fn_unload) = self.unload_fn {
                unsafe {
                    fn_unload();
                }
            }
            self.initialized = false;
        }
    }
}

impl GigabyteGPURGB {
    pub const CANDIDATE_ADDRESSES: &'static [u8] = &[0x32, 0x62, 0x71, 0x70, 0x64, 0x65];

    pub fn new() -> Self {
        Self {
            active_address: None,
            gpu_name: String::new(),
            gpu_handle: ptr::null_mut(),
            gpu_handles: [ptr::null_mut(); 64],
            gpu_count: 0,
            #[cfg(target_os = "windows")]
            get_name_fn: None,
            #[cfg(target_os = "windows")]
            i2c_write_fn: None,
            #[cfg(target_os = "windows")]
            i2c_read_fn: None,
            #[cfg(target_os = "windows")]
            unload_fn: None,
            #[cfg(target_os = "windows")]
            _lib: None,
            initialized: false,
        }
    }

    #[cfg(target_os = "windows")]
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
            let fn_init: NvInitFn = std::mem::transmute::<*mut c_void, NvInitFn>(init_ptr);
            if fn_init() != 0 {
                return Err("NvAPI_Initialize failed.".to_string());
            }

            let unload_ptr = get_proc(NVAPI_UNLOAD);
            let unload_fn = if !unload_ptr.is_null() {
                Some(std::mem::transmute::<*mut c_void, NvUnloadFn>(unload_ptr))
            } else {
                None
            };
            // Store library and unload handle immediately so Drop unloads NVAPI even on early error.
            self.unload_fn = unload_fn;
            self._lib = Some(lib);
            self.initialized = true;

            let enum_ptr = get_proc(NVAPI_ENUM_PHYSICAL_GPUS);
            if enum_ptr.is_null() {
                return Err("NvAPI_EnumPhysicalGPUs pointer is null.".to_string());
            }
            let fn_enum: NvEnumGpusFn = std::mem::transmute::<*mut c_void, NvEnumGpusFn>(enum_ptr);

            let mut handles: [*mut c_void; 64] = [ptr::null_mut(); 64];
            let mut count: i32 = 0;
            if fn_enum(&mut handles, &mut count) != 0 || count <= 0 {
                return Err("No physical NVIDIA GPUs detected via NVAPI.".to_string());
            }

            self.gpu_handles = handles;
            self.gpu_count = (count as usize).min(64);
            self.gpu_handle = handles[0];

            let name_ptr = get_proc(NVAPI_GPU_GET_FULL_NAME);
            if !name_ptr.is_null() {
                self.get_name_fn = Some(std::mem::transmute::<*mut c_void, NvGetFullNameFn>(
                    name_ptr,
                ));
            }

            let write_ptr = get_proc(NVAPI_I2C_WRITE_EX);
            let read_ptr = get_proc(NVAPI_I2C_READ_EX);

            if write_ptr.is_null() || read_ptr.is_null() {
                return Err("NvAPI I2C read/write functions unavailable.".to_string());
            }

            self.i2c_write_fn = Some(std::mem::transmute::<*mut c_void, NvI2cWriteExFn>(
                write_ptr,
            ));
            self.i2c_read_fn = Some(std::mem::transmute::<*mut c_void, NvI2cReadExFn>(read_ptr));
        }

        Ok(())
    }

    #[cfg(target_os = "windows")]
    fn query_gpu_name(&self, handle: *mut c_void) -> String {
        if let Some(fn_name) = self.get_name_fn {
            let mut buf = [0i8; 128];
            if unsafe { fn_name(handle, buf.as_mut_ptr()) } == 0 {
                return unsafe { CStr::from_ptr(buf.as_ptr()) }
                    .to_string_lossy()
                    .to_string();
            }
        }
        String::new()
    }

    #[cfg(not(target_os = "windows"))]
    fn query_gpu_name(&self, _handle: *mut c_void) -> String {
        String::new()
    }

    #[cfg(not(target_os = "windows"))]
    fn init_nvapi(&mut self) -> Result<(), String> {
        Err("NVIDIA NVAPI GPU RGB is currently only supported on Windows".to_string())
    }

    #[cfg(target_os = "windows")]
    fn i2c_write(&self, address: u8, data: &[u8]) -> bool {
        let Some(write_fn) = self.i2c_write_fn else {
            return false;
        };
        if !self.initialized {
            return false;
        }

        let mut stack_buf = [0u8; 64];
        let mut heap_buf;
        let data_ptr = if data.len() <= stack_buf.len() {
            stack_buf[..data.len()].copy_from_slice(data);
            stack_buf.as_mut_ptr()
        } else {
            heap_buf = data.to_vec();
            heap_buf.as_mut_ptr()
        };

        let mut info = NvI2cInfoV3 {
            i2c_dev_address: address << 1,
            data: data_ptr,
            size: data.len() as u32,
            ..Default::default()
        };

        let mut unknown: u32 = 0;
        let res = unsafe { write_fn(self.gpu_handle, &mut info, &mut unknown) };
        res == 0
    }

    #[cfg(not(target_os = "windows"))]
    #[allow(unused_variables)]
    fn i2c_write(&self, address: u8, data: &[u8]) -> bool {
        false
    }

    #[cfg(target_os = "windows")]
    fn i2c_read(&self, address: u8, length: usize) -> Option<Vec<u8>> {
        let read_fn = self.i2c_read_fn?;
        if !self.initialized {
            return None;
        }

        let mut buf = vec![0u8; length];
        let mut info = NvI2cInfoV3 {
            i2c_dev_address: address << 1,
            data: buf.as_mut_ptr(),
            size: length as u32,
            ..Default::default()
        };

        let mut unknown: u32 = 0;
        let res = unsafe { read_fn(self.gpu_handle, &mut info, &mut unknown) };
        if res == 0 {
            Some(buf)
        } else {
            None
        }
    }

    #[cfg(not(target_os = "windows"))]
    #[allow(unused_variables)]
    fn i2c_read(&self, address: u8, length: usize) -> Option<Vec<u8>> {
        None
    }

    /// Safely scans physical GPUs and candidate I2C addresses for the 0xAB signature.
    pub fn probe_and_connect(&mut self) -> bool {
        if self.init_nvapi().is_err() {
            return false;
        }

        for idx in 0..self.gpu_count {
            let handle = self.gpu_handles[idx];
            self.gpu_handle = handle;
            for &addr in Self::CANDIDATE_ADDRESSES {
                let probe_pkt = [0xAB, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00];
                if self.i2c_write(addr, &probe_pkt) {
                    if let Some(resp) = self.i2c_read(addr, 4) {
                        if !resp.is_empty() && resp[0] == 0xAB {
                            self.gpu_name = self.query_gpu_name(handle);
                            self.active_address = Some(addr);
                            return true;
                        }
                    }
                }
            }
        }

        self.gpu_handle = ptr::null_mut();
        false
    }

    /// Sets a static RGB color across the GPU logo/shroud.
    pub fn apply_color(&mut self, r: u8, g: u8, b: u8, brightness: u8) -> bool {
        if self.active_address.is_none() && !self.probe_and_connect() {
            return false;
        }

        let addr = self.active_address.unwrap();
        let b_clamped = brightness.min(GPU_BRIGHTNESS_MAX);

        let mode_pkt = [
            REG_MODE,
            GPU_MODE_STATIC,
            GPU_SPEED_NORMAL,
            b_clamped,
            0x00,
            0x01,
            0x00,
            0x00,
        ];
        let color_pkt = [REG_COLOR_LEFT_MID, GPU_MODE_STATIC, r, g, b, r, g, b];
        self.i2c_write(addr, &mode_pkt) && self.i2c_write(addr, &color_pkt)
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
    pub fn apply_mode(&mut self, mode: u8, r: u8, g: u8, b: u8, speed: u8, brightness: u8) -> bool {
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

        let mode_pkt = [
            REG_MODE,
            mode,
            s_clamped,
            b_clamped,
            mystery_flag,
            0x01,
            0x00,
            0x00,
        ];
        if !self.i2c_write(addr, &mode_pkt) {
            return false;
        }

        if mode != GPU_MODE_COLOR_CYCLE && mode != GPU_MODE_WAVE {
            let color_pkt = [REG_COLOR_LEFT_MID, mode, r, g, b, r, g, b];
            return self.i2c_write(addr, &color_pkt);
        }

        true
    }

    pub fn turn_off(&mut self) -> bool {
        self.apply_color(0, 0, 0, 0)
    }
}

impl Default for GigabyteGPURGB {
    fn default() -> Self {
        Self::new()
    }
}

impl GpuController for GigabyteGPURGB {
    fn probe_and_connect(&mut self) -> bool {
        GigabyteGPURGB::probe_and_connect(self)
    }

    fn gpu_name(&self) -> &str {
        &self.gpu_name
    }

    fn active_address(&self) -> Option<u8> {
        self.active_address
    }

    fn apply_color(&mut self, r: u8, g: u8, b: u8, brightness: u8) -> bool {
        GigabyteGPURGB::apply_color(self, r, g, b, brightness)
    }

    fn stream_color_fast(&self, r: u8, g: u8, b: u8) -> bool {
        GigabyteGPURGB::stream_color_fast(self, r, g, b)
    }

    fn apply_mode(&mut self, mode: u8, r: u8, g: u8, b: u8, speed: u8, brightness: u8) -> bool {
        GigabyteGPURGB::apply_mode(self, mode, r, g, b, speed, brightness)
    }
}
