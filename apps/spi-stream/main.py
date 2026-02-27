from __future__ import annotations

import random
import time
import serial
import itertools
from cobs import Decoder, Encode

PORT = "COM3"
BAUD = 500_000

# Opcodes https://github.com/damdoy/ice40_ultraplus_examples/blob/master/spi_hw/README.md
OPC_NOP = 0x00
OPC_INIT = 0x01
OPC_INV32 = 0x02
OPC_LEDS = 0x04
OPC_WR_32_CHUNK = 0x06
OPC_RD_32_CHUNK = 0x07

# new opcodes for read/write variable length packets
OPC_WR_32 = 0x08
OPC_RD_32 = 0x09

OPC_WR = 0x01
OPC_RD = 0x02

READ_LATENCY = 4

WIDTH = 320
HEIGHT = 100
PIXELS = HEIGHT * WIDTH


def read_frame(ser: serial.Serial, timeout_s: float = 1) -> bytes | None:
    dec = Decoder()

    rx = ser.read_until(b"\0")
    if dec.Decode(rx):
        return dec.output
    else:
        print("Received invalid COBS frame", rx.hex(" "))
        return None


def send_frame(ser: serial.Serial, payload: bytes) -> None:
    # print("payload:", payload.hex(" "))
    ser.write(Encode(payload))


def cmd_init() -> bytes:
    return bytes([OPC_INIT, 0, 0, 0, 0, 0, 0, 0x11])


def cmd_led(val: int) -> bytes:
    return bytes([OPC_LEDS, val & 0xFF, 0xAA, 0xAA, 0xAA, 0xAA, 0xBB, 0xBB, 0xBB, 0xBB])


def cmd_inv32(data4: bytes) -> bytes:
    assert len(data4) == 4
    return bytes((OPC_INV32, data4[0], data4[1], data4[2], data4[3], 0, 0, 0))


def cmd_wr_32_chunk(chunk4: bytes) -> bytes:
    assert len(chunk4) == 4
    return bytes([OPC_WR_32_CHUNK, *chunk4, 0, 0, 0])


def cmd_rd_32_chunk() -> bytes:
    return bytes([OPC_RD_32_CHUNK, 0, 0, 0, 0, 0, 0, 0])


def extract_data4(resp: bytes) -> bytes | None:
    if resp is None or len(resp) < 8:
        return None
    return resp[4:8]


def wr_32_chunk(ser: serial.Serial, vec16: bytes) -> None:
    assert len(vec16) == 16
    for i in range(0, 16, 4):
        send_frame(ser, cmd_wr_32_chunk(vec16[i : i + 4]))
        _ = read_frame(ser, timeout_s=1.0)


def rd_32_chunk(ser: serial.Serial) -> bytes:
    out = bytearray()
    for _ in range(4):
        send_frame(ser, cmd_rd_32_chunk())
        resp = read_frame(ser, timeout_s=1.0)
        d = extract_data4(resp) if resp else None
        if d is None:
            raise RuntimeError("RD_VEC timeout/short frame")
        out += d
    return bytes(out)


def cmd_wr_32(addr: int, data: bytes) -> bytes:
    assert 0 <= addr <= 0xFF
    return bytes([OPC_WR_32, addr & 0xFF, 0x00]) + data


def cmd_rd_32(addr: int, n: int) -> bytes:
    assert 0 <= addr <= 0xFF
    return bytes([OPC_RD_32, addr & 0xFF, 0x00]) + bytes(16)


