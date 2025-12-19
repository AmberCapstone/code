import binascii
import subprocess
import time
from pathlib import Path

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


class TUI(App):
    CSS_PATH = "app.tcss"

    d_command = reactive(dict())
    d_status = reactive(dict())

    action: Action = Action.ACTION_NONE
    tx_number: int = 0

    flashing_bytes: bytes | None = None
    request_number: int = -1
    sent_any: bool = False
    sequence_number: int = -1
    last_time: float = 0
    dev_state: State = State.STATE_UNKNOWN

    last_read_time: float = 0
    last_write_time: float = 0
    last_read_interval: float = 0
    last_write_interval: float = 0

    def on_mount(self):
        amber_ports = [
            p
            for p in serial.tools.list_ports.comports()
            if p.manufacturer == "amber" and p.product == "SPI Flash"
        ]

        if len(amber_ports) != 1:
            print(f"Expected 1 amber SPI Flash USB device, found {len(amber_ports)}")
            exit(1)

        self.serial = serial.Serial(amber_ports[0].device)

        self.set_interval(0.01, self.read)
        self.set_interval(0.01, self.write)

    def compose(self) -> ComposeResult:
        with Horizontal():
            with Vertical(classes="box"):
                yield Label("...", id="flash-file")
                with Horizontal():
                    yield Button("Flash", id="btn-flash")
                    yield Button("Readout", id="btn-readout")
                yield Label("Command")
                yield Pretty("command", id="lab-cmd")
            with Vertical(classes="box"):
                yield Label("Status")
                yield Pretty("status", id="lab-sts")

    def read(self):
        t = time.time()
        self.last_read_interval = t - self.last_read_time
        self.last_read_time = t

        cobs_data = self.serial.read_until(b"\0")
        decoder = cobs.Decoder()
        decoder.Decode(cobs_data)
        cobs.Decoder().Decode(cobs_data)

        status = Status()
        status.ParseFromString(decoder.output)
        self.d_status = json_format.MessageToDict(status)

        if status.state != State.STATE_IDLE:
            self.action = Action.ACTION_NONE

        self.dev_state = status.state

        if status.flash_status.state == flash.State.STATE_WRITING:
            self.request_number = status.flash_status.page_request
            if self.request_number >= NUM_PAGES:
                self.flashing_bytes = None

    def write(self):
        t = time.time()
        self.last_write_interval = t - self.last_write_time
        self.last_write_time = t

        command = Command(action=self.action, tx_number=self.tx_number)
        label = self.query_exactly_one("#flash-file", Label)

        text = f"Seq Num: {self.sequence_number}"
        if self.dev_state == State.STATE_FLASHING and self.flashing_bytes is not None:
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
                self.sent_any = True
                self.last_time = time.time()

        label.update(text + f"{self.sent_any}")

        b = cobs.Encode(command.SerializeToString())
        self.serial.write(b)

        self.d_command = json_format.MessageToDict(command)
        # self.d_command["data"] = " ".join(f"{b:02x}" for b in self.command.page.data)
        self.d_command["read_ms"] = f"{self.last_read_interval*1000:.2f} ms"
        self.d_command["write_ms"] = f"{self.last_write_interval*1000:.2f} ms"

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
        self.sent_any = False
        self.request_number = -1
        self.action = Action.ACTION_FLASH

    @on(Button.Pressed, "#btn-readout")
    @work
    async def readout(self, event: Button.Pressed) -> None:
        self.action = Action.ACTION_READOUT


if __name__ == "__main__":
    tui = TUI()
    tui.run()
