from sensor.fpga import flash_pb2 as _flash_pb2
from sensor.fpga import image_pb2 as _image_pb2
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
    STATE_BOOTING: _ClassVar[State]
    STATE_RUNNING: _ClassVar[State]
    STATE_SPI_FLASH: _ClassVar[State]
    STATE_LOW_POWER_IDLE: _ClassVar[State]

class Action(int, metaclass=_enum_type_wrapper.EnumTypeWrapper):
    __slots__ = ()
    ACTION_NONE: _ClassVar[Action]
    ACTION_OFF: _ClassVar[Action]
    ACTION_CAPTURE: _ClassVar[Action]
    ACTION_SPI_FLASH: _ClassVar[Action]
STATE_UNKNOWN: State
STATE_OFF: State
STATE_BOOTING: State
STATE_RUNNING: State
STATE_SPI_FLASH: State
STATE_LOW_POWER_IDLE: State
ACTION_NONE: Action
ACTION_OFF: Action
ACTION_CAPTURE: Action
ACTION_SPI_FLASH: Action

class Status(_message.Message):
    __slots__ = ()
    STATE_FIELD_NUMBER: _ClassVar[int]
    FLASH_FIELD_NUMBER: _ClassVar[int]
    LINE_FIELD_NUMBER: _ClassVar[int]
    state: State
    flash: _flash_pb2.Status
    line: _image_pb2.Line
    def __init__(self, state: _Optional[_Union[State, str]] = ..., flash: _Optional[_Union[_flash_pb2.Status, _Mapping]] = ..., line: _Optional[_Union[_image_pb2.Line, _Mapping]] = ...) -> None: ...

class Command(_message.Message):
    __slots__ = ()
    ACTION_FIELD_NUMBER: _ClassVar[int]
    FLASH_FIELD_NUMBER: _ClassVar[int]
    action: Action
    flash: _flash_pb2.Command
    def __init__(self, action: _Optional[_Union[Action, str]] = ..., flash: _Optional[_Union[_flash_pb2.Command, _Mapping]] = ...) -> None: ...