def main():
    with serial.Serial(PORT, baudrate=BAUD, timeout=1.0) as ser:
        print(f"Connected to {PORT} @ {BAUD} baud")

        time.sleep(1)

        ser.reset_input_buffer()
        ser.reset_output_buffer()

        time.sleep(1)

        # while True:
        #     data = random.randbytes(512)
        #     send_frame(ser, data)
        #     rx = read_frame(ser)
        #     print(rx == data)
        #     time.sleep(0.5)

        addr = 0
        data_buf = []
        for i in range(0, PIXELS, WIDTH):
            addr = i
            data = random.randbytes(WIDTH)
            data_buf.append((addr, data))
            # print("sending data: ", data.hex(" "))
            print("addr:", addr)
            addr_hi = (addr >> 8) & 0xFF
            addr_lo = addr & 0xFF
            payload = bytes([OPC_WR, addr_hi, addr_lo]) + data
            send_frame(ser, payload)
            resp = read_frame(ser)
            # print("resp = ", resp.hex(" " if resp else "no resp on write"))
            time.sleep(0.01)

        for addr, data in data_buf:
            addr_hi = (addr >> 8) & 0xFF
            addr_lo = addr & 0xFF
            payload = bytes([OPC_RD, addr_hi, addr_lo]) + bytes([0x00]*WIDTH)
            send_frame(ser, payload)
            resp = read_frame(ser)
            if (resp [3:(3+WIDTH)] == data):
                print ("addr:", addr, "\tpass")
            else:
                print("addr: ", addr, "fail: response: ", resp.hex(" "), "expected: ", data.hex(" "))

            # print("resp = ", resp.hex(" ") if resp else "<timeout>")

            time.sleep(0.01)







        # while True: 
        #     data = random.randbytes(WIDTH)
        #     print("sending data: ", data.hex(" "))
        #     addr_hi = (addr >> 8) & 0xFF
        #     addr_lo = addr & 0xFF
        #     payload = bytes([OPC_WR, addr_hi, addr_lo]) + data
        #     send_frame(ser, payload)
        #     resp = read_frame(ser)
        #     # print("resp = ", resp.hex(" " if resp else "no resp on write"))

        #     time.sleep(0.2)
        #     payload = bytes([OPC_RD, addr_hi, addr_lo]) + bytes([0x00]*WIDTH)
        #     send_frame(ser, payload)
        #     resp = read_frame(ser)
        #     if (resp [3:(3+WIDTH)] == data):
        #         print ("Pass")
        #     else:
        #         print("Fail: response: ", resp.hex(" "), "expected: ", data.hex(" "))

        #     # print("resp = ", resp.hex(" ") if resp else "<timeout>")

        #     time.sleep(0.2)



        # resp = None

        # while resp is None:
        #     send_frame(ser, cmd_init())
        #     resp = read_frame(ser)
        #     print("INIT resp:", resp.hex(" ") if resp else "<timeout>")

        # time.sleep(0.5)

        # # LED cycle
        # for val in itertools.cycle([0x01, 0x02, 0x04, 0x03, 0x05, 0x07, 0x00]):
        #     send_frame(ser, cmd_led(val))
        #     resp = read_frame(ser)
        #     print(f"LEDS {val:02x} resp:", resp.hex(" ") if resp else "<timeout>")
        #     time.sleep(0.5)

        # # INV32 tests
        # tests = [
        #     (bytes([0x00, 0x00, 0x00, 0x00]), bytes([0xFF, 0xFF, 0xFF, 0xFF])),
        #     (bytes([0xFF, 0xFF, 0xFF, 0xFF]), bytes([0x00, 0x00, 0x00, 0x00])),
        #     (bytes([0xAA, 0xAA, 0xAA, 0xAA]), bytes([0x55, 0x55, 0x55, 0x55])),
        #     (bytes([0x12, 0x34, 0x56, 0x78]), bytes([0xED, 0xCB, 0xA9, 0x87])),
        # ]

        # # WRITE/READ VEC TESTS

        # for raw, expect in tests:
        #     send_frame(ser, cmd_inv32(raw))
        #     resp = read_frame(ser)

        #     if not resp:
        #         print("INV32 resp: <timeout>")
        #         continue

        #     # Response format depends on your FPGA design,
        #     # but in your verilog you were echoing opcode in byte0,
        #     # then data bytes following.
        #     got = resp
        #     print("INV32 raw resp:", got.hex(" "))

        #     # Try to extract bytes 1..4 as the returned/inverted data
        #     if len(got) >= 5:
        #         got_data = got[3:7]
        #         ok = got_data == expect
        #         print(
        #             "  got:",
        #             got_data.hex(" "),
        #             " expected:",
        #             expect.hex(" "),
        #             " OK" if ok else " FAIL",
        #         )
        #     else:
        #         print("  resp too short to check inversion")

        #     time.sleep(0.5)

        # # # Vector test (16 bytes)
        # # vec = bytes.fromhex("00 01 02 03  10 11 12 13  20 21 22 23  30 31 32 33")

        # # print("Writing vector:", vec.hex(" "))
        # # wr_32_chunk(ser, vec)

        # # got = rd_32_chunk(ser)
        # # print("Read vector   :", got.hex(" "))

        # # print("VEC OK" if got == vec else "VEC FAIL")

        # # data = bytes.fromhex("FF FF FF FF  00 00 00 00  FF FF FF FF  00 00 00 00")
        # data = bytes.fromhex("AA AA AA AA AA AA AA AA")
        # send_frame(ser, cmd_wr_32(0x00, data))
        # resp = read_frame(ser)
        # print("WR resp:", resp.hex(" ") if resp else "<timeout>")

        # send_frame(ser, cmd_rd_32(0x00, len(data)))
        # resp = read_frame(ser)
        # print("Receivd:", resp.hex(" "))


if __name__ == "__main__":
    main()
