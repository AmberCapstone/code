from __future__ import annotations

import random
import os
import time
import serial
import itertools
import numpy as np
import matplotlib.pyplot as plt
from cobs import Decoder, Encode
from PIL import Image

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
HEIGHT = 240
PIXELS = HEIGHT * WIDTH


def save_framebuffer_as_image(
    frame_bytes: bytes, width: int, height: int, filename: str
) -> None:
    if len(frame_bytes) != width * height:
        raise ValueError(f"Expected {width*height} bytes, got {len(frame_bytes)}")

    img = Image.frombytes("L", (width, height), frame_bytes)
    img.save(filename)
    print(f"Saved image to {filename}")


def save_framebuffer_raw(frame_bytes: bytes, filename: str) -> None:
    with open(filename, "wb") as f:
        f.write(frame_bytes)
    print(f"Saved raw framebuffer to {filename}")


def load_mnist_image(filename: str, index: int) -> bytes:
    with open(filename, "rb") as f:
        f.read(16)  # header
        buf = f.read()

    data = np.frombuffer(buf, dtype=np.uint8)
    images = data.reshape(-1, 28, 28)

    img = images[index]

    plt.figure("MNIST", clear=True)
    plt.imshow(img, cmap="gray")
    # plt.show()
    # plt.show(block=False)
    # plt.pause(1)
    return img.flatten().tobytes()


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


