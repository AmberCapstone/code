from sensor import camera_pb2 as _camera_pb2
from sensor import fpga_pb2 as _fpga_pb2
from sensor import measure_pb2 as _measure_pb2
from sensor import parameters_pb2 as _parameters_pb2
from google.protobuf.internal import enum_type_wrapper as _enum_type_wrapper
from google.protobuf import descriptor as _descriptor
from google.protobuf import message as _message
from collections.abc import Mapping as _Mapping
from typing import ClassVar as _ClassVar, Optional as _Optional, Union as _Union

DESCRIPTOR: _descriptor.FileDescriptor

class State(int, metaclass=_enum_type_wrapper.EnumTypeWrapper):
    __slots__ = ()
    STATE_UNKNOWN: _ClassVar[State]
    STATE_MANUAL: _ClassVar[State]
    STATE_CHARGING: _ClassVar[State]
    STATE_CAPTURE: _ClassVar[State]
    STATE_LOW_CHARGE: _ClassVar[State]

class Action(int, metaclass=_enum_type_wrapper.EnumTypeWrapper):
    __slots__ = ()
    ACTION_NONE: _ClassVar[Action]
    ACTION_MONITOR: _ClassVar[Action]
    ACTION_MANUAL: _ClassVar[Action]
STATE_UNKNOWN: State
STATE_MANUAL: State
STATE_CHARGING: State
STATE_CAPTURE: State
STATE_LOW_CHARGE: State
ACTION_NONE: Action
ACTION_MONITOR: Action
ACTION_MANUAL: Action

class Command(_message.Message):
    __slots__ = ()
    ACTION_FIELD_NUMBER: _ClassVar[int]
    FPGA_FIELD_NUMBER: _ClassVar[int]
    CAMERA_FIELD_NUMBER: _ClassVar[int]
    action: Action
    fpga: _fpga_pb2.Command
    camera: _camera_pb2.Command
    def __init__(self, action: _Optional[_Union[Action, str]] = ..., fpga: _Optional[_Union[_fpga_pb2.Command, _Mapping]] = ..., camera: _Optional[_Union[_camera_pb2.Command, _Mapping]] = ...) -> None: ...

class Status(_message.Message):
    __slots__ = ()
    STATE_FIELD_NUMBER: _ClassVar[int]
    MEASUREMENT_FIELD_NUMBER: _ClassVar[int]
    ALERTS_FIELD_NUMBER: _ClassVar[int]
    FPGA_FIELD_NUMBER: _ClassVar[int]
    CAMERA_FIELD_NUMBER: _ClassVar[int]
    TX_COUNTER_FIELD_NUMBER: _ClassVar[int]
    RX_COUNTER_FIELD_NUMBER: _ClassVar[int]
    state: State
    measurement: _measure_pb2.Measurement
    alerts: int
    fpga: _fpga_pb2.Status
    camera: _camera_pb2.Status
    tx_counter: int
    rx_counter: int
    def __init__(self, state: _Optional[_Union[State, str]] = ..., measurement: _Optional[_Union[_measure_pb2.Measurement, _Mapping]] = ..., alerts: _Optional[int] = ..., fpga: _Optional[_Union[_fpga_pb2.Status, _Mapping]] = ..., camera: _Optional[_Union[_camera_pb2.Status, _Mapping]] = ..., tx_counter: _Optional[int] = ..., rx_counter: _Optional[int] = ...) -> None: ...
