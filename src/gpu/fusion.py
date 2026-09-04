"""Gigabyte GPU RGB Fusion 2.0 Controller via NVIDIA NVAPI."""

import ctypes
import logging
import time
from typing import Optional, List

from src.gpu.constants import (
    NVAPI_INITIALIZE,
    NVAPI_UNLOAD,
    NVAPI_ENUM_PHYSICAL_GPUS,
    NVAPI_GPU_GET_FULL_NAME,
    NVAPI_I2C_WRITE_EX,
    NVAPI_I2C_READ_EX,
    REG_COLOR,
    REG_MODE,
    REG_COLOR_LEFT_MID,
    REG_COLOR_RIGHT,
    GPU_MODE_STATIC,
    GPU_MODE_BREATHING,
    GPU_MODE_COLOR_CYCLE,
    GPU_MODE_FLASHING,
    GPU_MODE_GRADIENT,
    GPU_MODE_COLOR_SHIFT,
    GPU_MODE_WAVE,
    GPU_MODE_DUAL_FLASHING,
    GPU_MODE_TRICOLOR,
    GPU_SPEED_SLOWEST,
    GPU_SPEED_SLOW,
    GPU_SPEED_NORMAL,
    GPU_SPEED_FAST,
    GPU_SPEED_FASTEST,
    GPU_BRIGHTNESS_MIN,
    GPU_BRIGHTNESS_MAX,
)
from src.gpu.nvapi import NV_I2C_INFO_V3

logger = logging.getLogger(__name__)


