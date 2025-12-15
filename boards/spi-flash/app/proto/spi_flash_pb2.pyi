from google.protobuf.internal import enum_type_wrapper as _enum_type_wrapper
from google.protobuf import descriptor as _descriptor
from google.protobuf import message as _message
from typing import ClassVar as _ClassVar, Optional as _Optional, Union as _Union

DESCRIPTOR: _descriptor.FileDescriptor

class State(int, metaclass=_enum_type_wrapper.EnumTypeWrapper):
    __slots__ = ()
    UNKNOWN: _ClassVar[State]
    IDLE: _ClassVar[State]
    RECEIVING: _ClassVar[State]
    WRITING: _ClassVar[State]
    READING: _ClassVar[State]
    ERASING: _ClassVar[State]
UNKNOWN: State
IDLE: State
RECEIVING: State
WRITING: State
READING: State
ERASING: State

class Command(_message.Message):
    __slots__ = ()
    VALUE_FIELD_NUMBER: _ClassVar[int]
    value: int
    def __init__(self, value: _Optional[int] = ...) -> None: ...

class Status(_message.Message):
    __slots__ = ()
    TX_COUNTER_FIELD_NUMBER: _ClassVar[int]
    RX_COUNTER_FIELD_NUMBER: _ClassVar[int]
    TEMPERATURE_DEGC_FIELD_NUMBER: _ClassVar[int]
    VBAT_MV_FIELD_NUMBER: _ClassVar[int]
    VREFINT_MV_FIELD_NUMBER: _ClassVar[int]
    STATE_FIELD_NUMBER: _ClassVar[int]
    ECHO_FIELD_NUMBER: _ClassVar[int]
    tx_counter: int
    rx_counter: int
    temperature_degc: int
    vbat_mv: int
    vrefint_mv: int
    state: State
    echo: int
    def __init__(self, tx_counter: _Optional[int] = ..., rx_counter: _Optional[int] = ..., temperature_degc: _Optional[int] = ..., vbat_mv: _Optional[int] = ..., vrefint_mv: _Optional[int] = ..., state: _Optional[_Union[State, str]] = ..., echo: _Optional[int] = ...) -> None: ...
