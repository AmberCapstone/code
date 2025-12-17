from google.protobuf.internal import enum_type_wrapper as _enum_type_wrapper
from google.protobuf import descriptor as _descriptor
from google.protobuf import message as _message
from collections.abc import Mapping as _Mapping
from typing import ClassVar as _ClassVar, Optional as _Optional, Union as _Union

DESCRIPTOR: _descriptor.FileDescriptor

class Action(int, metaclass=_enum_type_wrapper.EnumTypeWrapper):
    __slots__ = ()
    ACTION_NONE: _ClassVar[Action]
    ACTION_FLASH: _ClassVar[Action]
    ACTION_READOUT: _ClassVar[Action]

class State(int, metaclass=_enum_type_wrapper.EnumTypeWrapper):
    __slots__ = ()
    STATE_UNKNOWN: _ClassVar[State]
    STATE_IDLE: _ClassVar[State]
    STATE_FLASHING: _ClassVar[State]
    STATE_READOUT: _ClassVar[State]
ACTION_NONE: Action
ACTION_FLASH: Action
ACTION_READOUT: Action
STATE_UNKNOWN: State
STATE_IDLE: State
STATE_FLASHING: State
STATE_READOUT: State

class Page(_message.Message):
    __slots__ = ()
    PAGE_NUMBER_FIELD_NUMBER: _ClassVar[int]
    DATA_CRC_FIELD_NUMBER: _ClassVar[int]
    DATA_FIELD_NUMBER: _ClassVar[int]
    page_number: int
    data_crc: int
    data: bytes
    def __init__(self, page_number: _Optional[int] = ..., data_crc: _Optional[int] = ..., data: _Optional[bytes] = ...) -> None: ...

class Command(_message.Message):
    __slots__ = ()
    ACTION_FIELD_NUMBER: _ClassVar[int]
    PAGE_FIELD_NUMBER: _ClassVar[int]
    PAGE_REQUEST_FIELD_NUMBER: _ClassVar[int]
    action: Action
    page: Page
    page_request: int
    def __init__(self, action: _Optional[_Union[Action, str]] = ..., page: _Optional[_Union[Page, _Mapping]] = ..., page_request: _Optional[int] = ...) -> None: ...

class Status(_message.Message):
    __slots__ = ()
    TX_COUNTER_FIELD_NUMBER: _ClassVar[int]
    RX_COUNTER_FIELD_NUMBER: _ClassVar[int]
    TEMPERATURE_DEGC_FIELD_NUMBER: _ClassVar[int]
    VBAT_MV_FIELD_NUMBER: _ClassVar[int]
    VREFINT_MV_FIELD_NUMBER: _ClassVar[int]
    STATE_FIELD_NUMBER: _ClassVar[int]
    PAGE_REQUEST_FIELD_NUMBER: _ClassVar[int]
    tx_counter: int
    rx_counter: int
    temperature_degc: int
    vbat_mv: int
    vrefint_mv: int
    state: State
    page_request: int
    def __init__(self, tx_counter: _Optional[int] = ..., rx_counter: _Optional[int] = ..., temperature_degc: _Optional[int] = ..., vbat_mv: _Optional[int] = ..., vrefint_mv: _Optional[int] = ..., state: _Optional[_Union[State, str]] = ..., page_request: _Optional[int] = ...) -> None: ...