class GigabyteGPURGB:
    """Hardware controller for Gigabyte GPU RGB via NVAPI I2C."""

    CANDIDATE_ADDRESSES = [0x32, 0x62, 0x71, 0x70, 0x64, 0x65]

    def __init__(self, target_address: Optional[int] = None):
        self.target_address = target_address
        self.active_address: Optional[int] = None
        self.gpu_handle = None
        self.gpu_name: str = ""
        self.nvapi = None
        self._initialized = False

        self._nvapi_i2c_write = None
        self._nvapi_i2c_read = None

    def _init_nvapi(self) -> bool:
        """Loads nvapi64.dll and binds the required function interfaces."""
        if self._initialized:
            return True

        try:
            self.nvapi = ctypes.windll.LoadLibrary("nvapi64.dll")
        except OSError as e:
            logger.error("Failed to load nvapi64.dll: %s", e)
            return False

        qi = self.nvapi.nvapi_QueryInterface
        qi.restype = ctypes.c_void_p
        qi.argtypes = [ctypes.c_uint32]

        def get_proc(hash_id, prototype):
            ptr = qi(hash_id)
            if not ptr:
                return None
            return prototype(ptr)

        fn_init = get_proc(NVAPI_INITIALIZE, ctypes.WINFUNCTYPE(ctypes.c_int))
        if not fn_init or fn_init() != 0:
            logger.error("NvAPI_Initialize failed.")
            return False

        fn_enum = get_proc(
            NVAPI_ENUM_PHYSICAL_GPUS,
            ctypes.WINFUNCTYPE(ctypes.c_int, ctypes.c_void_p * 64, ctypes.POINTER(ctypes.c_int)),
        )
        if not fn_enum:
            logger.error("NvAPI_EnumPhysicalGPUs not found.")
            return False

        handles = (ctypes.c_void_p * 64)()
        count = ctypes.c_int(0)
        res = fn_enum(handles, ctypes.byref(count))
        if res != 0 or count.value == 0:
            logger.error("No physical NVIDIA GPUs detected via NVAPI.")
            return False

        self.gpu_handle = handles[0]

        fn_name = get_proc(
            NVAPI_GPU_GET_FULL_NAME,
            ctypes.WINFUNCTYPE(ctypes.c_int, ctypes.c_void_p, ctypes.c_char_p),
        )
        if fn_name:
            buf = ctypes.create_string_buffer(128)
            if fn_name(self.gpu_handle, buf) == 0:
                self.gpu_name = buf.value.decode("utf-8", errors="ignore")

        self._nvapi_i2c_write = get_proc(
            NVAPI_I2C_WRITE_EX,
            ctypes.WINFUNCTYPE(
                ctypes.c_int,
                ctypes.c_void_p,
                ctypes.POINTER(NV_I2C_INFO_V3),
                ctypes.POINTER(ctypes.c_uint32),
            ),
        )
        self._nvapi_i2c_read = get_proc(
            NVAPI_I2C_READ_EX,
            ctypes.WINFUNCTYPE(
                ctypes.c_int,
                ctypes.c_void_p,
                ctypes.POINTER(NV_I2C_INFO_V3),
                ctypes.POINTER(ctypes.c_uint32),
            ),
        )

        if not self._nvapi_i2c_write or not self._nvapi_i2c_read:
            logger.error("NvAPI I2C read/write functions unavailable.")
            return False

        self._initialized = True
        return True

    def _i2c_write(self, address: int, data_bytes: List[int]) -> bool:
        """Sends raw bytes via NVAPI I2C write."""
        if not self._initialized or not self._nvapi_i2c_write:
            return False

        buf = (ctypes.c_uint8 * len(data_bytes))(*data_bytes)
        info = NV_I2C_INFO_V3()
        info.version = (3 << 16) | ctypes.sizeof(NV_I2C_INFO_V3)
        info.display_mask = 0
        info.is_ddc_port = 0
        info.i2c_dev_address = address << 1
        info.i2c_reg_address = None
        info.reg_addr_size = 0
        info.data = ctypes.cast(buf, ctypes.POINTER(ctypes.c_uint8))
        info.size = len(data_bytes)
        info.i2c_speed = 0xFFFF
        info.i2c_speed_khz = 0
        info.port_id = 1
        info.is_port_id_set = 1

        unknown = ctypes.c_uint32(0)
        res = self._nvapi_i2c_write(self.gpu_handle, ctypes.byref(info), ctypes.byref(unknown))
        return res == 0

    def _i2c_read(self, address: int, length: int = 4) -> Optional[List[int]]:
        """Reads raw bytes via NVAPI I2C read."""
        if not self._initialized or not self._nvapi_i2c_read:
            return None

        buf = (ctypes.c_uint8 * length)()
        info = NV_I2C_INFO_V3()
        info.version = (3 << 16) | ctypes.sizeof(NV_I2C_INFO_V3)
        info.display_mask = 0
        info.is_ddc_port = 0
        info.i2c_dev_address = address << 1
        info.i2c_reg_address = None
        info.reg_addr_size = 0
        info.data = ctypes.cast(buf, ctypes.POINTER(ctypes.c_uint8))
        info.size = length
        info.i2c_speed = 0xFFFF
        info.i2c_speed_khz = 0
        info.port_id = 1
        info.is_port_id_set = 1

        unknown = ctypes.c_uint32(0)
        res = self._nvapi_i2c_read(self.gpu_handle, ctypes.byref(info), ctypes.byref(unknown))
        if res == 0:
            return list(buf)
        return None

    def probe_and_connect(self) -> bool:
        """Safely scans candidate I2C addresses and validates the 0xAB signature."""
        if not self._init_nvapi():
            return False

        addresses_to_test = [self.target_address] if self.target_address else self.CANDIDATE_ADDRESSES

        for addr in addresses_to_test:
            # Send OpenRGB 0xAB probe packet
            probe_pkt = [0xAB, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00]
            if self._i2c_write(addr, probe_pkt):
                resp = self._i2c_read(addr, 4)
                if resp and len(resp) >= 1 and resp[0] == 0xAB:
                    self.active_address = addr
                    logger.info(
                        "Found Gigabyte RGB controller at 0x%02X on %s. Signature: %s",
                        addr,
                        self.gpu_name,
                        [hex(x) for x in resp],
                    )
                    return True

        logger.warning("No Gigabyte RGB Fusion 2.0 controller confirmed on GPU.")
        return False

    def apply_color(self, r: int, g: int, b: int, brightness: int = GPU_BRIGHTNESS_MAX) -> bool:
        """Sets a static RGB color across the GPU logo/shroud."""
        if not self.active_address and not self.probe_and_connect():
            return False

        brightness = max(GPU_BRIGHTNESS_MIN, min(GPU_BRIGHTNESS_MAX, brightness))
        mode_pkt = [REG_MODE, GPU_MODE_STATIC, GPU_SPEED_NORMAL, brightness, 0x00, 0x01, 0x00, 0x00]
        self._i2c_write(self.active_address, mode_pkt)

        color_pkt = [REG_COLOR_LEFT_MID, GPU_MODE_STATIC, r, g, b, r, g, b]
        return self._i2c_write(self.active_address, color_pkt)

    def stream_color_fast(self, r: int, g: int, b: int) -> bool:
        """High-frequency color update for audio visualizers (skips mode packet)."""
        if not self.active_address:
            return False
        color_pkt = [REG_COLOR_LEFT_MID, GPU_MODE_STATIC, r, g, b, r, g, b]
        return self._i2c_write(self.active_address, color_pkt)

    def apply_mode(
        self,
        mode: int,
        r: int = 255,
        g: int = 0,
        b: int = 0,
        speed: int = GPU_SPEED_NORMAL,
        brightness: int = GPU_BRIGHTNESS_MAX,
    ) -> bool:
        """Applies a hardware animation effect mode."""
        if not self.active_address and not self.probe_and_connect():
            return False

        brightness = max(GPU_BRIGHTNESS_MIN, min(GPU_BRIGHTNESS_MAX, brightness))
        speed = max(GPU_SPEED_SLOWEST, min(GPU_SPEED_FASTEST, speed))

        mystery_flag = 0x00
        if mode == GPU_MODE_GRADIENT or mode == GPU_MODE_TRICOLOR:
            mystery_flag = 0x08

        mode_pkt = [REG_MODE, mode, speed, brightness, mystery_flag, 0x01, 0x00, 0x00]
        self._i2c_write(self.active_address, mode_pkt)

        if mode != GPU_MODE_COLOR_CYCLE and mode != GPU_MODE_WAVE:
            color_pkt = [REG_COLOR_LEFT_MID, mode, r, g, b, r, g, b]
            return self._i2c_write(self.active_address, color_pkt)
        return True

    def turn_off(self) -> bool:
        """Powers off the GPU lighting."""
        return self.apply_color(0, 0, 0, brightness=0)

    def save_to_eeprom(self) -> bool:
        """Permanently writes current lighting state to the GPU microcontroller EEPROM."""
        if not self.active_address:
            return False
        save_pkt = [0xAA, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00]
        return self._i2c_write(self.active_address, save_pkt)
