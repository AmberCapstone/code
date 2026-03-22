import subprocess
import threading
import time

import cobs
import serial
import serial.tools.list_ports
from google.protobuf import json_format
from textual.app import App, ComposeResult
from textual.containers import Horizontal, Vertical
from textual.reactive import reactive
from textual.widgets import Label, Pretty

if subprocess.call(["sh", "generate_proto.sh"]) != 0:
    exit(1)

from sensor_pb2 import Command, Status


def lpf(old: float, new: float) -> float:
    LPF_ALPHA = 0.90
    if old == 0:
        return new
    return LPF_ALPHA * old + (1 - LPF_ALPHA) * new


class TUI(App):
    CSS_PATH = "app.tcss"

    d_command = reactive(dict())
    d_status = reactive(dict())

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
                yield Label("...", id="lab-speed")
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

            self.dev_status.CopyFrom(status)

    def write(self):
        t = time.time()
        self.last_write_interval = lpf(
            self.last_write_interval, t - self.last_write_time
        )
        self.last_write_time = t

        command = Command()
        self.send(command)

        text = f"\n READ  {self.last_read_interval * 1000:8.2f} ms"
        text += f"\n WRITE {self.last_write_interval * 1000:8.2f} ms"
        self.query_exactly_one("#lab-speed", Label).update(text)

    def send(self, command: Command) -> None:
        b = cobs.Encode(command.SerializeToString())
        self.serial.write(b)
        self.d_command = json_format.MessageToDict(command)

    def watch_d_command(self, value: dict) -> None:
        self.query_exactly_one("#lab-cmd", Pretty).update(value)

    def watch_d_status(self, value: dict) -> None:
        try:
            self.query_exactly_one("#lab-sts", Pretty).update(value)
        except Exception:
            pass


if __name__ == "__main__":
    tui = TUI()
    tui.run()
