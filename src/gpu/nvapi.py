"""
NVIDIA NVAPI ctypes struct definitions.
"""

import ctypes


class NV_I2C_INFO_V3(ctypes.Structure):
    """NVIDIA NVAPI I2C Transaction Info Struct V3."""
    _fields_ = [
        ("version", ctypes.c_uint32),
        ("display_mask", ctypes.c_uint32),
        ("is_ddc_port", ctypes.c_uint8),
        ("i2c_dev_address", ctypes.c_uint8),
        ("i2c_reg_address", ctypes.POINTER(ctypes.c_uint8)),
        ("reg_addr_size", ctypes.c_uint32),
        ("data", ctypes.POINTER(ctypes.c_uint8)),
        ("size", ctypes.c_uint32),
        ("i2c_speed", ctypes.c_uint32),
        ("i2c_speed_khz", ctypes.c_uint32),
        ("port_id", ctypes.c_uint8),
        ("is_port_id_set", ctypes.c_uint32),
    ]


__all__ = ["NV_I2C_INFO_V3"]
