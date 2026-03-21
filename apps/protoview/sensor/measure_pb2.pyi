from google.protobuf import descriptor as _descriptor
from google.protobuf import message as _message
from typing import ClassVar as _ClassVar, Optional as _Optional

DESCRIPTOR: _descriptor.FileDescriptor

class Measurement(_message.Message):
    __slots__ = ()
    TEMPERATURE_DEGC_FIELD_NUMBER: _ClassVar[int]
    VDD_MV_FIELD_NUMBER: _ClassVar[int]
    VBAT_MV_FIELD_NUMBER: _ClassVar[int]
    ISENSE_UA_FIELD_NUMBER: _ClassVar[int]
    FPGA_ISENSE_UA_FIELD_NUMBER: _ClassVar[int]
    temperature_degc: int
    vdd_mv: int
    vbat_mv: int
    isense_ua: int
    fpga_isense_ua: int
    def __init__(self, temperature_degc: _Optional[int] = ..., vdd_mv: _Optional[int] = ..., vbat_mv: _Optional[int] = ..., isense_ua: _Optional[int] = ..., fpga_isense_ua: _Optional[int] = ...) -> None: ...
