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
    STATE_CONFIGURING: _ClassVar[State]
    STATE_RUNNING: _ClassVar[State]
    STATE_LOW_POWER_IDLE: _ClassVar[State]

class Action(int, metaclass=_enum_type_wrapper.EnumTypeWrapper):
    __slots__ = ()
    ACTION_NONE: _ClassVar[Action]
    ACTION_RUN: _ClassVar[Action]
    ACTION_RECONFIGURE: _ClassVar[Action]
    ACTION_IDLE: _ClassVar[Action]
STATE_UNKNOWN: State
STATE_OFF: State
STATE_BOOTING: State
STATE_CONFIGURING: State
STATE_RUNNING: State
STATE_LOW_POWER_IDLE: State
ACTION_NONE: Action
ACTION_RUN: Action
ACTION_RECONFIGURE: Action
ACTION_IDLE: Action

class Status(_message.Message):
    __slots__ = ()
    STATE_FIELD_NUMBER: _ClassVar[int]
    CURRENT_SETTINGS_FIELD_NUMBER: _ClassVar[int]
    state: State
    current_settings: Settings
    def __init__(self, state: _Optional[_Union[State, str]] = ..., current_settings: _Optional[_Union[Settings, _Mapping]] = ...) -> None: ...

class Command(_message.Message):
    __slots__ = ()
    ENABLE_POWER_FIELD_NUMBER: _ClassVar[int]
    ACTION_FIELD_NUMBER: _ClassVar[int]
    NEW_SETTINGS_FIELD_NUMBER: _ClassVar[int]
    enable_power: bool
    action: Action
    new_settings: Settings
    def __init__(self, enable_power: _Optional[bool] = ..., action: _Optional[_Union[Action, str]] = ..., new_settings: _Optional[_Union[Settings, _Mapping]] = ...) -> None: ...

class Settings(_message.Message):
    __slots__ = ()
    EXPOSURE_US_FIELD_NUMBER: _ClassVar[int]
    exposure_us: int
    def __init__(self, exposure_us: _Optional[int] = ...) -> None: ...
