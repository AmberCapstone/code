from __future__ import annotations

import time
import serial
from cobs import Decoder, Encode

PORT = "COM3"
BAUD = 2_000_000

# Opcodes https://github.com/damdoy/ice40_ultraplus_examples/blob/master/spi_hw/README.md
OPC_NOP = 0x00
OPC_INIT = 0x01
OPC_INV32 = 0x02
OPC_LEDS = 0x04

def read_frame(ser: serial.Serial, timeout_s: float = 1) -> bytes | None:
    dec = Decoder()
    deadline = time.time() + timeout_s

    while time.time() < deadline:
        chunk = ser.read(256)
        if not chunk:
            continue
        if dec.Decode(chunk):
            payload = bytes(dec.output)
            return payload
        
    return None

def send_frame(ser: serial.Serial, payload: bytes) -> None:
    encoded = bytes(Encode(payload))
    ser.write(encoded)


def cmd_init() -> bytes:
    return bytes ([OPC_INIT, 0, 0, 0, 0, 0, 0, 0x11])

def cmd_led(val: int) -> bytes:
    return bytes([OPC_LEDS, val & 0xFF, 0, 0, 0, 0, 0, 0])

def cmd_inv32(data4: bytes) -> bytes:
    assert len(data4) == 4
    return bytes((OPC_INV32, data4[0], data4[1], data4[2], data4[3], 0, 0, 0))

def main():
    with serial.Serial(PORT, baudrate=BAUD, timeout=0.1) as ser:
        print(f"Connected to {PORT} @ {BAUD} baud")

        ser.reset_input_buffer()
        ser.reset_output_buffer()

        send_frame(ser, cmd_init())
        resp = read_frame(ser, timeout_s=1.0)
        print("INIT resp:", resp.hex(" ") if resp else "<timeout>")

        time.sleep(0.2)

        # LED cycle
        for val in [0x01, 0x02, 0x04, 0x03, 0x05, 0x07, 0x00]:
            send_frame(ser, cmd_led(val))
            resp = read_frame(ser, timeout_s=1.0)
            print(f"LEDS {val:02x} resp:", resp.hex(" ") if resp else "<timeout>")
            time.sleep(0.3)

        # INV32 tests
        tests = [
            (bytes([0x00, 0x00, 0x00, 0x00]), bytes([0xFF, 0xFF, 0xFF, 0xFF])),
            (bytes([0xFF, 0xFF, 0xFF, 0xFF]), bytes([0x00, 0x00, 0x00, 0x00])),
            (bytes([0xAA, 0xAA, 0xAA, 0xAA]), bytes([0x55, 0x55, 0x55, 0x55])),
            (bytes([0x12, 0x34, 0x56, 0x78]), bytes([0xED, 0xCB, 0xA9, 0x87])),
        ]

        for raw, expect in tests:
            send_frame(ser, cmd_inv32(raw))
            resp = read_frame(ser, timeout_s=1.0)

            if not resp:
                print("INV32 resp: <timeout>")
                continue

            # Response format depends on your FPGA design,
            # but in your verilog you were echoing opcode in byte0,
            # then data bytes following.
            got = resp
            print("INV32 raw resp:", got.hex(" "))

            # Try to extract bytes 1..4 as the returned/inverted data
            if len(got) >= 5:
                got_data = got[4:8]
                ok = (got_data == expect)
                print("  got:", got_data.hex(" "), " expected:", expect.hex(" "), " OK" if ok else " FAIL")
            else:
                print("  resp too short to check inversion")

            time.sleep(0.5)

    
if __name__ == "__main__":
    main()
