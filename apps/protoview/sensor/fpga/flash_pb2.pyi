from google.protobuf.internal import enum_type_wrapper as _enum_type_wrapper
from google.protobuf import descriptor as _descriptor
from google.protobuf import message as _message
from collections.abc import Mapping as _Mapping
from typing import ClassVar as _ClassVar, Optional as _Optional, Union as _Union

DESCRIPTOR: _descriptor.FileDescriptor

class State(int, metaclass=_enum_type_wrapper.EnumTypeWrapper):
    __slots__ = ()
    STATE_UNKNOWN: _ClassVar[State]
    STATE_OFF: _ClassVar[State]
    STATE_ERASING: _ClassVar[State]
    STATE_PROGRAMMING: _ClassVar[State]
    STATE_READOUT: _ClassVar[State]
    STATE_DONE: _ClassVar[State]

class Action(int, metaclass=_enum_type_wrapper.EnumTypeWrapper):
    __slots__ = ()
    ACTION_NONE: _ClassVar[Action]
    ACTION_PROGRAM: _ClassVar[Action]
    ACTION_READOUT: _ClassVar[Action]

class Segment(int, metaclass=_enum_type_wrapper.EnumTypeWrapper):
    __slots__ = ()
    SEGMENT_UNKNOWN: _ClassVar[Segment]
    SEGMENT_FPGA: _ClassVar[Segment]
    SEGMENT_QVGA0: _ClassVar[Segment]
    SEGMENT_QVGA1: _ClassVar[Segment]
    SEGMENT_QVGA2: _ClassVar[Segment]
    SEGMENT_QVGA3: _ClassVar[Segment]
    SEGMENT_QVGA4: _ClassVar[Segment]
    SEGMENT_USER: _ClassVar[Segment]
STATE_UNKNOWN: State
STATE_OFF: State
STATE_ERASING: State
STATE_PROGRAMMING: State
STATE_READOUT: State
STATE_DONE: State
ACTION_NONE: Action
ACTION_PROGRAM: Action
ACTION_READOUT: Action
SEGMENT_UNKNOWN: Segment
SEGMENT_FPGA: Segment
SEGMENT_QVGA0: Segment
SEGMENT_QVGA1: Segment
SEGMENT_QVGA2: Segment
SEGMENT_QVGA3: Segment
SEGMENT_QVGA4: Segment
SEGMENT_USER: Segment

class Status(_message.Message):
    __slots__ = ()
    STATE_FIELD_NUMBER: _ClassVar[int]
    STM_PAGE_REQUEST_FIELD_NUMBER: _ClassVar[int]
    READOUT_PAGE_FIELD_NUMBER: _ClassVar[int]
    state: State
    stm_page_request: int
    readout_page: Page
    def __init__(self, state: _Optional[_Union[State, str]] = ..., stm_page_request: _Optional[int] = ..., readout_page: _Optional[_Union[Page, _Mapping]] = ...) -> None: ...

class Command(_message.Message):
    __slots__ = ()
    ACTION_FIELD_NUMBER: _ClassVar[int]
    SEGMENT_FIELD_NUMBER: _ClassVar[int]
    PAGE_FIELD_NUMBER: _ClassVar[int]
    HOST_PAGE_REQUEST_FIELD_NUMBER: _ClassVar[int]
    action: Action
    segment: Segment
    page: Page
    host_page_request: int
    def __init__(self, action: _Optional[_Union[Action, str]] = ..., segment: _Optional[_Union[Segment, str]] = ..., page: _Optional[_Union[Page, _Mapping]] = ..., host_page_request: _Optional[int] = ...) -> None: ...

class Page(_message.Message):
    __slots__ = ()
    PAGE_NUMBER_FIELD_NUMBER: _ClassVar[int]
    DATA_FIELD_NUMBER: _ClassVar[int]
    CRC_FIELD_NUMBER: _ClassVar[int]
    page_number: int
    data: bytes
    crc: int
    def __init__(self, page_number: _Optional[int] = ..., data: _Optional[bytes] = ..., crc: _Optional[int] = ...) -> None: ...
