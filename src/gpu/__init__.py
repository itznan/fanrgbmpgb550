"""
GPU package for Gigabyte RGB Fusion 2.0 Controller.
Re-exports all public symbols for backwards compatibility.
"""

from src.gpu.constants import *  # noqa: F401,F403
from src.gpu.nvapi import NV_I2C_INFO_V3
from src.gpu.fusion import GigabyteGPURGB

__all__ = [
    "GigabyteGPURGB",
    "NV_I2C_INFO_V3",
    # constants
    "NVAPI_INITIALIZE",
    "NVAPI_UNLOAD",
    "NVAPI_ENUM_PHYSICAL_GPUS",
    "NVAPI_GPU_GET_FULL_NAME",
    "NVAPI_I2C_WRITE_EX",
    "NVAPI_I2C_READ_EX",
    "REG_COLOR",
    "REG_MODE",
    "REG_COLOR_LEFT_MID",
    "REG_COLOR_RIGHT",
    "GPU_MODE_STATIC",
    "GPU_MODE_BREATHING",
    "GPU_MODE_COLOR_CYCLE",
    "GPU_MODE_FLASHING",
    "GPU_MODE_GRADIENT",
    "GPU_MODE_COLOR_SHIFT",
    "GPU_MODE_WAVE",
    "GPU_MODE_DUAL_FLASHING",
    "GPU_MODE_TRICOLOR",
    "GPU_SPEED_SLOWEST",
    "GPU_SPEED_SLOW",
    "GPU_SPEED_NORMAL",
    "GPU_SPEED_FAST",
    "GPU_SPEED_FASTEST",
    "GPU_BRIGHTNESS_MIN",
    "GPU_BRIGHTNESS_MAX",
]
