from google.protobuf import descriptor as _descriptor
from google.protobuf import message as _message
from typing import ClassVar as _ClassVar, Optional as _Optional

DESCRIPTOR: _descriptor.FileDescriptor

class Line(_message.Message):
    __slots__ = ()
    NUMBER_FIELD_NUMBER: _ClassVar[int]
    DATA_FIELD_NUMBER: _ClassVar[int]
    number: int
    data: bytes
    def __init__(self, number: _Optional[int] = ..., data: _Optional[bytes] = ...) -> None: ...
