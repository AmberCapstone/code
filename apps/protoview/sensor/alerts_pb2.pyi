from google.protobuf.internal import enum_type_wrapper as _enum_type_wrapper
from google.protobuf import descriptor as _descriptor
from typing import ClassVar as _ClassVar

DESCRIPTOR: _descriptor.FileDescriptor

class Alert(int, metaclass=_enum_type_wrapper.EnumTypeWrapper):
    __slots__ = ()
    LOW_INPUT_POWER: _ClassVar[Alert]
    NO_INPUT_POWER: _ClassVar[Alert]
    LOW_CHARGE: _ClassVar[Alert]
    CAMERA_NOT_RESPONDING_I2C: _ClassVar[Alert]
    CAMERA_IMAGE_TIMEOUT: _ClassVar[Alert]
    FPGA_NOT_CONFIGURING: _ClassVar[Alert]
    FPGA_DATA_TIMEOUT: _ClassVar[Alert]
    WATCHDOG_RESET: _ClassVar[Alert]
LOW_INPUT_POWER: Alert
NO_INPUT_POWER: Alert
LOW_CHARGE: Alert
CAMERA_NOT_RESPONDING_I2C: Alert
CAMERA_IMAGE_TIMEOUT: Alert
FPGA_NOT_CONFIGURING: Alert
FPGA_DATA_TIMEOUT: Alert
WATCHDOG_RESET: Alert
