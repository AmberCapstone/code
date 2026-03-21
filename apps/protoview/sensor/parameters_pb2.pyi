from google.protobuf import descriptor as _descriptor
from google.protobuf import message as _message
from typing import ClassVar as _ClassVar, Optional as _Optional

DESCRIPTOR: _descriptor.FileDescriptor

class Parameters(_message.Message):
    __slots__ = ()
    SUPERCAPACITOR_MF_FIELD_NUMBER: _ClassVar[int]
    supercapacitor_mf: float
    def __init__(self, supercapacitor_mf: _Optional[float] = ...) -> None: ...