def main():
    seed = int.from_bytes(os.urandom(8), "big")
    random.seed(seed)
    print(f"Random seed: {seed}")
    with serial.Serial(PORT, baudrate=BAUD, timeout=1.0) as ser:
        print(f"Connected to {PORT} @ {BAUD} baud")

        time.sleep(1)

        ser.reset_input_buffer()
        ser.reset_output_buffer()

        framebuf = bytearray()

        time.sleep(2)

        test_payload = bytes([0x00] * 4)
        send_frame(ser, test_payload)
        _ = read_frame(ser)

        written_rows: list[bytes] = []

        start = 0
        for i in range(0, PIXELS//2, WIDTH//2):
            addr = i + start
            addr_hi = (addr >> 8) & 0xFF
            addr_lo = addr & 0xFF

            data = random.randbytes(WIDTH)
            written_rows.append(data)

            payload = bytes([OPC_WR, addr_hi, addr_lo]) + data
            send_frame(ser, payload)
            resp = read_frame(ser)

            # print("addr = ", addr, "resp = ", resp.hex(" ") if resp else "<timeout>")
            # framebuf.extend(resp[3 : 3 + WIDTH])

            time.sleep(0.2)

        start = 0
        row_idx = 0
        for i in range(0, PIXELS//2, WIDTH//2):
            addr = i + start
            addr_hi = (addr >> 8) & 0xFF
            addr_lo = addr & 0xFF

            payload = bytes([OPC_RD, addr_hi, addr_lo]) + bytes([0x00] * WIDTH)
            send_frame(ser, payload)
            resp = read_frame(ser)

            got = resp[3 : 3 + WIDTH]
            exp = written_rows[row_idx]

            if got == exp:
                print(f"READ row {row_idx} addr={addr}: PASS")
            else:
                print(f"READ row {row_idx} addr={addr}: FAIL")
                # print(f"  expected: {exp.hex(' ')}")
                # print(f"  got     : {got.hex(' ')}")

            # print("addr = ", addr, "resp = ", resp.hex(" ") if resp else "<timeout>")
            framebuf.extend(resp[3 : 3 + WIDTH])

            time.sleep(0.2)
            row_idx += 1

        if len(framebuf) == WIDTH * HEIGHT:
            save_framebuffer_raw(framebuf, "frame_qvga.raw")
            save_framebuffer_as_image(framebuf, WIDTH, HEIGHT, "frame_qvga.png")
        else:
            print(
                f"Incomplete frame: got {len(framebuf)} bytes, expected {WIDTH*HEIGHT}"
            )

        # while True:

        #     print("Loading MNIST image")
        #     mnist_bytes = load_mnist_image("train-images-idx3-ubyte", index=2)
        #     assert len(mnist_bytes) == 784

        #     time.sleep(1)

        #     # while True:
        #     #     data = random.randbytes(512)
        #     #     send_frame(ser, data)
        #     #     rx = read_frame(ser)
        #     #     print(rx == data)
        #     #     time.sleep(0.5)
        #     start = 0
        #     for i in range(start, PIXELS, WIDTH):
        #         addr = i
        #         data = mnist_bytes[addr : addr + WIDTH]
        #         print("addr:", addr, "\tsending data: ", data.hex(" "))
        #         # print("addr:", addr)
        #         addr_hi = (addr >> 8) & 0xFF
        #         addr_lo = addr & 0xFF
        #         payload = bytes([OPC_WR, addr_hi, addr_lo]) + data
        #         send_frame(ser, payload)
        #         resp = read_frame(ser)
        #         # print("resp = ", resp.hex(" " if resp else "no resp on write"))
        #         time.sleep(0.01)

        #     for i in range(start, PIXELS, WIDTH):
        #         addr = i
        #         data = mnist_bytes[addr : addr + WIDTH]
        #         addr_hi = (addr >> 8) & 0xFF
        #         addr_lo = addr & 0xFF
        #         payload = bytes([OPC_RD, addr_hi, addr_lo]) + bytes([0x00]*WIDTH)
        #         send_frame(ser, payload)
        #         resp = read_frame(ser)
        #         if (resp [3:(3+WIDTH)] == data):
        #             print ("addr:", addr, "\tpass", "data:", resp.hex(" "))
        #         else:
        #             print("addr: ", addr, "fail: response: ", resp.hex(" "), "expected: ", data.hex(" "))

        #         # print("resp = ", resp.hex(" ") if resp else "<timeout>")

        #         time.sleep(0.01)

        #     print("Loading MNIST image")
        #     mnist_bytes = load_mnist_image("train-images-idx3-ubyte", index=3)
        #     assert len(mnist_bytes) == 784

        #     time.sleep(1)

        # while True:
        #     data = random.randbytes(512)
        #     send_frame(ser, data)
        #     rx = read_frame(ser)
        #     print(rx == data)
        #     time.sleep(0.5)

        # start = 785
        # for i in range(0, PIXELS, WIDTH):
        #     addr = i + start
        #     data = mnist_bytes[addr - start : addr - start + WIDTH]
        #     print("addr:", addr, "\tsending data: ", data.hex(" "))
        #     # print("addr:", addr)
        #     addr_hi = (addr >> 8) & 0xFF
        #     addr_lo = addr & 0xFF
        #     payload = bytes([OPC_WR, addr_hi, addr_lo]) + data
        #     send_frame(ser, payload)
        #     resp = read_frame(ser)
        #     # print("resp = ", resp.hex(" " if resp else "no resp on write"))
        #     time.sleep(0.01)

        # for i in range(0, PIXELS, WIDTH):
        #     addr = i + start
        #     data = mnist_bytes[addr - start : addr - start + WIDTH]
        #     addr_hi = (addr >> 8) & 0xFF
        #     addr_lo = addr & 0xFF
        #     payload = bytes([OPC_RD, addr_hi, addr_lo]) + bytes([0x00]*WIDTH)
        #     send_frame(ser, payload)
        #     resp = read_frame(ser)
        #     if (resp [3:(3+WIDTH)] == data):
        #         print ("addr:", addr, "\tpass", "data:", resp.hex(" "))
        #     else:
        #         print("addr: ", addr, "fail: response: ", resp.hex(" "), "expected: ", data.hex(" "))

        #     # print("resp = ", resp.hex(" ") if resp else "<timeout>")

        #     time.sleep(0.01)

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
