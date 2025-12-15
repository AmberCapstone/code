import time
import serial
import serial.tools.list_ports
import cobs
from google.protobuf import json_format
from textual.app import App, ComposeResult
from textual.widgets import Label, Pretty
from textual.containers import Horizontal, Vertical
from textual.reactive import reactive

from proto.spi_flash_pb2 import Status, Command


class TUI(App):
    CSS_PATH = "app.tcss"

    d_command = reactive(dict())
    d_status = reactive(dict())

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
        self.status = Status()
        self.command = Command()

        self.set_interval(0.1, self.read)
        self.set_interval(0.5, self.write)

    def compose(self) -> ComposeResult:
        with Horizontal():
            with Vertical(classes="box"):
                yield Label("Command")
                yield Pretty("command", id="lab-cmd")
            with Vertical(classes="box"):
                yield Label("Status")
                yield Pretty("status", id="lab-sts")

    def read(self):
        self.curtime = str(time.time())

        cobs_data = self.serial.read_until(b"\0")
        decoder = cobs.Decoder()
        decoder.Decode(cobs_data)
        cobs.Decoder().Decode(cobs_data)

        self.status.ParseFromString(decoder.output)
        self.d_status = json_format.MessageToDict(self.status)

    def watch_d_command(self, value: str) -> None:
        self.query_exactly_one("#lab-cmd", Pretty).update(value)

    def watch_d_status(self, value: str) -> None:
        self.query_exactly_one("#lab-sts", Pretty).update(value)

    def write(self):
        self.command.value += 5
        b = cobs.Encode(self.command.SerializeToString())
        self.serial.write(b)
        self.d_command = json_format.MessageToDict(self.command)


if __name__ == "__main__":
    tui = TUI()
    tui.run()
