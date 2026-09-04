"""
Backwards-compatibility shim. All GPU logic now lives in src/gpu/.
"""

from src.gpu import *  # noqa: F401,F403
from src.gpu import GigabyteGPURGB, NV_I2C_INFO_V3  # noqa: F401
