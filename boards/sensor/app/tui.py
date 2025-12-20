import binascii
import subprocess
import time
from pathlib import Path
import threading

import cobs
import serial
import serial.tools.list_ports
from google.protobuf import json_format
from textual import on, work
from textual.app import App, ComposeResult
from textual.containers import Horizontal, Vertical
from textual.reactive import reactive
from textual.widgets import Button, Label, Pretty
from textual_fspicker import FileOpen

if subprocess.call(["sh", "generate_proto.sh"]) != 0:
    exit(1)

import flash_pb2 as flash
from sensor_pb2 import Action, Command, State, Status

PAGE_SIZE = 256
NUM_PAGES = 16 * 16 * 2
TOTAL_SIZE = PAGE_SIZE * NUM_PAGES


def lpf(old: float, new: float) -> float:
    LPF_ALPHA = 0.90
    if old == 0:
        return new
    return LPF_ALPHA * old + (1 - LPF_ALPHA) * new


class TUI(App):
    CSS_PATH = "app.tcss"

    d_command = reactive(dict())
    d_status = reactive(dict())

    action: Action = Action.ACTION_NONE

    flashing_bytes: bytes | None = None
    request_number: int = -1
    sequence_number: int = -1
    last_time: float = 0
    dev_status: Status = Status()

    last_read_time: float = time.time()
    last_write_time: float = time.time()
    last_read_interval: float = 0
    last_write_interval: float = 0

    def on_mount(self):
        amber_ports = [
            p
            for p in serial.tools.list_ports.comports()
            if p.manufacturer == "amber" and p.product == "Sensor Board"
        ]

        if len(amber_ports) != 1:
            self.exit(
                1,
                message=f"Expected 1 amber SPI Flash USB device, found {len(amber_ports)}",
            )

        self.serial = serial.Serial(amber_ports[0].device)

        self.rt = threading.Thread(target=self.read)
        self.rt.start()
        self.set_interval(0.01, self.write)

    def compose(self) -> ComposeResult:
        with Horizontal():
            with Vertical(classes="box"):
                with Horizontal():
                    yield Button("Flash", id="btn-flash")
                    yield Button("Readout", id="btn-readout")
                    yield Button("Reset", id="btn-reset")
                yield Label("...", id="flash-file")
                yield Label("Command")
                yield Pretty("command", id="lab-cmd")
            with Vertical(classes="box"):
                yield Label("Status")
                yield Pretty("status", id="lab-sts")

    def read(self):
        while True:
            t = time.time()
            self.last_read_interval = lpf(
                self.last_read_interval, t - self.last_read_time
            )
            self.last_read_time = t

            cobs_data = self.serial.read_until(b"\0")
            decoder = cobs.Decoder()
            decoder.Decode(cobs_data)
            cobs.Decoder().Decode(cobs_data)

            status = Status()
            status.ParseFromString(decoder.output)
            self.d_status = json_format.MessageToDict(status)

            if status.state != State.STATE_IDLE and self.action != Action.ACTION_RESET:
                self.action = Action.ACTION_NONE

            self.dev_status.CopyFrom(status)

            if status.flash_status.state == flash.State.STATE_PROGRAMMING:
                self.request_number = status.flash_status.page_request
                if self.request_number >= NUM_PAGES:
                    self.flashing_bytes = None

    def write(self):
        t = time.time()
        self.last_write_interval = lpf(
            self.last_write_interval, t - self.last_write_time
        )
        self.last_write_time = t

        command = Command(action=self.action)
        label = self.query_exactly_one("#flash-file", Label)

        text = f"Seq Num: {self.sequence_number}"

        if (
            self.dev_status.state == State.STATE_FLASHING
            and self.dev_status.flash_status.state == flash.State.STATE_PROGRAMMING
            and self.flashing_bytes is not None
        ):
            timeout = time.time() - self.last_time > 3
            if self.sequence_number < self.request_number or timeout:
                self.sequence_number += 1
                if timeout and self.sequence_number != 0:
                    text += f"\nTimeout SN={self.sequence_number}"
                    self.sequence_number -= 1

                idx = self.sequence_number * PAGE_SIZE
                data = self.flashing_bytes[idx : idx + PAGE_SIZE]
                crc = binascii.crc32(self.sequence_number.to_bytes(4, "little"))
                crc = binascii.crc32(data, crc)
                command.page.CopyFrom(
                    flash.Page(page_number=self.sequence_number, data=data, crc=crc)
                )
                self.last_time = time.time()

                self.send(command)
        else:
            self.send(command)

        text += f"\n READ  {self.last_read_interval*1000:8.2f} ms"
        text += f"\n WRITE {self.last_write_interval*1000:8.2f} ms"
        label.update(text)

    def send(self, command: Command) -> None:
        b = cobs.Encode(command.SerializeToString())
        self.serial.write(b)
        self.d_command = json_format.MessageToDict(command)

    def watch_d_command(self, value: dict) -> None:
        self.query_exactly_one("#lab-cmd", Pretty).update(value)

    def watch_d_status(self, value: dict) -> None:
        self.query_exactly_one("#lab-sts", Pretty).update(value)

    @on(Button.Pressed, "#btn-flash")
    @work
    async def flash(self) -> None:
        if opened := await self.push_screen_wait(FileOpen()):
            self.start_flash(opened)

    def start_flash(self, file: Path) -> None:
        label = self.query_exactly_one("#flash-file", Label)

        with open(file, "rb") as f:
            data = f.read()

        size = len(data)
        if size > TOTAL_SIZE:
            label.update(f"File is too large ({size} > {TOTAL_SIZE})")
            return

        data = data + b"\0" * (TOTAL_SIZE - size)
        self.flashing_bytes = data
        self.sequence_number = -1
        self.request_number = -1
        self.action = Action.ACTION_FLASH

    @on(Button.Pressed, "#btn-readout")
    @work
    async def readout(self, event: Button.Pressed) -> None:
        self.action = Action.ACTION_READOUT

    @on(Button.Pressed, "#btn-reset")
    @work
    async def reset(self, event: Button.Pressed) -> None:
        self.action = Action.ACTION_RESET


if __name__ == "__main__":
    tui = TUI()
    tui.run()
